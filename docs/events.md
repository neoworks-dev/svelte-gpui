# Events

Handlers are props named `on…`, as in Svelte 5. The names differ from the web's on
purpose. Svelte attaches some events (`click`, `input`, `keydown`, …) to one listener
at the root and routes them by bubbling. svelte-gpui's names (`press`, `edit`, `key`,
…) avoid those, so each handler sits on its own element.

An element only gets a GPUI handler when the app listens. A `View` without `onpress`
costs nothing for presses, and a missing handler is simply absent. Pass `undefined` to
turn one off.

## Delivery

Events go to the element they happened on. **Nothing bubbles**, so a press on a
`Button` inside a `View` with `onpress` calls only the button's handler. Pressing the
`View`'s own area outside the button calls the view's.

The one exception is the keyboard: `onkey` starts at the focused element and goes up
through its ancestors, then reaches `<svelte:window onkeydown>`. Call
`e.stopPropagation()` to end it, or `e.preventDefault()` to stop the host's own
handling, such as a field typing the key.

## Pointer

| Event | On | Carries |
|---|---|---|
| `onpress` | `View`, `Button` | `PointerEvent` |
| `onhover` | `View`, `Button`, `Canvas` | `boolean`: entered or left |
| `ondown`, `onmove`, `onup` | `View`, `Canvas` | `PointerEvent` |
| `onwheel` | `View`, `Canvas` | `WheelEvent` |

```ts
interface PointerEvent {
  x: number; y: number;              // inside the element, from its top left
  windowX: number; windowY: number;  // inside the window
  button: number;                    // 0 left, 1 middle, 2 right, -1 for plain movement
  clicks: number;                    // 2 for a double click
  ctrl: boolean; shift: boolean; alt: boolean; meta: boolean;
}

interface WheelEvent {
  dx: number; dy: number;            // pixels
  ctrl: boolean; shift: boolean; alt: boolean; meta: boolean;
}
```

### Capture

`ondown` captures the pointer. Until the button is released, `onmove` and `onup` go to
that element wherever the pointer goes, even outside it. This is
all a slider, a drag handle or a drawing needs:

```svelte
<View
  style={{ width: 200, height: 20 }}
  ondown={(e) => { dragging = true; seek(e.x); }}
  onmove={(e) => dragging && seek(e.x)}
  onup={() => (dragging = false)}
/>
```

`x` can be negative or larger than the element while captured. Clamp it.

A double click arrives as a press with `clicks: 2`.

## Keyboard and focus

`Button` and `Input` take focus when pressed, as does a `View` with `focusable`.
`Button.focus()` and `Input.focus()` give focus from code.

| Event | On | Carries |
|---|---|---|
| `onkey` | `View`, `Button`, `Input` | `KeyEvent` |
| `onfocus`, `onblur` | `View`, `Button`, `Input` | nothing |

```ts
interface KeyEvent {
  key: string;      // "Enter", "Escape", "ArrowLeft", "Backspace", "Tab", "F1", " ", or the character
  code: string;
  repeat: boolean;  // held down
  ctrl: boolean; shift: boolean; alt: boolean; meta: boolean;
  preventDefault(): void;
  stopPropagation(): void;
}
```

Keys that nothing else handled reach the window. That is where app-wide shortcuts go:

```svelte
<script lang="ts">
  function shortcut(event: KeyboardEvent): void {
    if (event.ctrlKey && event.key === "k") {
      event.preventDefault();
      search.focus();
    }
  }
</script>

<svelte:window onkeydown={shortcut} />
```

Window key events also have the DOM's `ctrlKey`, `shiftKey`, `altKey` and `metaKey`, so
`KeyboardEvent` handlers work. `onkeyup` on the window hears key releases.

A focused `Button` is pressed by Enter and Space. There is no Tab navigation between
elements yet.

## Fields

| Event | On | Carries |
|---|---|---|
| `oninput` | `Input` | The new text, on every change. `bind:value` is usually simpler. |
| `onsubmit` | `Input` | The text, on Enter in a single-line field. |
| `oncommit` | `Input` | The text, when it changed since focus and then focus left or Enter was pressed. Good for saving. |

## Layout, scrolling and loading

| Event | On | Carries |
|---|---|---|
| `onlayout` | `View`, `Text`, `Scroll`, `Canvas` | `{ x, y, width, height }` in window coordinates, whenever it changes. |
| `onresize` | `View` | The same, only when the width or height changed, not when the element merely moved. Use it to measure rows or panels that scroll, where `onlayout` fires on every frame of the scroll. |
| `onscroll` | `Scroll` | `{ x, y, maxX, maxY }` |
| `onload`, `onerror` | `Image` | nothing |

`onlayout` reports after the frame that laid the element out. Use it to size things
from what the layout decided, such as a popover's position or a chart's width. Do not
use it to feed the same element's size back in a loop.

## Overlays

| Event | On | Carries |
|---|---|---|
| `onoutside` | `Overlay` | `PointerEvent` plus `anchor: boolean`. A button went down outside the overlay, and outside any overlay opened from inside it. `anchor` says it went down on the element the overlay is placed against, which usually handles the press itself. |

A right press reaches the app as `ondown` with `button: 2`, which is how `ContextMenu` opens. Note that `ondown` captures the
pointer for any button.

## Files dragged in

Files dragged onto the window from other programs go to the innermost element under the pointer that
listens for any of these events. They are on `View`.

| Event | Carries |
|---|---|
| `ondragenter` | `FileDragEvent`. Files came over the element. |
| `ondragover` | `FileDragEvent`. They moved over it. |
| `ondragleave` | `FileDragEvent`. They left it, or the window. |
| `ondrop` | `FileDragEvent`. They were dropped on it. |

```ts
interface FileDragEvent {
  x: number; y: number;              // inside the element, from its top left
  windowX: number; windowY: number;  // inside the window
  paths: string[];                   // absolute paths
  count: number;
}
```

The names avoid the ones Svelte delegates, so each handler sits on its own element. Only `ondrop` makes the paths
usable: **the host allows the dropped paths through the [file limit](io.md#limiting-file-access)** from then on, as it
does paths picked in a dialog (a dropped folder allows everything below it), so `files.readText(path)` works on
them even outside the app's `allow` list. The paths in the earlier events are the same, but the app may not read them yet.
Nothing is allowed for a drop that no element listens for.

```svelte
<View
  style={{ borderColor: over ? "#8b5cf6" : "#3a3a42", borderWidth: 2, height: 120 }}
  ondragenter={() => (over = true)}
  ondragleave={() => (over = false)}
  ondrop={async (e) => { over = false; for (const path of e.paths) await open(path); }}
>
  <Text>Drop files here</Text>
</View>
```

Dragging things inside the app is a different thing, done with `Draggable` and `DropTarget`
(see [Primitives](primitives.md#draggable-and-droptarget)), which use pointer capture.

## The window

`<svelte:window>` hears:

- `onkeydown` and `onkeyup`: keys, as above.
- `onresize`: the window's size changed. Read `window.innerWidth` and
  `window.innerHeight`.

Pointer events are not delivered to the window. Put a `View` with `flex: 1` at the root
and listen there.
