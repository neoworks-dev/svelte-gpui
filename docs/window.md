# The window

An app has one window. It opens it with options, reads what the window is like, and asks
things of it, all through `window` from `@neoworks/svelte-gpui`:

```ts
import { window } from "@neoworks/svelte-gpui";
```

This is not the browser's `window`: there is no DOM, and the only things on it are the ones
listed here. `window.innerWidth` and `<svelte:window onresize>` still work as on the web.

## Opening options

`start` takes the window's options as `options.window`, and the host applies them when it
opens the window:

```ts
start(App, {}, {
  window: { title: "My app", width: 1100, height: 760, minWidth: 720, minHeight: 480 },
});
```

| Option | Meaning |
|---|---|
| `title` | The title the window manager shows. |
| `width`, `height` | The size of the content in logical pixels. The default is 668 by 821. |
| `minWidth`, `minHeight` | The smallest size the user can drag the window to. It can only be set here. |

The host's `--width` and `--height` flags override the size, so one build can be tried at
several sizes. The size is what the app asks for. A tiling window manager decides the real
size of a window it tiles, and the app learns it from `window.size`.

## What the window is like

These are reactive. A component or `$effect` that reads one runs again when it changes, and
they are kept current as the user resizes, focuses or maximizes the window.

| | |
|---|---|
| `window.size` | `{ width, height }`, the content's size in logical pixels. |
| `window.scale` | Device pixels per logical pixel. 2 on a high-density display. |
| `window.focused` | Whether the window has the keyboard focus. |
| `window.maximized` | Whether the window is maximized. A tiling window manager may report `true` for a tiled window. |
| `window.fullscreen` | Whether the window is fullscreen. |

They hold their defaults (size 0 by 0, scale 1, focused) until the window has drawn its first
frame, which is before the user can see anything.

```svelte
<script lang="ts">
  import { window } from "@neoworks/svelte-gpui";
  import { Text } from "@neoworks/svelte-gpui/ui";
  const narrow = $derived(window.size.width < 700);
</script>

<Text style={{ opacity: window.focused ? 1 : 0.6 }}>{narrow ? "Compact" : "Wide"} layout</Text>
```

## Asking things of the window

| | |
|---|---|
| `window.setTitle(title)` | Changes the title. |
| `window.minimize()` | |
| `window.maximize()`, `window.unmaximize()`, `window.toggleMaximize()` | |
| `window.setFullscreen(on)`, `window.toggleFullscreen()` | |
| `window.close()` | Closes the window, as the close button does: the handlers from `onCloseRequested` are asked first. |
| `window.destroy()` | Closes the window at once, without asking. Closing the only window ends the app. |

Each request is carried out on the next frame, and the window manager has the last word. It
can ignore `minimize` or `maximize`; Hyprland ignored `maximize` in testing.

## Asking before the window closes

`window.onCloseRequested(handler)` calls `handler` whenever the window is asked to close:
by its close button, by the window manager, or by `window.close()`. The handler decides:

- Return `false`, and the window stays open.
- Return a promise, and the window stays open until it resolves. A promise that resolves to
  anything but `false` then closes the window, so a handler can show a confirmation and
  answer when the user has chosen.
- Return anything else, or nothing, and the window may close.

It returns a function that removes the handler. With several handlers, every one has to
agree. A handler that throws is logged and counts as agreeing, so a bug cannot trap the user.

```svelte
<script lang="ts">
  import { window } from "@neoworks/svelte-gpui";

  let dirty = $state(false);

  // Registered while the component lives; the returned function removes it.
  $effect(() => window.onCloseRequested(() => (dirty ? ask() : undefined)));

  let answer: ((close: boolean) => void) | undefined;
  let asking = $state(false);
  function ask(): Promise<boolean> {
    asking = true;
    return new Promise((resolve) => (answer = resolve));
  }
</script>

{#if asking}
  <!-- Two buttons calling answer?.(false) and answer?.(true), in an overlay. -->
{/if}
```

While a promise is pending, further requests to close are ignored. `examples/demo` does this
for the Files section's note: close the window with unsaved changes, and it asks first.

## What is not there

GPUI 0.2.2 on Wayland does not support these, so the API leaves them out rather than
pretending:

- **Position** (`setPosition`, `center`). A Wayland client has no say over where its window
  goes; the compositor places it.
- **Resizing a window that is open** (`setSize`). GPUI's `resize` changes only its own idea of
  the window's size. The compositor does not hear of it, and the window draws nothing until
  the compositor sends a size of its own. Open the window at the size you want instead.
- **Changing the minimum size later** (`setMinSize`). Wayland takes it when the window is
  created, and GPUI has no call to change it.
- **Making the window not resizable.** GPUI's `is_resizable` option is ignored on Linux.

Several windows, native menus, notifications and a tray icon are separate features.
