//! One browser tab's session: the page's `host.io` calls in, and what they produce out.
//!
//! The thread owns the connection and an [`Io`], the same one the window uses, built for this
//! tab. It sleeps in a [`Poller`] until the socket has something to read, can take what waits to
//! be written, or a call finished (a helper thread hands those over and wakes it). tungstenite's
//! socket cannot be read on one thread and written on another, so this thread does both.

use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, PoisonError};
use std::time::Duration;

use futures::StreamExt;
use futures::channel::oneshot;
use futures::future::{Either, select};
use polling::{Event, Events, Poller};
use serde_json::Value;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::{Error, Message, WebSocket};

use super::Shared;
use super::dialogs;
use crate::io::{Completion, Io};

/// The largest message the page may send: a file it writes, or bytes it hands a command.
const MAX_MESSAGE: usize = 256 * 1024 * 1024;

/// What the handshake may take.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// A message from the page: the call's header, and its bytes.
#[derive(Debug, PartialEq)]
pub(super) struct Call {
    pub op: String,
    pub call: u64,
    pub args: Value,
    pub bytes: Option<Vec<u8>>,
}

/// Splits a message into its header and payload. `None` when it is not one.
pub(super) fn parse_call(message: &[u8]) -> Option<Call> {
    let length = u32::from_be_bytes(message.get(..4)?.try_into().ok()?) as usize;
    let header: Value = serde_json::from_slice(message.get(4..4 + length)?).ok()?;
    let rest = &message[4 + length..];
    Some(Call {
        op: header["op"].as_str()?.to_string(),
        call: header["call"].as_u64()?,
        args: header.get("args").cloned().unwrap_or(Value::Null),
        bytes: (!rest.is_empty()).then(|| rest.to_vec()),
    })
}

/// A completion as a message: the header, and the bytes after it.
pub(super) fn completion_message(completion: &Completion) -> Vec<u8> {
    // The payload is JSON already, so it goes into the header as it is.
    let json = if completion.json.is_empty() { "null" } else { completion.json.as_str() };
    let header =
        format!("{{\"call\":{},\"settle\":\"{}\",\"json\":{}}}", completion.call, completion.settle.name(), json);
    let mut message = Vec::with_capacity(4 + header.len() + completion.bytes.as_ref().map_or(0, Vec::len));
    message.extend_from_slice(&(header.len() as u32).to_be_bytes());
    message.extend_from_slice(header.as_bytes());
    if let Some(bytes) = &completion.bytes {
        message.extend_from_slice(bytes);
    }
    message
}

/// Runs the session of a connection whose request the checks accepted, until it ends. One
/// `alongside` runs next to the page's own session instead of taking it over.
pub(super) fn run(stream: TcpStream, shared: &Shared, alongside: bool) {
    // The clone shares the socket, so what is set on it applies to the stream tungstenite owns.
    let Ok(raw) = stream.try_clone() else { return };
    let _ = stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT));
    let _ = stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT));
    let config = WebSocketConfig::default().max_message_size(Some(MAX_MESSAGE)).max_frame_size(Some(MAX_MESSAGE));
    let Ok(mut socket) = tungstenite::accept_with_config(stream, Some(config)) else { return };
    let _ = raw.set_read_timeout(None);
    let _ = raw.set_write_timeout(None);
    if raw.set_nonblocking(true).is_err() {
        return;
    }
    let Ok(poller) = Poller::new().map(Arc::new) else { return };
    // Safety: the stream stays open until it is removed from the poller below.
    if unsafe { poller.add(&raw, Event::readable(0)) }.is_err() {
        return;
    }

    // This connection takes the app's session, and the one that had it ends; one `alongside` (a
    // frame the page opened beside itself) neither takes it nor gives it up.
    let mine = if alongside {
        None
    } else {
        let mine = shared.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let before = shared.holder.lock().unwrap_or_else(PoisonError::into_inner).replace(poller.clone());
        if let Some(before) = before {
            let _ = before.notify();
        }
        Some(mine)
    };

    let bundle = shared.dir.join("index.html");
    let io = Io::new(bundle.to_str(), &shared.allow_files, shared.commands.clone());
    let (finished, stop) = hand_over(&io, &poller);
    pump(&mut socket, &raw, &poller, &io, &finished, shared, mine);
    drop(stop);
    let _ = poller.delete(&raw);
    let _ = socket.close(None);
    let _ = socket.flush();
}

/// Starts the thread that moves finished calls from the `Io` to the channel this thread reads,
/// waking it each time. Ends when the returned sender is dropped.
fn hand_over(io: &Io, poller: &Arc<Poller>) -> (Receiver<Completion>, oneshot::Sender<()>) {
    let mut completions = io.take_receiver().expect("a new Io has its receiver");
    let (sender, receiver) = channel();
    let (stop, stopped) = oneshot::channel::<()>();
    let waker = poller.clone();
    let _ = std::thread::Builder::new().name("serve-completions".into()).spawn(move || {
        futures::executor::block_on(async move {
            let mut stopped = stopped;
            loop {
                match select(completions.next(), &mut stopped).await {
                    Either::Left((Some(completion), _)) => {
                        if sender.send(completion).is_err() {
                            return;
                        }
                        let _ = waker.notify();
                    }
                    _ => return,
                }
            }
        });
    });
    (receiver, stop)
}

type Socket = WebSocket<TcpStream>;

fn pump(
    socket: &mut Socket,
    raw: &TcpStream,
    poller: &Poller,
    io: &Io,
    finished: &Receiver<Completion>,
    shared: &Shared,
    mine: Option<u64>,
) {
    let mut events = Events::new();
    loop {
        let taken_over = mine.is_some_and(|mine| shared.generation.load(Ordering::SeqCst) != mine);
        if shared.stopping.load(Ordering::SeqCst) || taken_over {
            return;
        }
        while let Ok(completion) = finished.try_recv() {
            match socket.send(Message::binary(completion_message(&completion))) {
                Ok(()) => {}
                Err(Error::Io(error)) if would_block(error.kind()) => {}
                Err(_) => return,
            }
        }
        loop {
            match socket.read() {
                Ok(Message::Binary(message)) => match parse_call(&message) {
                    Some(call) => {
                        io.request(&call.op, call.call, call.args, call.bytes);
                        // The dialogs the window would show, which the desktop shows here (see `dialogs`).
                        dialogs::run(io);
                    }
                    None => eprintln!("svelte-gpui: the page sent a message that is not a call"),
                },
                Ok(Message::Close(_)) => return,
                Ok(_) => {}
                Err(Error::Io(error)) if would_block(error.kind()) => break,
                Err(_) => return,
            }
        }
        // A message, a pong or a close reply may still be waiting to be written.
        let unwritten = match socket.flush() {
            Ok(()) => false,
            Err(Error::Io(error)) if would_block(error.kind()) => true,
            Err(_) => return,
        };
        events.clear();
        let waited = poller.modify(raw, Event::new(0, true, unwritten)).and_then(|_| poller.wait(&mut events, None));
        if waited.is_err() {
            return;
        }
    }
}

fn would_block(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Settle;

    fn message(header: &str, bytes: &[u8]) -> Vec<u8> {
        let mut out = (header.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(header.as_bytes());
        out.extend_from_slice(bytes);
        out
    }

    #[test]
    fn a_call_is_its_header_and_the_bytes_after_it() {
        let call = parse_call(&message(r#"{"op":"fs.writeBytes","call":7,"args":{"path":"/a"}}"#, &[1, 2, 3])).unwrap();
        assert_eq!(call.op, "fs.writeBytes");
        assert_eq!(call.call, 7);
        assert_eq!(call.args["path"], "/a");
        assert_eq!(call.bytes, Some(vec![1, 2, 3]));
        let bare = parse_call(&message(r#"{"op":"fs.paths","call":1}"#, &[])).unwrap();
        assert_eq!(bare.bytes, None);
        assert_eq!(bare.args, Value::Null);
    }

    #[test]
    fn what_is_not_a_call_is_refused() {
        assert_eq!(parse_call(&[]), None);
        assert_eq!(parse_call(&[0, 0, 0, 9, b'{']), None);
        assert_eq!(parse_call(&message(r#"{"call":1}"#, &[])), None);
        assert_eq!(parse_call(&message(r#"{"op":"x"}"#, &[])), None);
    }

    #[test]
    fn a_completion_goes_out_as_a_header_and_its_bytes() {
        let completion = Completion {
            call: 4,
            settle: Settle::Resolve,
            json: r#"{"exists":true}"#.to_string(),
            bytes: Some(vec![9, 8]),
        };
        let out = completion_message(&completion);
        let length = u32::from_be_bytes(out[..4].try_into().unwrap()) as usize;
        let header: Value = serde_json::from_slice(&out[4..4 + length]).unwrap();
        assert_eq!(header["call"], 4);
        assert_eq!(header["settle"], "resolve");
        assert_eq!(header["json"]["exists"], true);
        assert_eq!(&out[4 + length..], &[9, 8]);
    }
}
