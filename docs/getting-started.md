# Getting started

An app has three parts:

- **Svelte components**, written as usual.
- **An entry file** that hands the root component to `start`.
- **A build script** that bundles everything into one script for the host.

The host is a native binary: it runs the script in QuickJS and draws what it builds
with GPUI.

## Files

```
my-app/
  build.mjs
  src/
    main.ts
    App.svelte
    env.d.ts        only if you import .svg files
  assets/           images the app shows by path, if any
```

### `src/main.ts`

```ts
// The library must be imported before anything that pulls in Svelte's client runtime:
// it installs the globals that runtime reads when it loads.
import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

start(App);
```

`start(Component, props?, options?)` mounts the component into the window. `props` and
`options` are optional. `options.files` limits the app's own file access; see
[Limiting file access](io.md#limiting-file-access). `options.window` sets the window's
title, size and minimum size; see [The window](window.md). Keep the
`@neoworks/svelte-gpui` import first.

### `src/App.svelte`

```svelte
<script lang="ts">
  import { Button, Input, Text, View } from "@neoworks/svelte-gpui/ui";

  let name = $state("");
</script>

<View style={{ flex: 1, padding: 24, gap: 12, background: "#18181b" }}>
  <Text style={{ fontSize: 20, fontWeight: "semibold" }}>Who is there?</Text>
  <Input bind:value={name} placeholder="Your name" style={{ padding: 8, radius: 6, background: "#27272a" }} />
  <Text style={{ color: "#a1a1aa" }}>{name === "" ? "Nobody yet." : `Hello, ${name}.`}</Text>
</View>
```

The root `View` fills the window when it has `flex: 1`. Without it, the root is only as
tall as its content.

### `build.mjs`

```js
import { cp } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { build } from "@neoworks/svelte-gpui/build";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({
  entry: here("./src/main.ts"),
  outfile: here("./dist/app.js"),
});

// Image paths are resolved next to the bundle.
await cp(here("./assets"), here("./dist/assets"), { recursive: true });
```

Inside this repository, examples import `../../build.mjs` instead.

`build` takes these options:

- `entry` and `outfile`.
- `alias`, `define`, `external`, `loader` and `plugins`, which are passed to esbuild.
- `watch: true`, which builds and then rebuilds whenever a source file changes; see
  [Development](#development-hot-reload-and-errors). `build` then resolves to a function
  that stops watching.
- `sourcemap`, which is esbuild's option of the same name. It defaults to `"external"` when
  watching, so the host can show errors against your files, and to off otherwise.

It compiles `.svelte` files and `.svelte.ts` modules with runes on. It strips
TypeScript, and it loads `.svg` imports as their markup text. It writes the bundle to a
temporary file and renames it into place, so nothing reads one half written.

### `src/env.d.ts`

Add this when you import SVG files for `Icon`:

```ts
declare module "*.svg" {
  const markup: string;
  export default markup;
}
```

## Build and run

```
bun build.mjs
svelte-gpui --bundle dist/app.js --width 900 --height 600
```

Host flags:

| Flag | Meaning |
|---|---|
| `--bundle PATH` | The built script. Required. |
| `--width N`, `--height N` | The window's size in logical pixels. They override the size the app asks for in `options.window`, which overrides the default of 668 × 821. |
| `--assets DIR` | Where relative image paths are resolved. The default is the bundle's directory. |
| `--app-id ID` | The window's app id, which the desktop matches window rules against. It overrides a packaged app's own id. |
| `--allow-files PATH` | Limits file access to PATH and what is below it, on top of any limit the app declares. Repeatable. |
| `--watch` | Watch the bundle, and start it again in the same window whenever a build writes it. Implies `--dev`. See below. |
| `--dev` | Show errors in an overlay over the app, with stacks mapped to your source files. Without it, errors are logged to a file and the window carries on. |
| `--do STEP` | A scripted input step, repeatable. See [Testing](testing.md). |
| `--exit-after-ms N` | Quit after N ms and print memory and timing figures. |
| `--report` | With `--exit-after-ms`, also print an outline of the element tree. |

To build the host itself:

```
cargo build --release --manifest-path host/Cargo.toml
```

`console.log`, `info`, `warn`, `error` and `debug` print to the host's standard error,
prefixed with their level.

## Development: hot reload and errors

Run two commands, a build that keeps rebuilding and a host that keeps reloading:

```
bun build.mjs --watch                                   # your build.mjs passes watch: true to build()
svelte-gpui --bundle dist/app.js --watch
```

For a `build.mjs` that reads the flag:

```js
const watch = process.argv.includes("--watch");
await build({ entry: here("./src/main.ts"), outfile: here("./dist/app.js"), watch });
```

Save a `.svelte` file and the window updates, usually within a quarter of a second, in the
same window with the same size, position and focus. The demo does both in one command:

```
bun examples/demo/build.mjs --watch --host host/target/release/svelte-gpui
```

Without `--host` it only watches, and prints the host command to run.

### What a reload does

The host watches the bundle's directory, waits for 40 milliseconds without a write (an
editor can write a file in several steps; esbuild writes once), and then:

- drops the running app: its QuickJS runtime, its elements, its timers, and the sockets,
  file watches and calls it had in flight. Whatever they still produce is ignored;
- forgets what the window kept for the old elements: focus, scroll positions, hover and
  transitions, fields, canvases and images being loaded;
- starts the new bundle in the same window, which keeps its size, position and focus, and
  applies the new title if the app sets one.

The app starts over, so its state is lost. State that lives in a file, or in `files`, is
read again. A bundle that fails to run does not stop the host: the window shows the error,
and the next good build replaces it.

### Errors

With `--dev` (and `--watch`, which implies it), an overlay over the app lists:

- **a build that failed.** The build writes its messages to `dist/app.js.error`, which the
  host shows while the app keeps running the last good build. The next good build removes
  the file and reloads;
- **a bundle that failed to run:** a syntax error, or an exception while it mounts;
- **an exception** in an event handler, an effect, a timer or a close handler;
- **an unhandled promise rejection.**

Each error shows its message, the component it came from, the lines of your source around
where it was thrown, and its stack, mapped back through the source map to your own files
(`src/Files.svelte:42:7`). Frames inside Svelte and the library are dimmed and folded. The
same error again is counted, not listed again. `Copy` puts the errors on the clipboard, and
`Dismiss` or Esc hides the overlay until another error happens. A successful reload clears it.

Line numbers need the bundle to run from its text, which `--dev` does; otherwise the host
keeps it as bytecode without them. Positions in a `.svelte` file are those of the file
itself, types included.

Without `--dev`, as in an installed app, nothing is drawn. Each error is appended to
`errors.log` in `~/.cache/svelte-gpui/<app>/` (the app's `id` when packaged, otherwise the
bundle's name) and the app keeps running.

## Type checking

Components are typed. `svelte-check` catches a misspelt style property, a prop a
primitive does not take, or a handler with the wrong event type:

```
bunx svelte-check --tsconfig ./tsconfig.json
```

Map `@neoworks/svelte-gpui` in your `tsconfig.json` `paths` if your editor cannot
resolve it.

## Next

- [Primitives](primitives.md) lists what you can build with.
- [Styles](styles.md) lists how it can look.
- [The window](window.md) sets its title and size, and asks before it closes.
- [Calling Rust](rust.md) adds your own commands and events.
- [Packaging](packaging.md) turns the app into one executable.
