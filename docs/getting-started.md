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
[Limiting file access](io.md#limiting-file-access). Keep the `@neoworks/svelte-gpui`
import first.

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

It compiles `.svelte` files and `.svelte.ts` modules with runes on. It strips
TypeScript, and it loads `.svg` imports as their markup text.

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
| `--width N`, `--height N` | The window's size in logical pixels. The defaults are 668 × 821. |
| `--assets DIR` | Where relative image paths are resolved. The default is the bundle's directory. |
| `--allow-files PATH` | Limits file access to PATH and what is below it, on top of any limit the app declares. Repeatable. |
| `--do STEP` | A scripted input step, repeatable. See [Testing](testing.md). |
| `--exit-after-ms N` | Quit after N ms and print memory and timing figures. |
| `--report` | With `--exit-after-ms`, also print an outline of the element tree. |

To build the host itself:

```
cargo build --release --manifest-path host/Cargo.toml
```

`console.log`, `info`, `warn`, `error` and `debug` print to the host's standard error,
prefixed with their level.

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
