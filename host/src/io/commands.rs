//! Calling Rust from the app: commands the app's own crate registers, and events it
//! pushes to the app.
//!
//! A [`Commands`] registry is built before the app starts and handed to the host in
//! `Options::commands`. The app reaches it through two kinds of request:
//! - `invoke` `{ name, args }` calls the command `name`, with the request's bytes if there
//!   are any. It settles once, with the command's result or its error.
//! - `events.listen` `{ name }` is a stream that produces every event [`Emitter::emit`]
//!   sends under `name`, and `events.unlisten` `{ stream }` ends it.
//!
//! Every command call runs on a thread of its own, like a `fetch` or a file operation, so
//! a command may block for as long as it needs and never stalls the window. A thread per
//! call, rather than a pool, means a slow command cannot hold up a fast one; the price is a
//! thread's start-up, which is small next to the calls worth making. A command that has to
//! run one at a time takes a lock of its own.

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde_json::{Value, json};

use super::Sink;

/// Why a command failed: a message for people, and optionally a short code the app can
/// test, which the rejected `Error` carries as `code`.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandError {
    pub message: String,
    pub code: Option<String>,
}

impl CommandError {
    pub fn new(message: impl Into<String>) -> Self {
        CommandError { message: message.into(), code: None }
    }

    /// A failure with a code, such as `NotFound`, that the app can tell apart.
    pub fn with_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        CommandError { message: message.into(), code: Some(code.into()) }
    }
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        CommandError::new(message)
    }
}

impl From<&str> for CommandError {
    fn from(message: &str) -> Self {
        CommandError::new(message)
    }
}

/// An I/O error becomes its message, with the kind (`NotFound`, `PermissionDenied`, …) as
/// the code.
impl From<std::io::Error> for CommandError {
    fn from(error: std::io::Error) -> Self {
        CommandError::with_code(format!("{:?}", error.kind()), error.to_string())
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

/// What a command answers with: JSON, and bytes when it has any. A command can return
/// a `Value`, `()` (JSON `null`), a `Vec<u8>` (bytes only) or a `(Value, Vec<u8>)`; each
/// converts into this.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Output {
    pub json: Value,
    pub bytes: Option<Vec<u8>>,
}

impl From<Value> for Output {
    fn from(json: Value) -> Self {
        Output { json, bytes: None }
    }
}

impl From<()> for Output {
    fn from(_: ()) -> Self {
        Output::default()
    }
}

impl From<Vec<u8>> for Output {
    fn from(bytes: Vec<u8>) -> Self {
        Output { json: Value::Null, bytes: Some(bytes) }
    }
}

impl From<(Value, Vec<u8>)> for Output {
    fn from((json, bytes): (Value, Vec<u8>)) -> Self {
        Output { json, bytes: Some(bytes) }
    }
}

/// What the app sent with a call.
struct Input {
    args: Value,
    bytes: Option<Vec<u8>>,
}

type Handler = Arc<dyn Fn(Input) -> Result<Output, CommandError> + Send + Sync>;

/// The commands an app's Rust side offers, by name, and the [`Emitter`] it pushes events
/// with. Cheap to clone; the registry is shared once running.
#[derive(Clone, Default)]
pub struct Commands {
    handlers: HashMap<String, Handler>,
    emitter: Emitter,
}

impl fmt::Debug for Commands {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Commands").field("names", &self.names()).finish()
    }
}

impl Commands {
    pub fn new() -> Self {
        Commands::default()
    }

    /// A command that takes the call's JSON arguments (`null` if the app passed none) and
    /// answers with JSON. Fails with a message, or a [`CommandError`] that also has a code.
    ///
    /// ```
    /// # use svelte_gpui::Commands;
    /// let commands = Commands::new().command("greet", |args| {
    ///     let name = args["name"].as_str().ok_or("greet needs a name")?;
    ///     Ok::<_, String>(serde_json::json!(format!("Hello, {name}!")))
    /// });
    /// ```
    pub fn command<R, E>(self, name: &str, handler: impl Fn(Value) -> Result<R, E> + Send + Sync + 'static) -> Self
    where
        R: Into<Output>,
        E: Into<CommandError>,
    {
        self.register(name, move |input| handler(input.args).map(Into::into).map_err(Into::into))
    }

    /// Like [`command`](Self::command), and also gets the bytes the app passed, if any.
    pub fn command_with_bytes<R, E>(
        self,
        name: &str,
        handler: impl Fn(Value, Option<Vec<u8>>) -> Result<R, E> + Send + Sync + 'static,
    ) -> Self
    where
        R: Into<Output>,
        E: Into<CommandError>,
    {
        self.register(name, move |input| handler(input.args, input.bytes).map(Into::into).map_err(Into::into))
    }

    /// A command written as an `async` function. Its future runs to completion on the
    /// call's own thread, so it may wait on channels, timers and other threads, but it
    /// needs no executor of its own and cannot rely on one being shared (a library that
    /// insists on Tokio's runtime brings that runtime itself).
    pub fn command_async<F, Fut, R, E>(self, name: &str, handler: F) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>>,
        R: Into<Output>,
        E: Into<CommandError>,
    {
        self.register(name, move |input| {
            futures::executor::block_on(handler(input.args)).map(Into::into).map_err(Into::into)
        })
    }

    /// Like [`command_async`](Self::command_async), and also gets the bytes the app passed.
    pub fn command_async_with_bytes<F, Fut, R, E>(self, name: &str, handler: F) -> Self
    where
        F: Fn(Value, Option<Vec<u8>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>>,
        R: Into<Output>,
        E: Into<CommandError>,
    {
        self.register(name, move |input| {
            futures::executor::block_on(handler(input.args, input.bytes)).map(Into::into).map_err(Into::into)
        })
    }

    /// The way to push events to the app. Clones share one subscription list, so an
    /// emitter taken now works for the whole run, from any thread.
    pub fn emitter(&self) -> Emitter {
        self.emitter.clone()
    }

    /// The registered names, sorted.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.handlers.keys().cloned().collect();
        names.sort();
        names
    }

    fn register(
        mut self,
        name: &str,
        handler: impl Fn(Input) -> Result<Output, CommandError> + Send + Sync + 'static,
    ) -> Self {
        assert!(!name.is_empty(), "a command needs a name");
        let previous = self.handlers.insert(name.to_string(), Arc::new(handler));
        assert!(previous.is_none(), "the command {name:?} is registered twice");
        self
    }
}

/// What the host holds of a [`Commands`] while the app runs: the registry as a snapshot,
/// which any thread may read, and the emitter wired to this run's sink.
#[derive(Clone)]
pub struct Router {
    handlers: Arc<HashMap<String, Handler>>,
    emitter: Emitter,
}

impl Router {
    /// Connects the emitter to `sink`. Subscriptions from an earlier run, whose call
    /// numbers mean nothing to this one, are dropped.
    pub fn new(commands: Commands, sink: &Sink) -> Self {
        commands.emitter.attach(sink.clone());
        Router { handlers: Arc::new(commands.handlers), emitter: commands.emitter }
    }

    /// `invoke`, `events.listen` and `events.unlisten`.
    pub fn request(&self, sink: &Sink, op: &str, call: u64, args: Value, bytes: Option<Vec<u8>>) {
        match op {
            "invoke" => self.invoke(sink, call, args, bytes),
            "events.listen" => self.listen(sink, call, &args),
            "events.unlisten" => self.unlisten(sink, call, &args),
            _ => sink.reject(call, format!("svelte-gpui: unknown operation {op}")),
        }
    }

    fn invoke(&self, sink: &Sink, call: u64, mut args: Value, bytes: Option<Vec<u8>>) {
        let Some(name) = args["name"].as_str().map(str::to_string) else {
            sink.reject(call, "invoke needs the name of a command");
            return;
        };
        let Some(handler) = self.handlers.get(&name).cloned() else {
            sink.reject_with_code(call, "UnknownCommand", self.unknown(&name));
            return;
        };
        let input = Input { args: args["args"].take(), bytes };
        let reply = sink.clone();
        let spawned = std::thread::Builder::new().name(format!("command-{name}")).spawn(move || {
            match catch_unwind(AssertUnwindSafe(|| handler(input))) {
                Ok(Ok(output)) => answer(&reply, call, output),
                Ok(Err(error)) => fail(&reply, call, error),
                Err(panic) => fail(&reply, call, panicked(&name, panic.as_ref())),
            }
        });
        if spawned.is_err() {
            sink.reject(call, "could not start a thread for the command");
        }
    }

    /// The message for a command nobody registered, naming the ones there are.
    fn unknown(&self, name: &str) -> String {
        let mut names: Vec<&str> = self.handlers.keys().map(String::as_str).collect();
        names.sort();
        if names.is_empty() {
            return format!("there is no Rust command named {name:?}: this app registers no commands");
        }
        format!("there is no Rust command named {name:?}; the app registers {}", names.join(", "))
    }

    fn listen(&self, sink: &Sink, call: u64, args: &Value) {
        match args["name"].as_str() {
            Some(name) => self.emitter.subscribe(name, call),
            None => sink.reject(call, "events.listen needs the name of an event"),
        }
    }

    /// Ends the stream `events.listen` opened. The stream closes, and this call resolves.
    fn unlisten(&self, sink: &Sink, call: u64, args: &Value) {
        if let Some(stream) = args["stream"].as_u64() {
            self.emitter.unsubscribe(stream);
            sink.close(stream, json!({}));
        }
        sink.resolve(call, json!({}));
    }
}

fn answer(sink: &Sink, call: u64, output: Output) {
    match output.bytes {
        Some(bytes) => sink.resolve_bytes(call, output.json, bytes),
        None => sink.resolve(call, output.json),
    }
}

fn fail(sink: &Sink, call: u64, error: CommandError) {
    match error.code {
        Some(code) => sink.reject_with_code(call, &code, error.message),
        None => sink.reject(call, error.message),
    }
}

fn panicked(name: &str, panic: &(dyn std::any::Any + Send)) -> CommandError {
    let detail = panic
        .downcast_ref::<&str>()
        .map(|text| text.to_string())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "no message".to_string());
    CommandError::with_code("Panicked", format!("the command {name:?} panicked: {detail}"))
}

#[derive(Default)]
struct Listeners {
    sink: Option<Sink>,
    /// The streams listening to each event name, by call number.
    streams: HashMap<String, Vec<u64>>,
}

/// Pushes events from Rust to the app. Clone it into any thread; `emit` never blocks.
/// The app hears an event with `listen(name, callback)`.
///
/// An event nobody listens to costs a lock and a lookup, and is dropped. Check
/// [`has_listeners`](Self::has_listeners) first to skip work that only an event needs.
/// Each event the app does listen to wakes the window and runs the app's callback, so
/// an emitter should send what the app can show, not every sample it takes.
#[derive(Clone, Default)]
pub struct Emitter {
    listeners: Arc<Mutex<Listeners>>,
}

impl Emitter {
    /// Sends `payload` to everything listening for `name`.
    pub fn emit(&self, name: &str, payload: Value) {
        self.send(name, payload, None);
    }

    /// Sends `payload` and bytes; the callback gets the bytes as an `ArrayBuffer`.
    pub fn emit_bytes(&self, name: &str, payload: Value, bytes: Vec<u8>) {
        self.send(name, payload, Some(bytes));
    }

    /// Whether the app is listening for `name` right now.
    pub fn has_listeners(&self, name: &str) -> bool {
        self.lock().streams.contains_key(name)
    }

    fn send(&self, name: &str, payload: Value, bytes: Option<Vec<u8>>) {
        let (sink, streams) = {
            let listeners = self.lock();
            let (Some(sink), Some(streams)) = (listeners.sink.clone(), listeners.streams.get(name).cloned()) else {
                return;
            };
            (sink, streams)
        };
        for stream in streams {
            sink.event(stream, payload.clone(), bytes.clone());
        }
    }

    fn attach(&self, sink: Sink) {
        let mut listeners = self.lock();
        listeners.sink = Some(sink);
        listeners.streams.clear();
    }

    fn subscribe(&self, name: &str, stream: u64) {
        self.lock().streams.entry(name.to_string()).or_default().push(stream);
    }

    fn unsubscribe(&self, stream: u64) {
        let mut listeners = self.lock();
        for streams in listeners.streams.values_mut() {
            streams.retain(|existing| *existing != stream);
        }
        listeners.streams.retain(|_, streams| !streams.is_empty());
    }

    fn lock(&self) -> MutexGuard<'_, Listeners> {
        self.listeners.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
