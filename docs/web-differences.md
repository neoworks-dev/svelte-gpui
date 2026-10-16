# Coming from Svelte on the web

svelte-gpui runs the real Svelte 5 compiler and runtime. Components, runes and template
syntax behave as you know them. The difference is underneath: there is no browser, so
nothing the browser provides exists.

## The same as the web

- Runes: `$state`, `$derived`, `$effect`, `$props`, `$bindable`, `$inspect`.
- Template syntax: `{#if}`, `{#each}` with keys, `{#await}`, `{#key}`, snippets and
  `{@render}`, `{@const}`.
- Components, props, `bind:` on component props, and `bind:this` for a primitive's
  methods.
- `.svelte.ts` modules with runes, for shared state.
- `svelte/motion` (`Tween`, `Spring`), `svelte/easing`, `svelte/transition` through
  `Presence`, `svelte/reactivity`, and stores.
- `setTimeout`, `setInterval`, `requestAnimationFrame`, `performance.now()`,
  `queueMicrotask`, promises and `async`/`await`.
- `<svelte:window onkeydown onkeyup onresize>`, `window.innerWidth` and
  `window.innerHeight`.
- `console.log` and friends, which print to the host's standard error.
- `fetch`, `Headers`, `Request`, `Response`, `AbortController`, `WebSocket`,
  `TextEncoder` and `TextDecoder`. See [Network and files](io.md).

## Different

| On the web | Here |
|---|---|
| `<div>`, `<span>`, `<button>`, `<input>`, … | `View`, `Text`, `Button`, `Input`, … HTML elements are not drawn. |
| `class`, CSS files, `<style>` blocks | A `style` object. See [Styles](styles.md). |
| `:hover`, `:active`, `:focus` | The `hover`, `active` and `focus` props. |
| CSS `transition` | `style.transition`, for colours and opacity. |
| `onclick` | `onpress` |
| `oninput`, `bind:value` on `<input>` | `bind:value` or `oninput` on `Input`, which receives the text, not an event. |
| `onkeydown` on an element | `onkey`, which bubbles to the window. |
| `onmousedown` / `onpointermove` / `onpointerup` with `setPointerCapture` | `ondown`, `onmove`, `onup`: capture is automatic. |
| Events bubble, and `event.target` matters | Nothing bubbles except keys. Each handler hears only its own element. |
| `in:fade` on an element | `<Presence in={fade}>` |
| `animate:flip` | Not available. |
| `<img>` | `Image`, with `fit`. |
| Inline `<svg>` | `Icon` with SVG markup. |
| `<canvas>` and its 2D context | `Canvas`, with strokes, lines, rectangles and circles. |
| Overflow scrolls with `overflow: auto` | `Scroll`, or `List` for long lists. |
| `z-index` | Later siblings draw on top. Things that must be above everything are `Overlay`s (`Popover`, `Tooltip`, `Menu`). |
| `ondragenter` / `ondrop` with `dataTransfer.files` | The same names on `View`, with `e.paths`, absolute paths the app may read. Dragging within the app is `Draggable` and `DropTarget`. |
| `oncontextmenu` | `ondown` with `event.button === 2`, or `ContextMenu`. |
| `em`, `rem`, `vh`, `calc()` | Pixels and percentages. Compute anything else in script. |
| Components nest as deep as you like | About 100 levels on Linux with an 8 MB stack (`ulimit -s`), and about 15 on Windows. Each level costs stack while Svelte mounts it, and past the limit the app stops with `RangeError: Maximum call stack size exceeded`. Draw deep trees, such as parsed markdown, from flat rows. |

## One thing that bites

- **A bare `x;` or `void x;` is deleted by the build.** The TypeScript step runs before Svelte compiles
  `$state`, and drops an expression statement that looks pure, so it cannot be used to make a `$derived`
  or `$effect` depend on a variable. Use the variable in an expression the code needs.

## Not available

- **The DOM.** `document.querySelector`, `element.style`, `getBoundingClientRect` and
  `ResizeObserver` do not exist. Use `bind:this` and a primitive's methods, and
  `onlayout` for measurements.
- **Browser storage.** There is no `localStorage` or `IndexedDB`. Write files with
  `files` instead; see [Network and files](io.md). `fetch` and `WebSocket` are there.
- **`Blob`, `FormData`, streams and cookies** in `fetch`.
- **Web components and libraries that render HTML.** A UI library built on HTML
  elements draws nothing. A library that only computes, such as date handling,
  parsing or state, works if it does not touch browser globals.
- **Tab navigation between elements, and accessibility** (screen readers). These are
  not built yet.
- **Text selection outside fields.** `Text` cannot be selected or copied.
- **Multiple windows and native menus.** The host opens one window.

## Why primitives and style objects

A browser engine is what makes a web page heavy. svelte-gpui draws with GPUI, which
has no CSS engine, so the useful subset is exposed directly: flexbox, paint and text.
Only what the host can draw can be expressed, and the types say what that is. A
misspelt property is a type error rather than a silent no-op, and the host never
matches selectors.
