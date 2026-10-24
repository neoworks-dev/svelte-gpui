# svelte-gpui documentation

svelte-gpui turns Svelte 5 components into native desktop apps drawn by
[GPUI](https://www.gpui.rs/), the UI framework behind Zed. You write ordinary Svelte:
runes, snippets, `{#each}`, transitions, `svelte/motion`. Instead of HTML elements you
use a small set of primitives, and instead of CSS you pass style objects, the way
React Native works. There is no webview and no DOM.

```svelte
<script lang="ts">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  let count = $state(0);
</script>

<View style={{ padding: 24, gap: 12 }}>
  <Text style={{ fontSize: 24, fontWeight: "bold" }}>Hello</Text>
  <Button style={{ paddingX: 12, paddingY: 6, radius: 6, background: "#3f3f46" }} onpress={() => count++}>
    <Text>Pressed {count} times</Text>
  </Button>
</View>
```

## Contents

1. [Getting started](getting-started.md): an app's files, building, and running it.
2. [Primitives](primitives.md): every component, with its props, events and methods.
3. [Styles](styles.md): every style property and the values it takes; hover, press and
   focus states; eased transitions; themes.
4. [Events](events.md): the event names, what each one carries, focus, the keyboard
   and pointer capture.
5. [Motion](motion.md): eased styles, Svelte transitions through `Presence`, tweens,
   springs and animation frames.
6. [Patterns](patterns.md): building your own controls, long lists and trees, popovers, menus
   and tooltips, dragging, drawing, shortcuts and layout recipes.
7. [Network and files](io.md): `fetch`, `WebSocket`, the `files` API, and limiting
   file access.
8. [The window](window.md): its title, size and minimum size, what the app can read
   about it, and asking before it closes.
9. [Calling Rust](rust.md): commands your own crate registers, `invoke`, events
   Rust pushes with `listen`, and views your crate draws with GPUI, shown by `element()`.
10. [Packaging](packaging.md): one executable with the bundle and its files inside, a
    `.desktop` file and an icon.
11. [Coming from Svelte on the web](web-differences.md): what works the same, what does
    not, and why.
12. [Testing](testing.md): running an app headless and scripting input.

[Getting started](getting-started.md#development-hot-reload-and-errors) also covers
developing with hot reload and the error overlay.

`examples/stick-to-end` is a long chat opened at its end, to see `List`'s `stickToEnd` and `Scroll`'s `anchor` work.
The demo in `examples/demo` uses every feature described here. When the docs and the
demo disagree, the types in `runtime/ui/` are the reference.
