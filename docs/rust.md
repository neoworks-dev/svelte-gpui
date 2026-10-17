# Calling Rust

The host offers an app `fetch`, `WebSocket` and `files`. Anything else, such as a
database, a hardware API or talking to another process, is your own Rust code. An app
with its own Rust crate **registers named commands**, which the UI calls with `invoke`,
and **emits events**, which the UI hears with `listen`. It can also **register elements**:
views GPUI draws directly, which the UI shows as components (see
[Elements drawn by Rust](#elements-drawn-by-rust)).

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
dependencies. The library's crate name is `svelte_gpui`. Inside this repository, run
`scripts/host.sh --seed` once and build with `CARGO_TARGET_DIR` set to the checkout's
`target/`, so GPUI is not compiled again.

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

## Elements drawn by Rust

Some things have to be drawn by GPUI itself, because they paint too much, or take input too
closely, for the UI to describe: a code editor, a terminal, a map. An app's crate
**registers a view under a name**, and its UI turns the name into a component with
`element(name)`.

```rust
use svelte_gpui::gpui::AppContext as _;
use svelte_gpui::{Elements, Options};

let elements = Elements::new().register("counter", |host, cx| cx.new(|cx| Counter::new(host, cx)));
svelte_gpui::run(options.elements(elements));
```

```svelte
<script lang="ts">
  import { element } from "@neoworks/svelte-gpui";

  const Counter = element<
    { label: string; step: number; onpressed: (event: { count: number }) => void },
    { add(amount: number): number; reset(): number }
  >("counter");

  let counter = $state<ElementMethods<{ add(amount: number): number; reset(): number }>>();
</script>

<Counter key="main" bind:this={counter} style={{ height: 110 }} label="Presses" step={5}
         onpressed={(event) => console.log(event.count)} onkey={(event) => console.log(event.key)} />
<Button onpress={async () => console.log(await counter?.add(10))}><Text>Add 10</Text></Button>
```

`examples/rust-commands/src/counter.rs` is a complete view: it takes props, emits an event when
pressed, answers methods, takes focus, handles some keys and types through its own input handler.

### The view in Rust

A view is a GPUI entity whose type implements `Render` (what it draws) and `AppElement` (how the app
talks to it):

```rust
pub trait AppElement: Render + Sized + 'static {
    fn set_prop(&mut self, name: &str, value: Value, cx: &mut Context<Self>);
    fn call(&mut self, method: &str, args: Value, cx: &mut Context<Self>) -> Result<Value, CommandError>;
    fn focus_handle(&self, cx: &App) -> Option<FocusHandle> { None }
}
```

| Method | |
|---|---|
| `set_prop(name, value, cx)` | The app set a prop, as JSON, or removed it (`null`). It runs once for each prop that has a value when the view appears, and after that only for those that change. |
| `call(method, args, cx)` | The app called a method. `args` is the JSON array of its arguments. The answer resolves the app's promise; a `CommandError` rejects it with its message and code, as for a command. Runs on the window's thread, so keep it short. |
| `focus_handle(cx)` | The view's own `FocusHandle`, if it takes keyboard focus. The view tracks it in its `render` (`.track_focus(&handle)`). |

`Elements::register(name, |host, cx| cx.new(|cx| ...))` takes a factory that builds one view each
time the app shows the component, and gets the `NativeHost` the view emits events through. A name
registered twice, or empty, panics at startup. `Options::elements(elements)` hands the registry to
the host; GPUI itself is re-exported as `svelte_gpui::gpui`, so the crate uses the version the host
draws with (a crate that uses GPUI's macros depends on `gpui = "0.2.2"` as well).

`NativeHost::emit(name, json)` fires the component's `on<name>` listener, with the JSON as what the
listener receives. It is `Clone + Send + Sync`, never blocks, and may be called from any thread; an
event nobody listens to is dropped, and so is one emitted while the view is not in the tree.

### The component in the UI

```ts
function element<Props, Methods>(name: string): Component<ElementProps & Props, ElementMethods<Methods>>;
```

`Props` types the props the view takes and the listeners for its events; `Methods` types its methods,
each of which becomes a function that returns a promise. Both are only types: nothing is checked
until the call.

- **Props** are sent as JSON when they change, so they must be JSON values. A prop that is a
  function is not a prop: one that starts with `on` is a listener, and the others are ignored. A
  `$state` object is read deeply.
- **Listeners:** `onsave` hears `host.emit("save", detail)` and is called with `detail`. The name is
  exact, so `emit("valueChanged")` is `onvalueChanged`.
- **Methods:** `bind:this` gives an instance with the view's methods, plus `focus()` and `blur()`.
  `await editor.insert("text")` calls `call("insert", ["text"])` and resolves with the answer. A call
  made before the view exists waits for it; one on an element that is gone rejects with the code
  `NoElement`. Calls run in the order the app made them, after the props set before them.
- **`style`** lays the node out the way a `View`'s does (`flex: 1`, a height, margins). The view is
  drawn inside that box, so a node with no size has none.
- **`key`** keeps the view across a hot reload (below).
- **`onkey`, `onfocus`, `onblur`, `onlayout` and `onresize`** are the host's own events, as on a
  `View`.

### Drawing

The node owns the view and draws it inside its box, clipped to it. The view's root should fill the
box (`size_full()`). The host draws it as a cached view: GPUI keeps what it painted until the view
calls `cx.notify()`, so a large view that does not change costs nothing per frame, and the UI
changing elsewhere does not redraw it. The host calls `cx.notify()` after every `set_prop` and
`call`. A view that changes for any other reason, such as a timer, a background thread or a focus
change, notifies itself.

An element nobody registered draws an error message where the view would be, and the host's log
names the elements the app does register.

### Keyboard focus and keys

A view that gives a `focus_handle` takes focus when it is pressed (the host focuses it after the
view's own mouse handlers have run, so a view that handles the press itself loses nothing), when the
UI calls `focus()`, and from the scripted `focus:` step.

Keys reach the view first. GPUI dispatches a key down to the focused view's own `on_key_down`
handlers, the way it does in any GPUI app. A view that uses the key calls `cx.stop_propagation()`.
A key it does not use bubbles to the node's `onkey` and the elements around it, and then to
`window`, exactly as a key typed into a `View` does, and `preventDefault()` there stops it from
being typed. Text that no one used goes to the view's input method handler.

For typing, the view implements GPUI's `EntityInputHandler` and registers it while it paints, with
`window.handle_input(&focus_handle, ElementInputHandler::new(bounds, entity), cx)`, as any GPUI text
view does. The platform's input method (composition, dead keys, emoji pickers) then talks to the
view, and the host's own text fields are not involved. The counter example does this in its `render`.

### Hot reload

A hot reload starts the app's script again, but the Rust views are not rebuilt. A component with a
`key` keeps its view: the host sets the view aside when the old app goes, and hands it to the first
node of the new app that asks for an element of the same name and the same key. Its props are
compared with what the view last received, so only those that differ are set again, and a prop the
new app no longer passes is set to `null`. A component without a `key` gets a new view.

A view that nothing asks for within five seconds of the reload is released. A component that is
removed and added again in one update, such as in a `{#key}` block, keeps its view the same way,
and one that stays removed releases it at once. Keyboard focus goes back to the window on a reload.

### Limits

- A method call and its answer are JSON, with no bytes. Use a command or an event for binary data.
- `call` is synchronous Rust on the window's thread. A method that has to wait starts the work on
  a thread and tells the UI with an event.
- A view is drawn inside one node's box and clipped to it.
- The scripted steps `key:` and `type:` on an element send real keystrokes through GPUI, so the
  view's key handlers and input handler see them; `press:` and `down:` reach the UI's listeners
  only, since they do not make a GPUI mouse event.

## Limits

- Arguments and answers are JSON; bytes ride alongside, one buffer per call. There is no
  streaming answer: emit events for that.
- A call cannot be cancelled. Abandoning the promise leaves the command running.
- Commands are registered once, before the app starts. A hot reload (`--watch`) restarts
  the UI with the same commands; changing Rust code means rebuilding the binary.
- Rust cannot call into the UI except by emitting events.
- Only commands in this process: there is one registry per `Options`.
- Elements and commands are separate registries (`Options::elements` and `Options::commands`).

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
