//! Work the app asks for that happens off the main thread: network requests, sockets,
//! files. The app calls `host.io(op, call, args, bytes)`; the request runs on a
//! thread, and what it produces comes back through a [`Sink`] as a [`Completion`],
//! which the window hands to the app at once (see `Workbench::deliver`).
//!
//! A call settles once, with `resolve` or `reject`. A stream, such as a socket, keeps
//! its call open and sends `event`s until it ends with `close`.
//!
//! Each area lives in its own module and handles the operations named after it:
//! `fetch`, `ws.*`, `fs.*`, and `invoke` and `events.*`, which reach the Rust code of the
//! app's own crate (see [`commands`]). Operations that need GPUI itself, such as a file dialog,
//! are queued as [`MainJob`]s for the window to run.

pub mod commands;
mod fetch;
mod files;
mod websocket;

use std::cell::RefCell;
use std::rc::Rc;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use serde_json::Value;

pub use commands::{CommandError, Commands, Emitter, Output};

/// How a completion settles its call.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Settle {
    /// The call succeeded; the promise resolves with the payload.
    Resolve,
    /// The call failed; the promise rejects with the payload's `message`.
    Reject,
    /// A stream produced something; the call stays open.
    Event,
    /// A stream ended; the call is closed.
    Close,
    /// An app element's view emitted an event (see `NativeHost::emit`). `call` is the element's node id and the
    /// payload holds the event's `name` and `detail`. It is for the window, and never reaches the app as a call.
    Element,
}

impl Settle {
    pub fn name(self) -> &'static str {
        match self {
            Settle::Resolve => "resolve",
            Settle::Reject => "reject",
            Settle::Event => "event",
            Settle::Close => "close",
            Settle::Element => "element",
        }
    }
}

/// What came back for a call: JSON, and bytes when there are any.
pub struct Completion {
    pub call: u64,
    pub settle: Settle,
    pub json: String,
    pub bytes: Option<Vec<u8>>,
}

/// Where threads send what they produce. Cheap to clone, and usable from any thread.
#[derive(Clone)]
pub struct Sink {
    sender: UnboundedSender<Completion>,
}

impl Sink {
    pub fn send(&self, call: u64, settle: Settle, json: Value, bytes: Option<Vec<u8>>) {
        let _ = self.sender.unbounded_send(Completion { call, settle, json: json.to_string(), bytes });
    }

    pub fn resolve(&self, call: u64, json: Value) {
        self.send(call, Settle::Resolve, json, None);
    }

    pub fn resolve_bytes(&self, call: u64, json: Value, bytes: Vec<u8>) {
        self.send(call, Settle::Resolve, json, Some(bytes));
    }

    /// Fails the call; the promise rejects with an `Error` carrying `message`.
    pub fn reject(&self, call: u64, message: impl Into<String>) {
        self.send(call, Settle::Reject, serde_json::json!({ "message": message.into() }), None);
    }

    /// Fails the call with a code the app can tell apart, such as `NotFound`, which the
    /// rejected `Error` carries as `code`.
    pub fn reject_with_code(&self, call: u64, code: &str, message: impl Into<String>) {
        self.send(call, Settle::Reject, serde_json::json!({ "message": message.into(), "code": code }), None);
    }

    pub fn event(&self, call: u64, json: Value, bytes: Option<Vec<u8>>) {
        self.send(call, Settle::Event, json, bytes);
    }

    pub fn close(&self, call: u64, json: Value) {
        self.send(call, Settle::Close, json, None);
    }

    /// Reports an event from the view drawn in the element `node`.
    pub fn element_event(&self, node: u32, name: &str, detail: Value) {
        self.send(node as u64, Settle::Element, serde_json::json!({ "name": name, "detail": detail }), None);
    }
}

/// A request that must run on the main thread with GPUI, such as a file dialog.
pub struct MainJob {
    pub op: String,
    pub call: u64,
    pub args: Value,
}

/// The app's way out: routes each request to the module that handles it.
pub struct Io {
    sink: Sink,
    receiver: RefCell<Option<UnboundedReceiver<Completion>>>,
    /// Requests the window has to run, taken by `Workbench::run_main_jobs`.
    pub main_jobs: Rc<RefCell<Vec<MainJob>>>,
    pub sockets: websocket::Sockets,
    pub files: files::Files,
    /// The app's own Rust commands, and the events they push.
    commands: commands::Router,
}

impl Io {
    /// See [`files::Files::new`] for `bundle` and `allow_files`.
    pub fn new(bundle: Option<&str>, allow_files: &[String], commands: Commands) -> Self {
        let (sender, receiver) = unbounded();
        let sink = Sink { sender };
        Io {
            commands: commands::Router::new(commands, &sink),
            sink,
            receiver: RefCell::new(Some(receiver)),
            main_jobs: Rc::new(RefCell::new(Vec::new())),
            sockets: websocket::Sockets::default(),
            files: files::Files::new(bundle, allow_files),
        }
    }

    pub fn sink(&self) -> Sink {
        self.sink.clone()
    }

    /// The other end of the sink, for the window to listen on. There is only one.
    pub fn take_receiver(&self) -> Option<UnboundedReceiver<Completion>> {
        self.receiver.borrow_mut().take()
    }

    /// Starts an operation. It never blocks: whatever takes time runs on a thread and
    /// settles `call` through the sink.
    pub fn request(&self, op: &str, call: u64, args: Value, bytes: Option<Vec<u8>>) {
        match op {
            "fetch" => fetch::fetch(&self.sink, call, args, bytes),
            op if op.starts_with("ws.") => self.sockets.request(&self.sink, op, call, args, bytes),
            op if op.starts_with("fs.") => self.files.request(&self.sink, &self.main_jobs, op, call, args, bytes),
            "invoke" | "events.listen" | "events.unlisten" => self.commands.request(&self.sink, op, call, args, bytes),
            _ => self.sink.reject(call, format!("svelte-gpui: unknown operation {op}")),
        }
    }
}

impl Drop for Io {
    /// Ends the sockets and watches the app left running. Their threads end on their own, and
    /// what they still send has no receiver to reach.
    fn drop(&mut self) {
        self.sockets.close_all();
        self.files.stop_watches();
    }
}
