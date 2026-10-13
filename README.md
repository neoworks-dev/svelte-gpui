# svelte-gpui

Svelte 5 components drawn by GPUI. No webview, no browser DOM.

```
bun install
cd examples/hello && bun build.mjs                 # bundle the app, compile its Tailwind CSS
cd ../../host && cargo build --release
./target/release/svelte-gpui --bundle ../examples/hello/dist/hello.js --css ../examples/hello/dist/hello.css
```

## How it works

Svelte has no renderer API, but the compiled output only ever touches a small
DOM surface: elements, text nodes, comment anchors, attributes, listeners. The
approach is the one `svelte-gtk4` takes, with a different back end.

- `runtime/dom/` is a node tree with that surface (`nodes.ts`), a parser for the
  template HTML the compiler emits (`parse.ts`), and the globals Svelte reads
  at import time plus a clock (`globals.ts`). Adapted from svelte-gtk4.
- `runtime/host/bridge.ts` watches that tree and records what changes as small
  operations: create, set attribute, insert, remove, set text, set style, listen,
  drop. They are sent to the host as one batch when Svelte has finished a flush.
- `runtime/host/events.ts` turns a click the host reports into a DOM-shaped
  event and walks it up the tree, so Svelte's delegated handlers run unchanged.
- `host/` is Rust. `runtime.rs` runs the bundle in QuickJS, `tree.rs` keeps a
  copy of the document from the operations, `css.rs` and `style.rs` resolve
  classes against a real Tailwind stylesheet, and `render.rs` draws the tree
  with GPUI.

## Styling

Elements are ordinary HTML tags with ordinary classes. The build runs the real
Tailwind compiler. The host parses the stylesheet it writes, matches each
element's classes to rules, evaluates `var()`, `calc()` and `color-mix()`, and
turns the declarations into GPUI styles. `:hover` and `group-hover:` become
GPUI hover refinements. Inline `style:` directives go through the same path.

Declarations and selectors the host cannot honour are counted, and `--report`
prints them, so coverage against a real stylesheet is measurable.

## Measured, 2026-10-14, Linux, Wayland

The hello example: a counter, a conditional, a keyed list, and a class directive.

| | |
|---|---|
| Mount, including Svelte's runtime | 7 ms, 46 nodes |
| Click that changes a counter | 1 operation, 0.11 ms |
| Click that hides a list of 3 rows | 15 operations, 0.25 ms |
| Building GPUI elements for a frame | 0.13 ms |
| Memory with the window open | 192 MB PSS, 233 MB RSS |

Svelte's fine-grained updates survive the bridge: the counter click sends the
one text node that changed, not a new tree.

## Not done

- No `svg`, `img`, `input`, `textarea` or `select`. Phosphor icons are `svg`, so
  most of Grove's components need `svg` first.
- Inline flow is approximated: runs of text and inline elements share a wrapping
  row, but there is no real inline layout across elements.
- GPUI draws one border colour per element, so a side with its own colour is a
  strip drawn on top.
- Nothing has been run against Grove's own components or stylesheet yet.
- Clicks only. No hover events to Svelte, no keyboard, no focus, no scrolling
  events, no transitions.
