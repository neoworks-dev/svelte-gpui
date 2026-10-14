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

## Running Grove's own components

`examples/grove-row` builds Grove's `GithubItemRow.svelte`, unmodified, with the
Grove checkout named by `GROVE` (default `../grove`). Its imports resolve from
Grove's own `node_modules`, so `@neoworks-dev/ui`'s `Checkbox` and Phosphor's icons
come along. Only `store.svelte`, which talks to the main process, is swapped for a
stub, in the example's `build.mjs`.

The stylesheet is what Grove's `main.css` compiles to, run from Grove's root so
Tailwind scans Grove's sources:

```
cd ~/Documents/neoworks/grove
~/Documents/neoworks/svelte-gpui/node_modules/.bin/tailwindcss \
  -i src/renderer/src/assets/main.css -o ~/Documents/neoworks/svelte-gpui/examples/grove-row/dist/grove.css
```

`--report` says how much of that stylesheet the page needed and the host could not
give it: the classes in use with no rule, and the classes whose only rules use a
selector the engine leaves out. For the row that is one class, `peer`, which only
marks a sibling and has no rule. Four checks on screen: the rows match the
Electron pane, the label pills and avatars render, `checked:bg-action` follows the
checkbox, and a real Space on a focused checkbox ran click, `input` and `change`,
Svelte's `bind:checked`, and the component's own `{#if checked}` check icon.

## Grove's whole GitHub pane

`examples/grove-pane` builds `GithubPane.svelte` with its real store, `store.svelte.ts`,
and everything that store imports: the app store, layout, dialogs, markdown and the
rest, 1.2 MB of bundle. What it cannot have is the main process, so
`src/fake.ts` stands in for `window.workbench`: the GitHub calls the pane makes on
load are answered from `fixtures.json`, which `node fixtures.mjs` writes from the
real `neoworks-dev/grove` issues and pull requests with `gh` (reading only), and
every other call answers with an empty promise.

It runs: the tabs with their counts, the search box, the filter bar, 42 issues, a
selected thread with its labels, assignees, participants and actions, and the
comment box. Comment bodies are Grove's own markdown, rendered by `marked`, and
drawn with the typography plugin's descendant selectors.

What that took, beyond the earlier examples:

- A web platform layer for a bare engine (`runtime/dom/polyfills.ts`): `URL`,
  `URLSearchParams`, `TextEncoder`, `atob`, `structuredClone`, `AbortController`,
  `EventTarget` and a few more. Grove's code reaches for them at import time.
- DOM members Svelte and the UI library use: `children`, `closest`, `matches`,
  `compareDocumentPosition`, `scrollTo`, `Event` and `CustomEvent`.
- `{@html}`: markup parsed into nodes that were never announced is announced when
  it is inserted.
- A real selector engine (`host/src/selector.rs`) for the compiled stylesheet:
  descendant and child combinators, `:where()`, `:is()` and `:not()`, attribute
  tests, first and last child, and nested rules, which Tailwind 4 writes for its
  typography plugin. Before, nested declarations leaked onto their parent.
- Inline text flow: a paragraph of text and inline elements is drawn as one
  wrapping text with a run per element, so `<code>` and `<strong>` sit in the line
  as they do in a browser. Padding and borders on inline elements are dropped.

`--report` over the pane: three classes with no rule, which are the markers `group`
and `peer` and `-space-x-1`, and three that only a `::placeholder` rule styles.

## Cost, and what brought it down

Measured on the GitHub pane with a thread open, 2,650 nodes, 2026-10-14, one machine.

| | Before | After |
|---|---|---|
| Building a frame's elements | 12 ms average, 26 ms worst | 3 to 4 ms average, 14 ms worst, the first frame after a change |
| Frames with every node rebuilt each time | not measured | 60 fps held: 16.7 ms mean, 18.6 ms worst (200 frames) |
| Memory with a thread open | 378 MB PSS | 270 MB PSS |
| Memory at start, 171 nodes | 205 MB PSS | 205 MB PSS |

What did it:

- **A style cache.** A node's style was worked out several times per frame, by the
  checks that decide how its parent lays it out. It is now worked out once per
  version of the tree. The work itself was never the cost: all 4,000 styles of a run
  take 37 ms.
- **Images that wait and shrink.** The thread holds ten 1439 by 899 screenshots,
  decoded to 5 MB each and uploaded to the GPU. An image now loads only when it is
  within a screen of the window, and is decoded no wider than it is shown, on a
  thread. That was 120 MB of the 378. Until it loads the box has the image's own
  proportions, read from its header, so the page does not jump.

What is left, and where it sits at start: GPUI loads the system fonts (43 MB of
heap), the GPU driver maps about 65 MB of libraries and 35 MB of its own, and
Vulkan loads every driver it finds, here both NVIDIA's and Mesa's, which costs 20 MB
that setting `VK_DRIVER_FILES` to one driver gives back. The page's own state, the
QuickJS heap and the host tree, is under 25 MB at 170 nodes.

Tried and removed: drawing far-off elements as empty boxes. It would cut the build
cost further for very long pages, but the boxes did not keep the size of what they
replaced, and elements near the edge flipped between the two every frame.

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

## Input methods

A focused field registers GPUI's `EntityInputHandler` while it paints, so the
platform's input method types into it: committed text, composed text that is shown
underlined until it is committed, and the preedit of a dead key. Typed characters
arrive that way too, and a key the host or the app already used (Enter, arrows,
Backspace, Tab, anything the app cancelled) stops there so it is not typed twice.
The app sees `compositionstart`, `compositionupdate` and `compositionend`, and an
`input` event for each change. `--do compose:TARGET:text` and `commit:TARGET:text`
call the handler methods the platform calls.

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

## Running Grove's own components

`examples/grove-row` builds Grove's `GithubItemRow.svelte`, unmodified, with the
Grove checkout named by `GROVE` (default `../grove`). Its imports resolve from
Grove's own `node_modules`, so `@neoworks-dev/ui`'s `Checkbox` and Phosphor's icons
come along. Only `store.svelte`, which talks to the main process, is swapped for a
stub, in the example's `build.mjs`.

The stylesheet is what Grove's `main.css` compiles to, run from Grove's root so
Tailwind scans Grove's sources:

```
cd ~/Documents/neoworks/grove
~/Documents/neoworks/svelte-gpui/node_modules/.bin/tailwindcss \
  -i src/renderer/src/assets/main.css -o ~/Documents/neoworks/svelte-gpui/examples/grove-row/dist/grove.css
```

`--report` says how much of that stylesheet the page needed and the host could not
give it: the classes in use with no rule, and the classes whose only rules use a
selector the engine leaves out. For the row that is one class, `peer`, which only
marks a sibling and has no rule. Four checks on screen: the rows match the
Electron pane, the label pills and avatars render, `checked:bg-action` follows the
checkbox, and a real Space on a focused checkbox ran click, `input` and `change`,
Svelte's `bind:checked`, and the component's own `{#if checked}` check icon.

## Grove's whole GitHub pane

`examples/grove-pane` builds `GithubPane.svelte` with its real store, `store.svelte.ts`,
and everything that store imports: the app store, layout, dialogs, markdown and the
rest, 1.2 MB of bundle. What it cannot have is the main process, so
`src/fake.ts` stands in for `window.workbench`: the GitHub calls the pane makes on
load are answered from `fixtures.json`, which `node fixtures.mjs` writes from the
real `neoworks-dev/grove` issues and pull requests with `gh` (reading only), and
every other call answers with an empty promise.

It runs: the tabs with their counts, the search box, the filter bar, 42 issues, a
selected thread with its labels, assignees, participants and actions, and the
comment box. Comment bodies are Grove's own markdown, rendered by `marked`, and
drawn with the typography plugin's descendant selectors.

What that took, beyond the earlier examples:

- A web platform layer for a bare engine (`runtime/dom/polyfills.ts`): `URL`,
  `URLSearchParams`, `TextEncoder`, `atob`, `structuredClone`, `AbortController`,
  `EventTarget` and a few more. Grove's code reaches for them at import time.
- DOM members Svelte and the UI library use: `children`, `closest`, `matches`,
  `compareDocumentPosition`, `scrollTo`, `Event` and `CustomEvent`.
- `{@html}`: markup parsed into nodes that were never announced is announced when
  it is inserted.
- A real selector engine (`host/src/selector.rs`) for the compiled stylesheet:
  descendant and child combinators, `:where()`, `:is()` and `:not()`, attribute
  tests, first and last child, and nested rules, which Tailwind 4 writes for its
  typography plugin. Before, nested declarations leaked onto their parent.
- Inline text flow: a paragraph of text and inline elements is drawn as one
  wrapping text with a run per element, so `<code>` and `<strong>` sit in the line
  as they do in a browser. Padding and borders on inline elements are dropped.

`--report` over the pane: three classes with no rule, which are the markers `group`
and `peer` and `-space-x-1`, and three that only a `::placeholder` rule styles.

## Cost, and what brought it down

Measured on the GitHub pane with a thread open, 2,650 nodes, 2026-10-14, one machine.

| | Before | After |
|---|---|---|
| Building a frame's elements | 12 ms average, 26 ms worst | 3 to 4 ms average, 14 ms worst, the first frame after a change |
| Frames with every node rebuilt each time | not measured | 60 fps held: 16.7 ms mean, 18.6 ms worst (200 frames) |
| Memory with a thread open | 378 MB PSS | 270 MB PSS |
| Memory at start, 171 nodes | 205 MB PSS | 205 MB PSS |

What did it:

- **A style cache.** A node's style was worked out several times per frame, by the
  checks that decide how its parent lays it out. It is now worked out once per
  version of the tree. The work itself was never the cost: all 4,000 styles of a run
  take 37 ms.
- **Images that wait and shrink.** The thread holds ten 1439 by 899 screenshots,
  decoded to 5 MB each and uploaded to the GPU. An image now loads only when it is
  within a screen of the window, and is decoded no wider than it is shown, on a
  thread. That was 120 MB of the 378. Until it loads the box has the image's own
  proportions, read from its header, so the page does not jump.

What is left, and where it sits at start: GPUI loads the system fonts (43 MB of
heap), the GPU driver maps about 65 MB of libraries and 35 MB of its own, and
Vulkan loads every driver it finds, here both NVIDIA's and Mesa's, which costs 20 MB
that setting `VK_DRIVER_FILES` to one driver gives back. The page's own state, the
QuickJS heap and the host tree, is under 25 MB at 170 nodes.

Tried and removed: drawing far-off elements as empty boxes. It would cut the build
cost further for very long pages, but the boxes did not keep the size of what they
replaced, and elements near the edge flipped between the two every frame.

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

- No undo and no spellcheck in text fields. Composition was checked through the
  input handler, not with a real IME or real dead keys: the keys sent by
  `hyprctl` did not go through the compose table.
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
