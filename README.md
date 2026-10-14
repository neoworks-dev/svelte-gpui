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

## Forms

`input` (text, search, email, password, number), `textarea`, `checkbox`, `radio`
and `select` work with `bind:value`, `bind:checked`, `bind:group` and
`onchange`. The host owns what is typed: `host/src/forms.rs` keeps each field's
text, cursor and selection, and a key changes it and sends an `input` event
carrying the new value, which the app writes back as the `value` attribute.
The app setting `value` replaces the text.

Fields take typing, Backspace and Delete (with Ctrl for words), arrows, Home and
End, Shift to select, Ctrl+A, up and down in a textarea, `maxlength`, number
filtering, `placeholder`, `disabled` and `readonly`. The mouse places the cursor
and drags a selection. A single-line field scrolls to keep its cursor in view.
`change` fires on blur and on Enter. A `label` activates the control it names or
holds. A `select` shows its chosen option and a caret, opens a list with option
groups and disabled options, and steps with the arrow keys.

`copy`, `cut` and `paste` reach the app first, with a `clipboardData` object, and
a handler that calls `preventDefault` takes over; otherwise the field copies,
cuts or pastes itself. `navigator.clipboard.writeText` and `readText` work.
Wayland only accepts a clipboard write that follows a real input event, so the
host writes in the same turn as the event that asked.

## Transitions

Svelte's `transition:`, `in:` and `out:` call `element.animate()` with a list of
keyframes sampled at 60 Hz. `runtime/dom/animate.ts` plays them against the
host's clock by writing the inline style each keyframe names, and fires
`onfinish`, so elements that are leaving stay until the outro ends. `fade` and
`fly` were checked on screen. `transform: translate()` becomes a relative offset,
since a div cannot be transformed.

CSS transitions work for colours and opacity: `transition-colors`,
`transition-opacity`, `duration-*` and `ease-*` come from the real stylesheet,
and a change of class, or of hover, eases from what was last drawn. Elements that
ease take hover from the host rather than GPUI, which is why hover is tracked.

## Testing without a pointer

`--do` steps act on the deepest element containing some text, or on an attribute
with `@name=value`: `click:`, `focus:`, `type:TARGET:text`, `key:TARGET:ctrl-a`,
`open:` and `choose:TARGET:index` for a select, `scroll:TEXT:PIXELS`, and `wait:`.
Typing and keys go through the same functions real keys do, and real keys can be
sent with `hyprctl dispatch hl.dsp.send_shortcut`. A real `ctrl-x` and `ctrl-v`
moved text through the system clipboard and fired `cut` and `paste`, a real Enter
on a focused button ran `navigator.clipboard.writeText`, and real arrow keys
stepped a select. Mouse placement of a cursor, dragging a selection, and hover
transitions were not exercised with a pointer.

## Not done

- No IME or dead-key composition in text fields, no undo, and no spellcheck.
- `input type` of date, color, range and file; no `form` submission.
- Svelte's `slide` has nothing to measure, since layout only comes back for
  elements that asked, so it does not animate height; `scale` and `blur` only
  change opacity; `animate:flip` has no positions.
- No touch, drag or `animationend` events.
- Inline flow is approximated: runs of text and inline elements share a wrapping
  row, but there is no real inline layout across elements.
- GPUI draws one border colour per element, so a side with its own colour is a
  strip drawn on top.
- Nothing has been run against Grove's own components or stylesheet yet.
