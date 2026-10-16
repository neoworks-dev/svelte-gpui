# Calling Rust

The host offers an app `fetch`, `WebSocket` and `files`. Anything else, such as a
database, a hardware API or talking to another process, is your own Rust code. An app
with its own Rust crate **registers named commands**, which the UI calls with `invoke`,
and **emits events**, which the UI hears with `listen`.

```rust
// src/main.rs
use serde_json::json;
use svelte_gpui::{Commands, Options};

fn main() {
    let commands = Commands::new().command("greet", |args| {
        let name = args["name"].as_str().ok_or("greet needs a name")?;
        Ok::<_, String>(json!(format!("Hello, {name}!")))
    });
    let options = Options::from_command_line().expect("usage: my-app --bundle dist/app.js");
    svelte_gpui::run(options.commands(commands));
}
```

```ts
import { invoke } from "@neoworks/svelte-gpui";

const greeting = await invoke<string>("greet", { name: "Ada" }); // "Hello, Ada!"
```

`examples/rust-commands` is a complete app: a command that hashes a file, one that
lists processes, bytes in and out, an `async` command, errors, and a Rust thread that
emits the system's load every second. Its UI calls all of them.

## Setting up the crate

Your program is a Rust binary that depends on the host library, `svelte_gpui`, instead
of running the `svelte-gpui` command:

```toml
# Cargo.toml
[dependencies]
svelte_gpui_host = { path = "path/to/svelte-gpui/host" }
serde_json = "1"

[patch.crates-io]
xattr = { path = "path/to/svelte-gpui/host/vendor/xattr" }
```

The `[patch]` is the same one `host/Cargo.toml` has: a `libc` incompatibility in GPUI's
dependencies. The library's crate name is `svelte_gpui`. Inside this repository, build
with the shared `CARGO_TARGET_DIR` (see `CLAUDE.md`) so GPUI is not compiled again.

Run it like the plain host, against a build of the UI:

```
bun build.mjs
cargo run --release -- --bundle dist/app.js
```

`Options::from_command_line()` reads the same flags as the `svelte-gpui` command
(`--bundle`, `--width`, `--do`, `--watch`, …; see [Getting started](getting-started.md)).
To ship the app as one file, see [Packaging](packaging.md): there the bundle is embedded
and `Options::embedded(embed!())` replaces the flags.

## Commands in Rust

`Commands` is a registry built before the app starts. Each method takes the name the UI
calls it by, and returns the registry, so they chain.

| Method | The handler |
|---|---|
| `command(name, \|args\| ...)` | `Fn(serde_json::Value) -> Result<R, E>` |
| `command_with_bytes(name, \|args, bytes\| ...)` | `Fn(Value, Option<Vec<u8>>) -> Result<R, E>`; `bytes` is what the UI passed as its third argument. |
| `command_async(name, \|args\| async move { ... })` | `Fn(Value) -> impl Future<Output = Result<R, E>>` |
| `command_async_with_bytes(name, \|args, bytes\| async move { ... })` | Both. |

- **Arguments:** `args` is whatever the UI passed as JSON, and `null` if it passed
  nothing. Read it with `args["name"].as_str()`, or `serde_json::from_value` into your
  own type.
- **Answers:** `R` is anything that converts into an `Output`: a `Value`, `()` (JSON
  `null`), a `Vec<u8>` (bytes only), or a `(Value, Vec<u8>)` (JSON and bytes).
- **Errors:** `E` is anything that converts into a `CommandError`: a `String`, a `&str`,
  a `std::io::Error` (its kind becomes the code), or `CommandError::with_code("NotFound",
  "no such user")` for a message and a code the UI can test. Closures need their error
  type spelled once, as in `Ok::<_, String>(...)`.
- **Names:** a name registered twice, or empty, panics at startup.

### Threads

Every call runs on a **thread of its own**, like a `fetch` or a file operation. A command
may block for as long as it needs and never stalls the window.

A thread per call, rather than a pool, was chosen because a slow command (a long query,
a wait on another process) can then never hold up a fast one, and a thread's start-up
(tens of microseconds) is small next to anything worth calling. If a command must not run
twice at once, take a lock inside it. Calls may therefore overlap, and finish in any order.

A command that panics does not take the app down: the UI's promise rejects with code
`Panicked`.

An **async command** runs its future to completion on the call's thread, so it needs no
executor. It may await channels and anything woken from other threads. A library that
needs its own runtime, such as Tokio's reactor, must start that runtime itself (keep one
in a `static`, and `block_on` inside a plain command or `handle.spawn` from it).

### The window and the UI are on another thread

Commands cannot touch the UI. They answer a call, or push an event; the UI decides what to
show.

## Events from Rust

An `Emitter` pushes events to the app from any thread. Take one from the registry and
move it where it is needed:

```rust
let commands = Commands::new();
let emitter = commands.emitter();
std::thread::spawn(move || loop {
    std::thread::sleep(std::time::Duration::from_secs(1));
    if emitter.has_listeners("tick") {
        emitter.emit("tick", serde_json::json!({ "load": read_load() }));
    }
});
```

| Method | |
|---|---|
| `emit(name, payload)` | Sends the JSON `payload` to everything listening for `name`. Never blocks. |
| `emit_bytes(name, payload, bytes)` | Also sends bytes, which the callback gets as an `ArrayBuffer`. |
| `has_listeners(name)` | Whether the UI listens for `name` right now, so a producer can skip work nobody sees. |

`Emitter` is `Clone + Send + Sync`, and clones from one registry share their listeners.

- **Cost:** an event nobody listens to costs a lock and a lookup, and is dropped. One
  that is heard wakes the window, runs the callback, and draws what changed. Send what
  the UI can show (a value a second, a batch of rows), not every sample.
- **Order:** events from one thread arrive in the order they were emitted.
- **Before the window exists:** events emitted then are dropped; nothing listens yet.

## Calling from JavaScript

```ts
import { invoke, invokeBytes, listen } from "@neoworks/svelte-gpui";
```

### `invoke(name, args?, bytes?)`

```ts
function invoke<T = unknown>(name: string, args?: unknown, bytes?: ArrayBuffer | ArrayBufferView): Promise<T>;
function invokeBytes<T = unknown>(name: string, args?: unknown, bytes?: ArrayBuffer | ArrayBufferView):
  Promise<{ value: T; bytes: Uint8Array }>;
```

`invoke` resolves with the command's JSON. A command that also sends bytes back needs
`invokeBytes`, which returns them next to the value (`bytes` is empty if there were none);
`invoke` drops them.

```ts
const sent = new TextEncoder().encode("hello");
const { value, bytes } = await invokeBytes<{ length: number }>("reverse", null, sent);
```

A failure rejects with an `Error`:

- `message` is the command's.
- `code` is its code when it gave one. The host adds `UnknownCommand` (no command has
  that name; the message lists the ones that exist) and `Panicked`.

```ts
try {
  await invoke("open_database", { path });
} catch (error) {
  if ((error as Error & { code?: string }).code === "NotFound") { /* ... */ }
}
```

An app run by the plain `svelte-gpui` command has no commands, so every `invoke` rejects
with `UnknownCommand`.

### `listen(name, callback)`

```ts
function listen<T = unknown>(name: string, callback: (payload: T, bytes?: ArrayBuffer) => void): () => void;
```

Calls `callback` with each event Rust emits under `name`, and returns a function that
stops listening. Call it from the cleanup of an `$effect` or in `onDestroy`:

```svelte
<script lang="ts">
  import { listen } from "@neoworks/svelte-gpui";

  let load = $state(0);
  $effect(() => listen<{ load: number }>("tick", (tick) => (load = tick.load)));
</script>
```

- Any number of callbacks may listen to one name; they share one stream from the host.
- A callback that throws is logged and does not stop the others.
- Stopping twice is harmless. When the last callback stops, the host stops sending.
- A name needs no registration. Nothing arrives until Rust emits it.

### Typing commands and events

Merge your commands into `CommandMap`, and your events into `EventMap`, and `invoke` and
`listen` check names, arguments and results. Put it in any `.ts` file the app includes:

```ts
declare module "@neoworks/svelte-gpui" {
  interface CommandMap {
    greet: { args: { name: string }; result: string };
  }
  interface EventMap {
    tick: { load: number };
  }
}
```

`invoke("greet", { name })` then returns `Promise<string>`, and `invoke("greet", 5)` is an
error. Names that are not declared still work, typed with the generic `T`.

## Limits

- Arguments and answers are JSON; bytes ride alongside, one buffer per call. There is no
  streaming answer: emit events for that.
- A call cannot be cancelled. Abandoning the promise leaves the command running.
- Commands are registered once, before the app starts. A hot reload (`--watch`) restarts
  the UI with the same commands; changing Rust code means rebuilding the binary.
- Rust cannot call into the UI except by emitting events.
- Only commands in this process: there is one registry per `Options`.

## Under the hood

Calls and events use the channel the rest of the host's input and output use, with no
second one (see [Network and files](io.md#under-the-hood)):

1. `invoke` is `host.io("invoke", call, { name, args }, bytes)`. The host looks the name
   up in the registry (`host/src/io/commands.rs`), runs the handler on a thread, and
   answers through the same sink as a `fetch`: `resolve`, `resolve_bytes` or
   `reject_with_code`.
2. `listen` is a stream, `events.listen { name }`, one per distinct name however many
   callbacks share it. `Emitter::emit` finds the streams for `name` and sends each an
   event. Stopping sends `events.unlisten { stream }`.
3. Each answer or event wakes the window at once, hands it to the app, and draws what
   changed, whichever thread produced it.
