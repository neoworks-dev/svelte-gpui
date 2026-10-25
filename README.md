# svelte-gpui

Native apps from Svelte 5 components, drawn by [GPUI](https://www.gpui.rs/). No
webview, no DOM, no CSS: an app is built from a fixed set of primitives, styled with
plain objects, the way React Native apps are.

```svelte
<script lang="ts">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  let count = $state(0);
</script>

<View style={{ padding: 24, gap: 12 }}>
  <Text style={{ fontSize: 24, fontWeight: "bold" }}>Hello <Text style={{ color: "#a78bfa" }}>GPUI</Text></Text>
  <Button
    style={{ paddingX: 12, paddingY: 6, radius: 6, background: "#3f3f46", transition: 150 }}
    hover={{ background: "#52525b" }}
    onpress={() => count++}
  >
    <Text>Pressed {count} times</Text>
  </Button>
</View>
```

**Documentation:** [docs/](docs/README.md) covers getting started, every primitive,
every style property, events, motion, patterns, and what differs from Svelte on the
web. AI agents can load the `svelte-gpui` skill in
[`.claude/skills/svelte-gpui`](.claude/skills/svelte-gpui/SKILL.md).

## Running the demo

`examples/demo` shows everything there is: layout, text, fields and controls, a list of
100,000 rows, drawing with the pointer, icons and images, and motion.

```
bun install
bun examples/demo/build.mjs
scripts/host.sh
target/release/svelte-gpui --bundle examples/demo/dist/demo.js --width 1100 --height 760
```

The demo sets its own title and minimum size, and asks before closing while the note in its
Files section has unsaved changes (`docs/window.md`). `--width` and `--height` override the
size it asks for.

To edit it with hot reload, run a build that watches and a host that reloads, in two
terminals, or in one with `--host`:

```
bun examples/demo/build.mjs --watch
target/release/svelte-gpui --bundle examples/demo/dist/demo.js --watch

bun examples/demo/build.mjs --watch --host target/release/svelte-gpui
```

Saving a `.svelte` file updates the window in about a quarter of a second, keeping its
size, position and focus. A build that fails, or an app that throws, shows the error over
the window, with its stack mapped to your source files, until the next good build
(`docs/getting-started.md`).

`examples/stick-to-end` is a chat of three thousand messages of unknown height in a `List` with `stickToEnd`,
opened at its end, with buttons that grow the last row and a row above the window (`Scroll`'s `anchor`).

```
bun examples/stick-to-end/build.mjs
target/release/svelte-gpui --bundle examples/stick-to-end/dist/stick-to-end.js --width 760 --height 640
```

`examples/rust-commands` is an app with its own Rust crate: a UI that calls Rust commands,
listens to events a Rust thread emits, and shows a view that Rust draws with GPUI (`docs/rust.md`).

```
bun examples/rust-commands/build.mjs
cargo run --release --manifest-path examples/rust-commands/Cargo.toml -- --bundle examples/rust-commands/dist/rust-commands.js
```

## Packaging

`bun scripts/package.mjs --app examples/demo` builds one executable with the bundle
(precompiled to bytecode) and its images inside, plus a `.desktop` file, an icon and a
tarball, in `dist/package/demo/`. It runs from anywhere with no flags. The app's name,
id, version and icon live in its `package.json`. See [docs/packaging.md](docs/packaging.md).

## How it works

- **Svelte compiles the app as usual**, and `build.mjs` bundles it into one script.
- **QuickJS runs it** inside the host (`host/src/runtime.rs`). Svelte's compiled output
  only touches a small DOM surface (create, insert, remove, set an attribute, listen), so
  `runtime/dom/` provides exactly that over a plain node tree, plus a clock for timers and
  animation frames. The host drives that clock from the display's frame callbacks while
  anything animates, so motion keeps the display's refresh rate, and from timers
  otherwise; an idle window does no work.
- **The bridge** (`runtime/host/bridge.ts`) records each change to that tree as an
  operation and sends them to the host in one batch after Svelte flushes. The operations
  are listed once, in `protocol.json`, which the bridge imports and `host/build.rs` turns
  into Rust constants. A press that changes a counter sends one operation: the new text.
- **The host** (`host/`) keeps a copy of the tree, parses each element's style object
  once per distinct value (`style/`), and draws every primitive as a GPUI element
  (`render/`). Large, stable parts of the tree become cached GPUI views, so a change
  elsewhere does not rebuild them. `workbench/` is the window: input, focus, the clock
  and what it reports back to the app.
- **Network and files** (`fetch`, `WebSocket`, `files`) run on threads in the host
  (`host/src/io/`), and each result reaches the app as soon as it arrives. See
  [docs/io.md](docs/io.md).
- **Your own Rust code** is called with `invoke(name, args)` and pushes events with
  `listen(name, callback)`, over the same channel: an app with its own crate registers
  commands that run on threads. It can also register GPUI views of its own, which the app
  shows with `element(name)`: they take props, emit events, answer methods, take focus and keys,
  and paint inside a box like any element. See [docs/rust.md](docs/rust.md).
- **Input goes back the same way.** The host reports an event by node id, and
  `runtime/host/events.ts` calls that element's listener directly.

## Primitives

From `@neoworks/svelte-gpui/ui`:

| | |
|---|---|
| `View` | A flex container. A column unless `style.direction` is `"row"`. Takes `onpress`, `onhover`, `ondown`/`onmove`/`onup`, `onwheel`, `onlayout`, `onresize`, `onkey`, `focusable`, and `ondragenter`/`ondragover`/`ondragleave`/`ondrop` for files dragged in from other programs. |
| `Text` | A paragraph. Nested `Text` elements are runs inside it, with their own colour, weight, font or background, and the paragraph wraps as one. `pre` keeps whitespace. |
| `Button` | Pressed by the pointer, or by Enter and Space while it has focus. `disabled`. |
| `Input` | A text field the host edits, with input methods, selection, the clipboard, `multiline`, `secure`, `numeric`, `maxLength`, `bind:value`, `oninput`, `onsubmit` and `oncommit`. |
| `Scroll` | Content that scrolls, vertically or `horizontal`, with `onscroll` and `scrollTo()`. `anchor` keeps the end, or what you are looking at, in place while the host lays out. |
| `List` | Only the rows near the window are built, so 100,000 items cost what a screenful does. Rows of one height (`rowHeight`), or of different heights, which it measures as they are drawn while the window keeps its place. `scrollToIndex()`, `stickToEnd`. |
| `Image` | A file path relative to the bundle, a `data:` URL, or an `http(s)` URL fetched on a thread and cached. It loads when it nears the window and is decoded no larger than it is drawn. |
| `Icon` | SVG markup or a file, tinted with `color` or the text colour around it. The build loads `.svg` imports as text, so `import gear from "@phosphor-icons/core/regular/gear.svg"` works. |
| `Canvas` | Pixels to paint into. The host keeps a buffer at the window's scale and rasterises each call with tiny-skia as it arrives: `begin`, `lineTo`, `end`, `line`, `fillRect`, `fillCircle`, `clear`. A stroke costs the same however much is already drawn. Takes the pointer events `View` does. |
| `Presence` | A view that plays Svelte transitions as it comes and goes: `<Presence in={[fly, { x: -20 }]} out={fade}>`. Svelte only allows `in:` and `out:` on elements, so this carries them through components. |
| `Overlay` | Content drawn above everything, placed against its parent's box: a side, an alignment, a gap, flipped and shifted to stay in the window. Or at a point in the window. |

Checkboxes, switches, sliders and segmented controls are not primitives. They are
ordinary Svelte components built from these; the demo has one of each in
`examples/demo/src/components`.

Some come with the library because they need the host's help, or are worth getting right once, and are
imported from the same place:

| | |
|---|---|
| `Popover` | Content that floats next to a trigger, closed by an outside press or Escape, with focus moving in and back. |
| `Tooltip` | A label that appears after the pointer has rested on its content. |
| `Menu`, `ContextMenu` | A list of actions with keyboard control, in a popover or at the pointer on a right press. |
| `Select` | A dropdown. |
| `Draggable`, `DropTarget` | Dragging inside the app: a preview follows the pointer, and targets hear the drag and the drop. |
| `Tree` | Expanding and collapsing rows, keyboard navigation, built on `List`. |

`frameStats()` from `@neoworks/svelte-gpui` gives the frames the window drew in the last
second and how long building them took. GPUI only draws when something changed, so an
idle window reports few frames; the demo shows it in its sidebar.

## Styles

`style` is a typed object (`runtime/ui/style.ts`): flexbox layout, sizes in pixels or
percentages, padding and margin, absolute positioning, background, borders, per-corner
radius, shadows, opacity, a cursor, and text properties that children inherit. `hover`,
`active` and `focus` take the same object and are laid over `style` in those states.
`transition` eases colour and opacity changes, including hover.

A theme is plain data: style objects and colours, spread into each other.

## Events

Event names are chosen so Svelte attaches every handler to its own element, never to a
delegated listener at the root. Nothing bubbles, except `onkey`, which goes up through
the focused element's ancestors and then to `<svelte:window onkeydown>`. Pressing a
`View` that listens for `ondown` holds the pointer: `onmove` and `onup` follow it
wherever it goes until the button is released, which is all a drag or a slider needs.
Pointer events carry positions inside the element and in the window.

## Measured, 2026-10-16, Linux, Wayland

The demo, on one machine:

| | |
|---|---|
| Mount, including Svelte's runtime | 8 ms, 191 nodes |
| Memory, first section | 198 MB PSS, of which QuickJS holds 2 MB |
| Building GPUI elements for a frame | 0.02 to 0.05 ms |
| The list section, 100,000 rows in JS | 679 nodes in the host, 29 MB in QuickJS |
| A canvas taking 6 line segments every 16 ms | 60 fps held for 20 s, 0.01 ms building each frame, 208 MB PSS flat |
| A `requestAnimationFrame` loop moving a box and drawing on a canvas, 144 Hz output | 144 fps |

Most of the memory is GPUI itself: the system fonts it loads, the GPU driver, and
Vulkan's loader.

## Testing without a pointer

`scripts/shot.sh BUNDLE OUT.png [SECONDS] [flags]` runs a bundle on the offscreen display,
a headless sway that draws to no screen, and saves a screenshot. Steps act on the deepest element containing some text, or
on an attribute with `@name=value`:

- `--do press:TEXT` presses what listens for it at or above that element.
- `--do type:TARGET:text` types, and `key:TARGET:ctrl-a` sends a key.
- `--do down:TEXT:x,y`, `move:TEXT:x,y` and `up:TEXT:x,y` drive the pointer, in window
  coordinates.
- `--do scroll:TEXT:PIXELS` scrolls the nearest scrolling ancestor.
- `--do drop:TEXT:/a/file,/another` drops files from another program on what listens for `ondrop`.

`--report` and `--exit-after-ms` print memory, timings and the tree's shape.

## Not done

- `Canvas` draws strokes, lines, rectangles and circles; no paths with curves, text or
  images yet, and no reading pixels back.
- Svelte transitions are drawn for opacity, translation, and the heights, padding and
  margins vertical `slide` animates. `scale` and `blur` do not draw their own
  properties. `animate:flip` has no element positions to work from.
- `fetch` buffers whole bodies: no streams, `Blob` or `FormData`, and no cookies.
- Overlays do not animate in or out, a drag does not scroll what is near the edge, and menus have no submenus.
- Text fields have no undo.
- Only tested on Linux with Wayland. Packaging produces Linux executables only.
