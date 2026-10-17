//! A svelte-gpui app with its own Rust code. The UI calls the commands registered here
//! with `invoke`, and hears the events this program emits with `listen`. It also registers a view
//! drawn by GPUI (`counter.rs`), which the UI shows with `element("counter")`.
//!
//! Run it against a build of the UI:
//!
//! ```text
//! bun examples/rust-commands/build.mjs
//! cargo run --release -- --bundle examples/rust-commands/dist/rust-commands.js
//! ```
//!
//! `bun scripts/package.mjs --app examples/rust-commands` builds it with the UI packed in.

mod counter;
mod system;

use std::time::Duration;

use counter::Counter;
use serde_json::{Value, json};
use svelte_gpui::gpui::AppContext as _;
use svelte_gpui::{CommandError, Commands, Elements, Emitter, Options};

fn main() {
    let commands = Commands::new()
        // The simplest command: JSON in, JSON out.
        .command("greet", |args| {
            let name = args["name"].as_str().filter(|name| !name.is_empty()).ok_or("greet needs a name")?;
            Ok::<_, CommandError>(json!(format!("Hello, {name}, from Rust!")))
        })
        // Real work, off the window's thread. A missing file fails with the code NotFound.
        .command("hash_file", |args| Ok::<_, CommandError>(system::hash_file(args["path"].as_str().unwrap_or(""))?))
        .command("list_processes", |args| {
            let limit = args["limit"].as_u64().unwrap_or(10) as usize;
            Ok::<_, CommandError>(json!(system::list_processes(limit)?))
        })
        // Bytes in and bytes out: the answer is the same bytes reversed.
        .command_with_bytes("reverse_bytes", |_, bytes| {
            let mut bytes = bytes.ok_or("reverse_bytes needs bytes")?;
            let checksum = bytes.iter().fold(0u32, |sum, byte| sum.wrapping_add(*byte as u32));
            bytes.reverse();
            Ok::<_, CommandError>((json!({ "length": bytes.len(), "checksum": checksum }), bytes))
        })
        // An async command: it waits for another thread to answer.
        .command_async("wait", |args| async move {
            let millis = args["millis"].as_u64().unwrap_or(500);
            let (sender, receiver) = futures::channel::oneshot::channel();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(millis));
                let _ = sender.send(());
            });
            receiver.await.map_err(|_| "the timer thread ended")?;
            Ok::<_, CommandError>(json!(format!("waited {millis} ms")))
        })
        // A failure with a code the UI can test.
        .command("fail", |_| Err::<Value, _>(CommandError::with_code("Teapot", "I am a teapot")));

    spawn_ticker(commands.emitter());
    // A view drawn by GPUI; the UI shows it with `element("counter")`.
    let elements = Elements::new().register("counter", |host, cx| cx.new(|cx| Counter::new(host, cx)));
    svelte_gpui::run(options().commands(commands).elements(elements));
}

/// Emits `tick` once a second, with the system's load, for as long as the app runs.
fn spawn_ticker(emitter: Emitter) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            // Nobody to tell: skip reading /proc.
            if emitter.has_listeners("tick") {
                emitter.emit("tick", system::tick());
            }
        }
    });
}

#[cfg(feature = "package")]
fn options() -> Options {
    Options::embedded(svelte_gpui::embed!())
}

#[cfg(not(feature = "package"))]
fn options() -> Options {
    Options::from_command_line().unwrap_or_else(|| {
        eprintln!("usage: rust-commands --bundle examples/rust-commands/dist/rust-commands.js");
        std::process::exit(2);
    })
}
