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

## Events

An element only gets a GPUI handler for an event type that something listens
for. Svelte delegates `click`, `mousedown`, `keydown` and friends to one
listener on the mount root, so the host learns they matter from the root's
listener list, and every element reports them: the deepest one under the pointer
sends the event and stops it, and the app bubbles it from there. Events Svelte
does not delegate (`mouseenter`, `scroll`, `wheel`, `focus`, `load`) go only to
the element that asked, without bubbling.

| Event | Source in the host |
|---|---|
| `click`, `dblclick` | GPUI click, including Enter and Space on a focused button |
| `mousedown`, `mouseup`, `pointerdown`, `pointerup`, `contextmenu` | GPUI mouse buttons |
| `mouseenter`, `mouseleave`, `mouseover`, `mouseout` and the pointer spellings | GPUI hover |
| `mousemove`, `pointermove` | GPUI mouse move, only where listened for |
| `wheel` | GPUI scroll wheel, only on elements that listen, so scrolling still works |
| `scroll` | the scroll position of `overflow: auto` boxes, checked every tick |
| `keydown`, `keyup` | the focused element, then up through `document` and `window` |
| `focus`, `blur`, `focusin`, `focusout` | focus handles on `tabindex`, buttons, links and inputs |
| `load`, `error` | an `img` whose source settled |
| `resize` | the window |

`bind:clientWidth`, `ResizeObserver` and `svelte:window bind:innerWidth` work: the
host measures observed elements after layout and reports changes. Assigning
`scrollTop` or calling `focus()` in the app reaches the host as a command.

## svg and img

An `svg` is serialised from its subtree, registered as an asset, and drawn by
GPUI tinted with the inherited text colour, so Phosphor's components work as
they are and `text-green` colours them. An `img` takes a file path, a `data:`
URL or an `http(s)` URL; remote ones are fetched on a thread into
`~/.cache/svelte-gpui/images`, and `load` or `error` fires when they settle.
`rounded-full` clips images, and `object-cover` sets the fit.

## Testing without a pointer

`--do click:TEXT`, `--do focus:TEXT` and `--do scroll:TEXT:PIXELS` act on the
deepest element containing that text. Focus and scroll go through the same
command path an app's `focus()` and `scrollTop` use, and real keys can be sent
to the window with `hyprctl dispatch hl.dsp.send_shortcut`. In the events example
a real `z` reached the focused element's handler and `svelte:window`'s, a
scroll reached `onscroll`, and `bind:clientWidth` reported 642 for a 668 pixel
window. Hover, mouse buttons and clicks were seen in the logs as real events,
because the pointer strayed onto the test display; the wheel was not exercised.

## Not done

- No `input`, `textarea` or `select`, so no `input`, `change` or `beforeinput`.
  Text entry needs an editable element with IME handling.
- No touch, drag, clipboard or `animationend` events.
- Svelte's `transition:` directives need `element.animate()`, which is not
  shimmed, and CSS transitions are ignored, so changes snap.
- Inline flow is approximated: runs of text and inline elements share a wrapping
  row, but there is no real inline layout across elements.
- GPUI draws one border colour per element, so a side with its own colour is a
  strip drawn on top.
- Nothing has been run against Grove's own components or stylesheet yet.
