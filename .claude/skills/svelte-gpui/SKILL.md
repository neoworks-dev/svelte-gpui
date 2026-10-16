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
   `Button`, `Input`, `Scroll`, `List`, `Image`, `Icon`, `Canvas`, `Presence`. Never
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
| `View` | `style hover active focus focusable onpress onhover ondown onmove onup onwheel onlayout onkey onfocus onblur` | |
| `Text` | `style pre onlayout`; nest `Text` for runs in one paragraph | |
| `Button` | `style hover active focus disabled disabledStyle onpress onhover onkey onfocus onblur` | `focus()` |
| `Input` | `bind:value placeholder multiline rows secure numeric maxLength disabled readonly style hover focus oninput onsubmit oncommit onkey` | `focus()`, `select(start?, end?)` |
| `Scroll` | `style horizontal onscroll onlayout` (needs a bounded height) | `scrollTo(offset)` |
| `List` | `items rowHeight row={snippet(item, index)} key overscan style` (fixed-height rows, virtualised) | `scrollToIndex(i)` |
| `Image` | `src` (path, `data:`, `http(s)`) `fit style onload onerror` | |
| `Icon` | `svg` (markup) or `src`, `size color style`; tinted by the text colour | |
| `Canvas` | `style ondown onmove onup onhover onwheel onlayout` | `begin(x,y,pen) lineTo end line fillRect fillCircle clear` |
| `Presence` | `in out style`, each a transition or `[fn, params]` | |

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

## Patterns

- **Controls** such as checkboxes, switches, sliders and tabs are Svelte components
  built from `Button`/`View`. `examples/demo/src/components` has working ones.
- **Drag or slider:** set a flag in `ondown`, update in `onmove`, clear it in `onup`.
  The pointer is captured, so moves keep arriving outside the element. Clamp `e.x`.
- **Many marks** (drawings, plots): use one `Canvas`, never a `View` per mark.
- **Long uniform lists:** `List`. For rows of varying height, `Scroll` with `{#each}`.
- **Overlays and dialogs:** the last child of the root, with
  `position: "absolute", inset: 0`.
- **Shortcuts:** `<svelte:window onkeydown>` with `event.ctrlKey` and similar. Call
  `preventDefault()` so a focused field does not type the key too.
- **Shared state:** a `.svelte.ts` module exporting `$state`.

## Build, run, verify

```
bun examples/demo/build.mjs                        # or the app's own build.mjs, which calls build() from "@neoworks/svelte-gpui/build"
cargo build --release --manifest-path host/Cargo.toml
svelte-gpui --bundle dist/app.js --width 900 --height 600
bun run typecheck                                  # tsc + svelte-check: catches bad style keys, props and handlers
```

Inside the svelte-gpui repo, the host build shares GPUI artifacts through
`CARGO_TARGET_DIR`; `CLAUDE.md` there has the path.

Always check a UI change on screen, not just in the types. Use the headless runner:

```
scripts/shot.sh BUNDLE /tmp/out.png SECONDS --do "press:Save" --do "type:@placeholder=Name:Ada"
```

Then read the PNG. Steps run 0.9 s apart, so make `SECONDS` at least one per step plus
one. `docs/testing.md` lists the steps:

- `press`, `hover`, `focus`, `type`, `key`, `compose`, `commit`;
- `down`/`move`/`up` with window `x,y`;
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
