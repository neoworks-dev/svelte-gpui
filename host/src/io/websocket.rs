//! `WebSocket`: one thread per socket.
//!
//! The thread owns the connection. tungstenite's socket cannot be read from one thread
//! and written from another, so the thread does both. After the handshake the TCP stream
//! is non-blocking, and the thread sleeps in a [`Poller`] until the stream has something
//! to read, can take what is waiting to be written, or the app queues a message, which
//! wakes the poller. An idle socket costs nothing, and a message goes out at once.
//!
//! Operations, all named by the call that opened the socket:
//! - `ws.open` `{ url, protocols }` starts the socket. It is a stream: it produces the
//!   events `open`, `message` and `error`, and ends with `close`.
//! - `ws.send` `{ socket, text? }` sends a text message, or the request's bytes as a
//!   binary one.
//! - `ws.close` `{ socket, code?, reason? }` starts the closing handshake.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{TcpStream, ToSocketAddrs};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use polling::{Event, Events, Poller};
use serde_json::{Value, json};
use tungstenite::client::IntoClientRequest;
use tungstenite::error::ProtocolError;
use tungstenite::http::HeaderValue;
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Error, Message, WebSocket};

use super::Sink;

/// How long connecting, including the TLS and WebSocket handshakes, may take.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// How long a closing handshake waits for the server's answer before giving up.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

/// The close code for a connection that ended without a closing handshake.
const ABNORMAL_CLOSE: u16 = 1006;

/// The close code for a close frame that carried no status.
const NO_STATUS: u16 = 1005;

/// What the app asks of a socket's thread.
enum Command {
    Send(Message),
    Close(Option<CloseFrame>),
    /// Does nothing; sending it tells whether the thread is still there.
    Probe,
}

/// The way to a socket's thread: its commands, and the poller to wake it with.
struct Handle {
    commands: Sender<Command>,
    waker: Arc<Poller>,
}

/// The open sockets, by the call that opened them.
#[derive(Default)]
pub struct Sockets {
    sockets: RefCell<HashMap<u64, Handle>>,
}

impl Sockets {
    /// `ws.open`, `ws.send`, `ws.close`.
    pub fn request(&self, sink: &Sink, op: &str, call: u64, args: Value, bytes: Option<Vec<u8>>) {
        match op {
            "ws.open" => self.open(sink, call, &args),
            "ws.send" => self.send(sink, call, &args, bytes),
            "ws.close" => self.close(sink, call, &args),
            _ => sink.reject(call, format!("svelte-gpui: unknown operation {op}")),
        }
    }

    fn open(&self, sink: &Sink, call: u64, args: &Value) {
        let Some(url) = args["url"].as_str().map(str::to_string) else {
            sink.reject(call, "ws.open needs a url");
            return;
        };
        let protocols: Vec<String> = args["protocols"]
            .as_array()
            .map(|list| list.iter().filter_map(|protocol| protocol.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        self.forget_finished();
        let poller = match Poller::new() {
            Ok(poller) => Arc::new(poller),
            Err(error) => {
                sink.reject(call, format!("could not start the socket: {error}"));
                return;
            }
        };
        let (sender, receiver) = channel();
        self.sockets.borrow_mut().insert(call, Handle { commands: sender, waker: poller.clone() });
        let thread_sink = sink.clone();
        let spawned = std::thread::Builder::new().name(format!("ws-{call}")).spawn(move || {
            let outcome =
                catch_unwind(AssertUnwindSafe(|| run(&thread_sink, call, &url, &protocols, &receiver, &poller)));
            if outcome.is_err() {
                fail(&thread_sink, call, "the socket thread panicked");
            }
        });
        if spawned.is_err() {
            self.sockets.borrow_mut().remove(&call);
            sink.reject(call, "could not start a thread for the socket");
        }
    }

    fn send(&self, sink: &Sink, call: u64, args: &Value, bytes: Option<Vec<u8>>) {
        let message = match (args["text"].as_str(), bytes) {
            (Some(text), _) => Message::text(text),
            (None, Some(bytes)) => Message::binary(bytes),
            (None, None) => Message::binary(Vec::new()),
        };
        self.command(sink, call, args, Command::Send(message));
    }

    fn close(&self, sink: &Sink, call: u64, args: &Value) {
        let frame = args["code"].as_u64().map(|code| CloseFrame {
            code: CloseCode::from(code as u16),
            reason: args["reason"].as_str().unwrap_or("").to_string().into(),
        });
        self.command(sink, call, args, Command::Close(frame));
    }

    /// Hands a command to the socket's thread, and answers `call`. A socket that has
    /// ended is forgotten and its calls are answered all the same: the app already
    /// heard it close.
    fn command(&self, sink: &Sink, call: u64, args: &Value, command: Command) {
        let socket = args["socket"].as_u64().unwrap_or(0);
        let delivered = match self.sockets.borrow().get(&socket) {
            Some(handle) => handle.commands.send(command).is_ok() && handle.waker.notify().is_ok(),
            None => false,
        };
        if !delivered {
            self.sockets.borrow_mut().remove(&socket);
        }
        sink.resolve(call, json!({}));
    }

    /// Drops the senders of sockets whose threads have ended.
    fn forget_finished(&self) {
        self.sockets.borrow_mut().retain(|_, handle| handle.commands.send(Command::Probe).is_ok());
    }
}

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

/// Runs a socket from connecting to its end, and reports every step.
fn run(sink: &Sink, call: u64, url: &str, protocols: &[String], commands: &Receiver<Command>, poller: &Poller) {
    let (mut socket, stream, protocol) = match connect(url, protocols) {
        Ok(connected) => connected,
        Err(message) => {
            fail(sink, call, &message);
            return;
        }
    };
    // Safety: the stream stays open until it is removed from the poller below.
    if let Err(error) = unsafe { poller.add(&stream, Event::readable(0)) } {
        fail(sink, call, &format!("could not watch the socket: {error}"));
        return;
    }
    sink.event(call, json!({ "type": "open", "protocol": protocol }), None);
    let end = pump(sink, call, &mut socket, &stream, commands, poller);
    let _ = poller.delete(&stream);
    sink.close(call, end.json());
}

/// Reports a failure the way a browser does: an error event, then an abnormal close.
fn fail(sink: &Sink, call: u64, message: &str) {
    sink.event(call, json!({ "type": "error", "message": message }), None);
    sink.close(call, End { code: ABNORMAL_CLOSE, reason: String::new(), clean: false }.json());
}

/// Opens the connection and returns it, with the TCP stream under it (now non-blocking)
/// and the sub-protocol the server chose.
fn connect(url: &str, protocols: &[String]) -> Result<(Socket, TcpStream, String), String> {
    let mut request = url.into_client_request().map_err(|error| error.to_string())?;
    if !protocols.is_empty() {
        let value = HeaderValue::from_str(&protocols.join(", ")).map_err(|error| error.to_string())?;
        request.headers_mut().insert("Sec-WebSocket-Protocol", value);
    }
    let uri = request.uri().clone();
    let host = uri.host().ok_or("the url has no host")?.trim_matches(['[', ']']).to_string();
    let port = uri.port_u16().unwrap_or(if uri.scheme_str() == Some("wss") { 443 } else { 80 });
    let stream = connect_tcp(&host, port)?;
    // The clone shares the socket, so what is set on it applies to the stream tungstenite owns.
    let shared = stream.try_clone().map_err(|error| error.to_string())?;
    let (socket, response) =
        tungstenite::client_tls_with_config(request, stream, None, None).map_err(|error| error.to_string())?;
    shared.set_nonblocking(true).map_err(|error| error.to_string())?;
    let protocol = response
        .headers()
        .get("Sec-WebSocket-Protocol")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    Ok((socket, shared, protocol))
}

/// Connects to the first address that answers. The stream waits at most `CONNECT_TIMEOUT`
/// for the handshakes that follow, after which the caller makes it non-blocking.
fn connect_tcp(host: &str, port: u16) -> Result<TcpStream, String> {
    let mut last = format!("could not resolve {host}");
    for address in (host, port).to_socket_addrs().map_err(|error| error.to_string())? {
        match TcpStream::connect_timeout(&address, CONNECT_TIMEOUT) {
            Ok(stream) => {
                let _ = stream.set_nodelay(true);
                let _ = stream.set_read_timeout(Some(CONNECT_TIMEOUT));
                let _ = stream.set_write_timeout(Some(CONNECT_TIMEOUT));
                return Ok(stream);
            }
            Err(error) => last = error.to_string(),
        }
    }
    Err(last)
}

/// How a socket ended.
struct End {
    code: u16,
    reason: String,
    clean: bool,
}

impl End {
    fn json(&self) -> Value {
        json!({ "code": self.code, "reason": self.reason, "wasClean": self.clean })
    }
}

/// Reads messages and sends the app's, until the connection ends. Between rounds the
/// thread sleeps until the stream or the app has something for it.
fn pump(
    sink: &Sink,
    call: u64,
    socket: &mut Socket,
    stream: &TcpStream,
    commands: &Receiver<Command>,
    poller: &Poller,
) -> End {
    let mut received_close: Option<End> = None;
    let mut close_started: Option<Instant> = None;
    let mut events = Events::new();
    loop {
        if let Some(problem) = send_queued(socket, commands, &mut close_started) {
            return failed(sink, call, &problem);
        }
        if let Some(end) = read_available(sink, call, socket, &mut received_close) {
            return end;
        }
        // A message, a pong or a close reply may still be waiting to be written.
        let unwritten = match socket.flush() {
            Ok(()) => false,
            Err(Error::Io(error)) if is_would_block(error.kind()) => true,
            Err(Error::ConnectionClosed | Error::AlreadyClosed) => {
                return received_close.unwrap_or(End { code: NO_STATUS, reason: String::new(), clean: true });
            }
            Err(error) => return failed(sink, call, &error.to_string()),
        };
        let deadline = close_started.map(|started| started + CLOSE_TIMEOUT);
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return End { code: ABNORMAL_CLOSE, reason: String::new(), clean: false };
        }
        events.clear();
        let waited = poller
            .modify(stream, Event::new(0, true, unwritten))
            .and_then(|_| poller.wait(&mut events, deadline.map(|deadline| deadline - Instant::now())));
        if let Err(error) = waited {
            return failed(sink, call, &error.to_string());
        }
    }
}

/// Reads every message that has arrived. Returns how the socket ended, if it did.
fn read_available(sink: &Sink, call: u64, socket: &mut Socket, received_close: &mut Option<End>) -> Option<End> {
    loop {
        match socket.read() {
            Ok(Message::Text(text)) => sink.event(call, json!({ "type": "message", "data": text.as_str() }), None),
            Ok(Message::Binary(bytes)) => {
                sink.event(call, json!({ "type": "message", "binary": true }), Some(bytes.to_vec()))
            }
            Ok(Message::Close(frame)) => *received_close = Some(close_of(frame)),
            // Ping, pong and raw frames: tungstenite answers pings itself.
            Ok(_) => {}
            Err(Error::Io(error)) if is_would_block(error.kind()) => return None,
            Err(Error::ConnectionClosed | Error::AlreadyClosed) => {
                let end = received_close.take();
                return Some(end.unwrap_or(End { code: NO_STATUS, reason: String::new(), clean: true }));
            }
            Err(error) => return Some(failed(sink, call, &error.to_string())),
        }
    }
}

/// Reports a problem with an open socket, which ends it abnormally.
fn failed(sink: &Sink, call: u64, problem: &str) -> End {
    sink.event(call, json!({ "type": "error", "message": problem }), None);
    End { code: ABNORMAL_CLOSE, reason: String::new(), clean: false }
}

/// Writes everything the app queued. Returns what went wrong, if anything.
fn send_queued(
    socket: &mut Socket,
    commands: &Receiver<Command>,
    close_started: &mut Option<Instant>,
) -> Option<String> {
    while let Ok(command) = commands.try_recv() {
        let result = match command {
            Command::Probe => Ok(()),
            // Once closing has started, nothing more may be sent.
            Command::Send(_) if close_started.is_some() => Ok(()),
            Command::Send(message) => socket.send(message),
            Command::Close(_) if close_started.is_some() => Ok(()),
            Command::Close(frame) => {
                *close_started = Some(Instant::now());
                socket.close(frame)
            }
        };
        match result {
            Ok(()) => {}
            Err(Error::Io(error)) if is_would_block(error.kind()) => {}
            Err(Error::ConnectionClosed | Error::AlreadyClosed) => {}
            // The server started closing first; there is nobody to send to.
            Err(Error::Protocol(ProtocolError::SendAfterClosing)) => {}
            Err(error) => return Some(error.to_string()),
        }
    }
    None
}

fn close_of(frame: Option<CloseFrame>) -> End {
    match frame {
        Some(frame) => End { code: u16::from(frame.code), reason: frame.reason.to_string(), clean: true },
        None => End { code: NO_STATUS, reason: String::new(), clean: true },
    }
}

fn is_would_block(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
