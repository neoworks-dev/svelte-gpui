//! The browser target: the host without a window. It serves a built web app (see `buildWeb`
//! in `build.mjs`) over HTTP and answers the app's `host.io` calls over a WebSocket, so
//! `invoke`, `files`, `fetch` and `WebSocket` reach the same Rust code they do in the native
//! window. Only the drawing differs: the page draws with the DOM.
//!
//! One listener does both jobs. It is bound to `127.0.0.1` only. A page is served to any
//! request whose `Host` is the server's own address (which keeps a page on another site, whose
//! name was pointed at this machine, from reading it), and the WebSocket at `/io` is opened only
//! by a page that knows the token, from the server's own origin.
//!
//! The token is random, made when the server starts, and given to the page in the fragment of
//! its address (`http://127.0.0.1:PORT/#token=…`), which a browser never sends to anybody. The
//! page keeps it for the tab, and sends it back in the WebSocket's address.
//!
//! ## The wire
//!
//! Every WebSocket message is binary: four bytes (big endian) with the length of a JSON header,
//! the header, and then the bytes of the call's payload, when it has one.
//!
//! - Page to host: `{ "op", "call", "args" }`, the arguments of `host.io(op, call, args, bytes)`.
//! - Host to page: `{ "call", "settle", "json" }`, what a [`Completion`](crate::io::Completion)
//!   holds, where `settle` is `resolve`, `reject`, `event` or `close` and `json` is the payload.
//!
//! A page has one session at a time: a new connection ends the one before it, so a reload or a
//! second tab takes over the app's events instead of splitting them.
//!
//! The file dialogs of `files.pickFiles` and `files.pickSavePath`, which the window shows with GPUI, are shown by
//! the desktop's file chooser instead (see [`dialogs`]), and give real paths as they do there.

mod dialogs;
mod http;
mod session;

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{self, Read};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

use polling::Poller;

use crate::io::Commands;

/// How to serve an app.
#[derive(Clone, Debug)]
pub struct ServeOptions {
    /// The directory `buildWeb` wrote: `index.html` and what it loads.
    pub dir: PathBuf,
    /// What the app can call in Rust, and the events Rust can push to it. The same registry the
    /// native window takes in [`Options::commands`](crate::Options::commands).
    pub commands: Commands,
    /// Paths file access is narrowed to, on top of the app's own limit.
    pub allow_files: Vec<String>,
    /// The port to listen on, on `127.0.0.1`. Zero, the default, takes any free one.
    pub port: u16,
}

impl ServeOptions {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        ServeOptions { dir: dir.into(), commands: Commands::default(), allow_files: Vec::new(), port: 0 }
    }

    /// These options with `commands` as what the app can call in Rust.
    pub fn commands(mut self, commands: Commands) -> Self {
        self.commands = commands;
        self
    }

    /// These options narrowing file access to `paths` and what is below them.
    pub fn allow_files(mut self, paths: Vec<String>) -> Self {
        self.allow_files = paths;
        self
    }

    /// These options listening on `port`.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Reads the command line of the `svelte-gpui` binary: `--web DIR`, with `--port N` and
    /// `--allow-files PATH` as for the window. `None` without `--web`.
    pub fn from_command_line() -> Option<Self> {
        let arguments: Vec<String> = std::env::args().collect();
        let all = |name: &str| -> Vec<String> {
            let positions = arguments.iter().enumerate().filter(|(_, argument)| *argument == name);
            positions.filter_map(|(index, _)| arguments.get(index + 1).cloned()).collect()
        };
        let dir = all("--web").into_iter().next()?;
        let port = all("--port").into_iter().next().and_then(|port| port.parse().ok()).unwrap_or(0);
        Some(ServeOptions::new(dir).allow_files(all("--allow-files")).port(port))
    }
}

/// What the connections share.
pub(crate) struct Shared {
    pub dir: PathBuf,
    pub commands: Commands,
    pub allow_files: Vec<String>,
    pub port: u16,
    pub token: String,
    /// Which connection holds the app's session now; a connection that finds it is not its own ends.
    pub generation: AtomicU64,
    /// What wakes the session that holds it, so it can end when another takes over.
    pub holder: Mutex<Option<Arc<Poller>>>,
    pub stopping: AtomicBool,
}

/// A running server. It stops when this is dropped.
pub struct Server {
    shared: Arc<Shared>,
    acceptor: Option<JoinHandle<()>>,
}

impl Server {
    pub fn port(&self) -> u16 {
        self.shared.port
    }

    pub fn token(&self) -> &str {
        &self.shared.token
    }

    /// Where to open the app: the address with the token in its fragment.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/#token={}", self.shared.port, self.shared.token)
    }

    /// Blocks for as long as the server runs, which is until the process ends.
    pub fn wait(mut self) {
        if let Some(acceptor) = self.acceptor.take() {
            let _ = acceptor.join();
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shared.stopping.store(true, Ordering::SeqCst);
        // The acceptor is waiting in `accept`; a connection wakes it to see the flag.
        let _ = TcpStream::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, self.shared.port)));
        if let Some(holder) = self.shared.holder.lock().unwrap_or_else(PoisonError::into_inner).take() {
            let _ = holder.notify();
        }
    }
}

/// Starts serving, on threads of its own, and returns at once.
pub fn serve(options: ServeOptions) -> io::Result<Server> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, options.port)))?;
    let port = listener.local_addr()?.port();
    let shared = Arc::new(Shared {
        dir: options.dir,
        commands: options.commands,
        allow_files: options.allow_files,
        port,
        token: make_token(),
        generation: AtomicU64::new(0),
        holder: Mutex::new(None),
        stopping: AtomicBool::new(false),
    });
    let accepting = shared.clone();
    let acceptor = std::thread::Builder::new().name("serve-accept".into()).spawn(move || {
        for stream in listener.incoming() {
            if accepting.stopping.load(Ordering::SeqCst) {
                return;
            }
            let Ok(stream) = stream else { continue };
            let connection = accepting.clone();
            let _ = std::thread::Builder::new().name("serve-connection".into()).spawn(move || {
                http::handle(stream, &connection);
            });
        }
    })?;
    Ok(Server { shared, acceptor: Some(acceptor) })
}

/// Serves until the process ends, after printing where to open the app.
pub fn run_web(options: ServeOptions) {
    let dir = options.dir.clone();
    match serve(options) {
        Ok(server) => {
            println!("serving {} at {}", dir.display(), server.url());
            server.wait();
        }
        Err(error) => {
            eprintln!("svelte-gpui: cannot serve {}: {error}", dir.display());
            std::process::exit(1);
        }
    }
}

/// 256 random bits, as hex. The system's entropy when it has a device for it, and otherwise the
/// keys the standard library seeds its hash maps with, which come from the system as well.
fn make_token() -> String {
    let mut bytes = [0u8; 32];
    let from_device = std::fs::File::open("/dev/urandom").and_then(|mut device| device.read_exact(&mut bytes));
    if from_device.is_err() {
        for chunk in bytes.chunks_mut(8) {
            let mut hasher = RandomState::new().build_hasher();
            hasher.write_u64(std::process::id() as u64);
            chunk.copy_from_slice(&hasher.finish().to_le_bytes());
        }
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Whether two tokens are the same, taking as long for a near miss as for a far one.
pub(crate) fn same_token(given: &str, token: &str) -> bool {
    let (given, token) = (given.as_bytes(), token.as_bytes());
    let mut difference = (given.len() ^ token.len()) as u8;
    for (index, expected) in token.iter().enumerate() {
        difference |= given.get(index).copied().unwrap_or(0) ^ expected;
    }
    difference == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_and_different_each_time() {
        let (first, second) = (make_token(), make_token());
        assert_eq!(first.len(), 64);
        assert_ne!(first, second);
    }

    #[test]
    fn a_token_matches_only_itself() {
        assert!(same_token("abc123", "abc123"));
        assert!(!same_token("abc124", "abc123"));
        assert!(!same_token("abc12", "abc123"));
        assert!(!same_token("", "abc123"));
        assert!(!same_token("abc1234", "abc123"));
    }
}
