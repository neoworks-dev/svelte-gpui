---
name: svelte-gpui
description: Write, change or debug apps built with svelte-gpui (Svelte 5 components drawn natively by GPUI, no DOM or CSS). Use when editing .svelte files that import @neoworks/svelte-gpui, building UI from View/Text/Button/Input/Scroll/List/Image/Icon/Canvas/Presence, styling with style objects, or testing such an app headless.
---

svelte-gpui runs Svelte 5 in QuickJS inside a Rust host that draws with GPUI. Runes,
snippets, `{#each}`, `svelte/motion` and stores work as on the web. **There is no
browser underneath.** HTML elements, CSS, `class`, the DOM API and `localStorage`
do not exist. Code written from web habits compiles and then draws nothing. `fetch`,
`WebSocket` and a `files` API do exist, run by the host (`docs/io.md`).

Full docs: `docs/` in the svelte-gpui repo. Start at `docs/README.md`. The types in
`runtime/ui/style.ts`, `runtime/ui/events.ts` and each `runtime/ui/*.svelte` `Props`
are the reference. The app in `examples/demo` uses every feature.

## Rules

1. **Only primitives draw.** Import from `@neoworks/svelte-gpui/ui`: `View`, `Text`,
   `Button`, `Input`, `Scroll`, `List`, `Image`, `Icon`, `Canvas`, `Presence`, `Overlay`. Never
   write `<div>`, `<span>`, `<button>`, `<input>`, `<img>`, `<svg>` or `<canvas>`.
2. **Style with objects, never CSS.** Use `style={{ … }}`, plus `hover`, `active` and
   `focus` objects for states. There is no `class`, no `<style>` block, no CSS strings
   and no units: numbers are pixels, and strings are `"50%"`, `"full"` or `"auto"`.
3. **Use the host's event names:** `onpress` (not `onclick`), `onkey` (not
   `onkeydown`, except on `<svelte:window>`), `ondown`/`onmove`/`onup` (not
   mouse or pointer events), and `oninput`/`onsubmit`/`oncommit` on `Input`, which
   receive the text, not an event.
4. **Nothing bubbles except keys.** A handler hears only its own element. `onkey`
   goes up through ancestors to `<svelte:window onkeydown>`.
5. **No `in:`/`out:`/`transition:` on components.** Wrap the content in
   `<Presence in={[fly, { y: 8 }]} out={fade}>`. `animate:flip` is unavailable.
6. **Import the library first** in the entry file: `import { start } from
   "@neoworks/svelte-gpui"` before `App.svelte`, then `start(App)`.
7. **No DOM or browser storage.** Measure with `onlayout`. Reach a primitive's methods
   with `bind:this`. There is no `document` and no `getBoundingClientRect`. For
   storage, write files with `files`.

## Primitives at a glance

| Component | Key props | Methods |
|---|---|---|
| `View` | `style hover active focus focusable onpress onhover ondown buttons onmove onup onwheel onlayout onresize onkey onfocus onblur ondragenter ondragover ondragleave ondrop` | `focus()` |
| `Text` | `style pre onlayout`; nest `Text` for runs in one paragraph | |
| `Button` | `style hover active focus disabled disabledStyle onpress onhover onkey onfocus onblur` | `focus()` |
| `Input` | `bind:value placeholder multiline rows secure numeric maxLength disabled readonly style hover focus oninput onsubmit oncommit onkey` | `focus()`, `select(start?, end?)` |
| `Scroll` | `style horizontal onscroll onlayout` (needs a bounded height) | `scrollTo(offset)` |
| `List` | `items rowHeight? estimatedRowHeight row={snippet(item, index)} key overscan stickToEnd style` (virtualised; without `rowHeight` rows are measured and may differ) | `scrollToIndex(i, { align })` |
| `Image` | `src` (path, `data:`, `http(s)`) `fit style onload onerror` | |
| `Icon` | `svg` (markup) or `src`, `size color style`; tinted by the text colour | |
| `Canvas` | `style ondown onmove onup onhover onwheel onlayout` | `begin(x,y,pen) lineTo end line fillRect fillCircle clear` |
| `Presence` | `in out style`, each a transition or `[fn, params]` | |
| `Overlay` | `side align offset crossOffset at flip shift margin matchWidth passthrough layer focusable onoutside style` + `View` events; drawn above everything, placed against its parent's box | `focus()` |

`View` is a column. `Button` and `Input` are rows. Text properties (`color`,
`fontSize`, `fontWeight`, `fontFamily`, …) are inherited by children.

## Style keys

These are the only keys; anything else is ignored. See `docs/styles.md` for values.

- **Layout:** `direction wrap flex grow shrink basis align alignSelf justify gap rowGap
  columnGap padding paddingX paddingY padding{Top,Right,Bottom,Left} margin marginX
  marginY margin{Top,Right,Bottom,Left} display`
- **Size and position:** `width height size minWidth minHeight maxWidth maxHeight
  aspectRatio position inset top right bottom left overflow translate`
- **Paint:** `background opacity borderWidth border{Top,Right,Bottom,Left}Width
  borderColor radius shadow cursor transition`
- **Text:** `color fontSize fontWeight fontFamily lineHeight italic underline
  strikethrough textAlign nowrap truncate maxLines uppercase`

The values that trip people up:

- `flex: 1` grows from a basis of 0, as in React Native.
- `justify` takes `"between"` and `"around"`.
- `align` takes `"start"`, `"center"`, `"end"`, `"stretch"` and `"baseline"`.
- `padding` is a number, `[vertical, horizontal]` or `[top, right, bottom, left]`.
- `radius` is a number or four corners.
- `fontFamily` is `"sans"` (Geist), `"mono"` (JetBrains Mono) or an installed font.
- `lineHeight` is in pixels.
- `transition` eases only `background`, `color`, `borderColor` and `opacity`. Animate
  size and position with `Tween` or `Spring` driving `translate`, `width` and so on.
- There is no `zIndex`: later siblings draw on top.
- A flexible child that should truncate needs `minWidth: 0`.

A theme is plain data: exported `Style` objects spread together, or combined with
`merge(a, cond && b)`.

## Network and files

- **`fetch`** is the web's: `text()`, `json()`, `arrayBuffer()`, abort signals. A 4xx or
  5xx resolves; only network failures reject. Bodies are buffered, and there are no
  streams, `Blob`, `FormData` or cookies.
- **`WebSocket`** is the web's, with `binaryType = "arraybuffer"` only.
- **`files`** comes from `@neoworks/svelte-gpui`. It has:
  - `readText`/`readBytes`, `writeText`/`writeBytes`/`appendText`;
  - `exists`, `stat`, `list`, `mkdir`, `remove`, `rename`, `copy`;
  - `watch(path, cb)`, which returns a stop function to call from the `$effect` cleanup;
  - `paths()`, and the dialogs `pickFiles`/`pickSavePath`.
  - Errors are `FileError` with a `code` such as `"NotFound"` or `"NotAllowed"`.
- **Build absolute paths from `await files.paths()`** (`data`, `config`, `cache`, …),
  never from relative paths.
- **File access is unlimited unless the app declares a limit:**
  `start(App, {}, { files: { allow: ["$data/my-app"] } })`. Dialog picks are always
  allowed. The `--allow-files` host flag narrows it further.

## Rust commands and packaging

- **Rust code of the app's own** is called with `invoke` and `listen` from
  `@neoworks/svelte-gpui`; the app is then built from its own crate (`docs/rust.md`,
  `examples/rust-commands`).
  - `await invoke<T>(name, args?, bytes?)` runs a command registered in Rust with
    `Commands::new().command(name, |args| ...)`, on a thread. A failure rejects with an
    `Error` that has `message` and, if given, `code` (`UnknownCommand` for a bad name).
  - `invokeBytes` also returns the bytes the command sent back: `{ value, bytes }`.
  - `const stop = listen<T>(name, (payload, bytes?) => ...)` hears what Rust
    `Emitter::emit`s. Call `stop()` from the `$effect` cleanup.
  - Declare commands once for type checking: `declare module "@neoworks/svelte-gpui" {
    interface CommandMap { greet: { args: { name: string }; result: string } } }`.
  - Commands never touch the UI; they answer, or emit events.
- **Packaging** (`docs/packaging.md`): `bun scripts/package.mjs --app examples/demo` makes
  one executable with the bundle (as bytecode) and its files embedded, plus a `.desktop`
  file and an icon, in `dist/package/<app>/`. The app's metadata (`name`, `id`, `bundle`,
  `icon`, ...) is the `"svelte-gpui"` key in its `package.json`. Embedded files are
  unpacked to `~/.cache/<id>/`, which is `files.paths().bundle`. Linux only.

## Patterns

- **Controls** such as checkboxes, switches, sliders and tabs are Svelte components
  built from `Button`/`View`. `examples/demo/src/components` has working ones.
- **Drag or slider:** set a flag in `ondown`, update in `onmove`, clear it in `onup`.
  The pointer is captured, so moves keep arriving outside the element. Clamp `e.x`.
- **Many marks** (drawings, plots): use one `Canvas`, never a `View` per mark.
- **Long lists:** `List`. Give `rowHeight` for one height; leave it out for rows that differ (chat,
  wrapped text): they are measured as drawn, the window keeps its place, `stickToEnd` follows a chat.
  Give `key` when items insert at the start or reorder. A tree is `Tree`, built on it.
- **Modal dialogs:** the last child of the root, with `position: "absolute", inset: 0`.
- **Anything that floats next to an element:** `Popover` (trigger snippet + content, `bind:open`),
  `Tooltip`, `Menu` in a `Popover` with `focusPanel={false}`, `ContextMenu`, `Select`. See below.
- **Dragging:** files from other programs are `ondragenter`/`ondragleave`/`ondrop` on a `View`
  (`e.paths`, readable once dropped). Inside the app, `Draggable` and `DropTarget`.
- **Shortcuts:** `<svelte:window onkeydown>` with `event.ctrlKey` and similar. Call
  `preventDefault()` so a focused field does not type the key too.
- **Shared state:** a `.svelte.ts` module exporting `$state`.

## Floating content, dragging and trees

`import { Popover, Tooltip, Menu, ContextMenu, Select, Overlay, Draggable, DropTarget, Tree } from "@neoworks/svelte-gpui/ui"`.

- **`Overlay`** is the primitive (`g-overlay`): it is drawn above everything, takes no room where it
  sits, is never clipped by ancestors, and is placed after layout against its **parent's** box
  (`side`, `align`, `offset`), flipped and shifted to stay in the window, or at a point with
  `at={[x, y]}`. Put it inside the element it belongs to. An overlay inside another is drawn in
  place, so no `overflow: "hidden"` around it.
- **`Popover`** wraps a trigger and an overlay: `<Popover bind:open>{#snippet trigger({ toggle })}…{/snippet}
  {#snippet children({ close })}…{/snippet}</Popover>`. An outside press and Escape close it; focus
  moves in and returns. A press on the trigger is not "outside".
- **A right press** is `ondown` with `event.button === 2`; `ContextMenu` does it. `buttons={["left"]}` on a view makes its `ondown` take only the left button and lets the others through to what is around it; `Draggable` does this.
- **`Draggable data={x}`** inside a **`DropTarget ondragover={(info) => …} ondrop={({ data }) => …}`**;
  `info` has `x y width height` inside the target, for before/after decisions.
- **`Tree`** takes `items`, `key`, `childrenOf` (called for expanded branches only, so make or load children
  lazily) and a `row` snippet; `bind:expanded`, `bind:selected`, `onexpand`, `onactivate`.
- **A bare `x;` or `void x;` is deleted by the build** (esbuild drops what looks pure before Svelte sees
  it), so it cannot be used to make `$derived` depend on a state variable. Read it in an expression.

## Build, run, verify

```
bun examples/demo/build.mjs                        # or the app's own build.mjs, which calls build() from "@neoworks/svelte-gpui/build"
cargo build --release --manifest-path host/Cargo.toml
svelte-gpui --bundle dist/app.js --width 900 --height 600
bun run typecheck                                  # tsc + svelte-check: catches bad style keys, props and handlers
```

Inside the svelte-gpui repo, the host build shares GPUI artifacts through
`CARGO_TARGET_DIR`; `CLAUDE.md` there has the path.

**Develop with hot reload.** Run `bun build.mjs --watch` (the build passes `watch: true` to
`build()`) and `svelte-gpui --bundle dist/app.js --watch`. Saving a source file reloads the app
in the same window within about a quarter of a second, and its state starts over. `--watch`
implies `--dev`: build failures, exceptions in handlers and effects, and unhandled rejections
show in an overlay, with stacks mapped to source files through a source map. Esc dismisses it
and the next good build clears it. Without `--dev`, errors go to
`~/.cache/svelte-gpui/<app>/errors.log` and nothing is drawn. Pass `--dev` when testing
with `scripts/shot.sh`, so an error shows up in the screenshot.

## The window

`import { window } from "@neoworks/svelte-gpui"` (`docs/window.md`).

- **Open it:** `start(App, {}, { window: { title, width, height, minWidth, minHeight } })`.
  `--width` and `--height` override the size.
- **Read it,** reactively: `window.size` (`{ width, height }`), `scale`, `focused`,
  `maximized`, `fullscreen`.
- **Ask things of it:** `setTitle`, `minimize`, `maximize`, `unmaximize`, `toggleMaximize`,
  `setFullscreen(on)`, `toggleFullscreen`, `close()` (asks the handlers) and `destroy()`.
- **Confirm before closing:** `$effect(() => window.onCloseRequested(() => dirty ? ask() :
  undefined))`. Return `false` or a promise of `false` to keep the window open. The
  `$effect` returns the unsubscribe function.
- **Not there:** position, `center`, `setSize` and `setMinSize` after opening, and
  `resizable`. GPUI 0.2.2 on Wayland cannot do them; do not write them.

Always check a UI change on screen, not just in the types. Use `scripts/shot.sh`, which
opens the window on a headless output without taking focus, and screenshots it:

```
scripts/shot.sh BUNDLE /tmp/out.png SECONDS --do "press:Save" --do "type:@placeholder=Name:Ada"
```

Then read the PNG. Steps run 0.9 s apart, so make `SECONDS` at least one per step plus
one. `docs/testing.md` lists the steps:

- `press`, `hover`, `focus`, `type`, `key`, `compose`, `commit`;
- `down`/`move`/`up` with window `x,y` (a third number is the button: `down:T:10,20,2` is a right press);
- `drop:TARGET:/a/file,/b` and `dragover:…` for files dragged in;
- `scroll`, `wait`, `frames`.

Targets are text, `@attr=value` or `@tag=g-canvas`. The host log is at
`/tmp/svelte-gpui-host.log`, and `console.log` from the app lands there.
`nothing matches "…"` in the log means a step's target was wrong.

## When something does not show

- Is it an HTML element or a CSS class? Replace it with a primitive and a style object.
- Is a `Scroll` or `List` missing a bounded height (`flex: 1` or `height`)?
- Is a flex child collapsed? Give it a size, or `flex: 1` in a parent that has room.
- Is text cut off or overflowing? Add `minWidth: 0` on the flexible parent, and
  `truncate` or `maxLines` on the text.
- Did a transition run on a component directly? Use `Presence`.
- Does a style key do nothing? Check the spelling against `runtime/ui/style.ts`. The
  host ignores unknown keys.
