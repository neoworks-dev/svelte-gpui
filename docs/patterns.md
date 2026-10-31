# Patterns

## Building your own controls

Checkboxes, switches, sliders, tabs and menus are not primitives. Build them from
`Button`, `View`, `Text` and `Icon` as ordinary Svelte components with bindable props.
`examples/demo/src/components` has a checkbox, a switch, a slider and a segmented
control.

A checkbox:

```svelte
<script lang="ts">
  import { Button, Icon, Text, View } from "@neoworks/svelte-gpui/ui";
  import check from "@phosphor-icons/core/bold/check-bold.svg";

  let { checked = $bindable(false), label }: { checked?: boolean; label: string } = $props();
</script>

<Button style={{ gap: 8 }} onpress={() => (checked = !checked)}>
  <View
    style={{
      size: 16, radius: 4, borderWidth: 1, align: "center", justify: "center", transition: 120,
      borderColor: checked ? "#8b5cf6" : "#3a3a42",
      background: checked ? "#8b5cf6" : "transparent",
    }}
  >
    {#if checked}<Icon svg={check} size={10} color="#ffffff" />{/if}
  </View>
  <Text>{label}</Text>
</Button>
```

Because `Button` is focusable and presses on Enter and Space, the checkbox works from
the keyboard without extra code.

A slider uses pointer capture, so dragging keeps working outside the track:

```svelte
<script lang="ts">
  import { View, type PointerEvent } from "@neoworks/svelte-gpui/ui";

  let { value = $bindable(50), width = 200 } = $props();
  let dragging = false;

  function seek(event: PointerEvent): void {
    value = Math.round((Math.min(Math.max(event.x, 0), width) / width) * 100);
  }
</script>

<View
  style={{ width, height: 20, justify: "center", cursor: "pointer" }}
  ondown={(e) => { dragging = true; seek(e); }}
  onmove={(e) => dragging && seek(e)}
  onup={() => (dragging = false)}
>
  <View style={{ height: 4, radius: 2, background: "#3a3a42" }}>
    <View style={{ width: (value / 100) * width, height: 4, radius: 2, background: "#8b5cf6" }} />
  </View>
  <View style={{ position: "absolute", left: (value / 100) * width - 8, top: 2, size: 16, radius: 8, background: "#fff" }} />
</View>
```

Tabs or a segmented control are a row of `Button`s whose style depends on which one is
selected, with `transition` so the change eases.

## Layout recipes

**An app shell**, with a fixed sidebar and content that scrolls:

```svelte
<View style={{ flex: 1, direction: "row", background: "#0c0c0e" }}>
  <View style={{ width: 220, shrink: 0, padding: 12, gap: 4, borderRightWidth: 1, borderColor: "#26262c" }}>
    …navigation…
  </View>
  <Scroll style={{ flex: 1, minWidth: 0 }}>
    <View style={{ padding: 24, gap: 16 }}>…</View>
  </Scroll>
</View>
```

**Centred content**: `align: "center", justify: "center"` on the parent.

**A row whose text truncates** next to fixed items:

```svelte
<View style={{ direction: "row", align: "center", gap: 8 }}>
  <Icon svg={file} size={14} />
  <Text style={{ flex: 1, minWidth: 0, truncate: true }}>{name}</Text>
  <Text style={{ shrink: 0, color: "#6b6b74" }}>{size}</Text>
</View>
```

**Pushing something to the end of a row**: a spacer `<View style={{ flex: 1 }} />`, or
`marginLeft: "auto"` on the item.

**A modal dialog**: an absolutely positioned `View` with `inset: 0`, placed last
in the root so it draws on top:

```svelte
<View style={{ flex: 1 }}>
  …the app…
  {#if dialog}
    <Presence in={[fade, { duration: 120 }]} out={[fade, { duration: 120 }]} style={{ position: "absolute", inset: 0, background: "#00000099", align: "center", justify: "center" }}>
      <View style={{ width: 360, padding: 20, gap: 12, radius: 12, background: "#16161a" }} onpress={() => {}}>
        …
      </View>
    </Presence>
  {/if}
</View>
```

The empty `onpress` on the dialog stops presses inside it from reaching a
close-on-press handler on the backdrop. That works because nothing bubbles, and the
dialog is the element pressed. For things that float next to an element rather than cover the window, use
[an overlay](#overlays-menus-and-tooltips).

**Positioning against something else**: use `Popover` or `Overlay`, below. For anything else, read
its position with `onlayout` and place an absolute element from that.

## Long lists

Use `List`. It builds only the rows near the window, so a 100,000-item array is fine. Filter and sort the
array with `$derived`; the list follows.

```svelte
<script lang="ts">
  let query = $state("");
  const shown = $derived(query === "" ? items : items.filter((item) => item.name.includes(query)));
  let list: ReturnType<typeof List> | undefined = $state();
</script>

<Input bind:value={query} placeholder="Filter" />
<List bind:this={list} items={shown} rowHeight={30} key={(item) => item.id} style={{ flex: 1 }}>
  {#snippet row(item)}
    <Button style={{ height: 30, paddingX: 10, background: item.id === selected ? "#8b5cf622" : "transparent" }} onpress={() => (selected = item.id)}>
      <Text style={{ truncate: true }}>{item.name}</Text>
    </Button>
  {/snippet}
</List>
<Button onpress={() => list?.scrollToIndex(0)}><Text>Top</Text></Button>
```

**Rows of different heights**, such as chat messages or wrapped text: leave out `rowHeight`. Each row is
measured as it is drawn, and rows not drawn yet count as the average. The host lays out the built rows
itself, so a row above the window that turns out taller than guessed does not move what you are looking at.

```svelte
<List items={messages} key={(message) => message.id} stickToEnd style={{ flex: 1 }}>
  {#snippet row(message)}
    <View style={{ padding: 8 }}><Text>{message.text}</Text></View>
  {/snippet}
</List>
```

- Give `key` when items are inserted at the start or reordered, so measured heights follow their items.
- `stickToEnd` opens a chat at its end and keeps it there, while the user has not scrolled up, as items are added and rows change height. The host does it in its own layout pass, so there is nothing to call. Scrolling back to the end takes it again.
- `list.scrollToIndex(i, { align: "end" })` and `"center"` and `"nearest"` bring a row into view.
- Content inside a row that changes its height later (an image loading, a row that expands) is measured again.

**A tree** is a `List` of the rows of expanded branches. Give `Tree` the top level, how to find a node's children
and a key, and draw a row:

```svelte
<Tree items={roots} key={(n) => n.path} childrenOf={(n) => children[n.path]} isBranch={(n) => n.folder}
      bind:selected onexpand={load} style={{ height: 320 }}>
  {#snippet row(node, info)}<Text style={{ color: info.selected ? "#fff" : "#a1a1aa" }}>{node.name}</Text>{/snippet}
</Tree>
```

`childrenOf` is called only for expanded branches, so children can be made, or loaded, when asked for: load a
folder in `onexpand` into a `$state` map and let `childrenOf` read it.

## Overlays, menus and tooltips

Everything that floats next to an element is a `Popover` (or an `Overlay` under it): placed against the
element's box, flipped and shifted to stay in the window, drawn above everything.

```svelte
<Popover bind:open side="bottom" align="end">
  {#snippet trigger({ toggle })}<Button onpress={toggle}><Text>Share</Text></Button>{/snippet}
  <View style={{ padding: 12, width: 260 }}>…</View>
</Popover>
```

- **Closing:** an outside press and Escape close it, and focus returns to what had it. Build your own close
  button with `close()` from the snippet argument.
- **Menus and dropdowns:** `Menu` inside a `Popover` with `focusPanel={false}` (the menu takes focus itself),
  or the ready-made `Select` and `ContextMenu`.
- **A context menu** is a right press: `ondown` with `event.button === 2`. `ContextMenu` does that for a region, or `menu.openAt(x, y, items)` when your own code takes the press.
- **Tooltips** wrap their content: `<Tooltip text="Save"><Button>…</Button></Tooltip>`.
- **A panel's width** is its content's, so give wide content a `width`. `matchWidth` makes it at least as wide as the trigger.
- **Do not clip them.** An overlay inside another overlay is drawn in place, so `overflow: "hidden"` on the content around it cuts it off.

## Dragging

**Files from other programs**: `ondragenter`, `ondragleave` and `ondrop` on a `View`, with absolute paths that the
app may read once dropped. See [Events](events.md#files-dragged-in).

**Inside the app**: wrap what can be picked up in `Draggable` and where it can go in `DropTarget`. A reorderable
list marks where the card would land from the pointer's place in the target under it:

```svelte
{#each items as item, index (item.id)}
  <DropTarget ondragover={(info) => (at = info.y < info.height / 2 ? index : index + 1)} ondrop={({ data }) => move(data, at)}>
    <Draggable data={item.id} onend={() => (at = null)}><Text>{item.title}</Text></Draggable>
  </DropTarget>
{/each}
```

`dragging.active` (from `@neoworks/svelte-gpui/ui`) is reactive, for dimming or outlining things everywhere
while a drag is under way. For a drag that is not about dropping, such as a slider or a resize handle, pointer
capture alone is enough, as shown above.

## Drawing

For many marks (strokes, plots, particles), use `Canvas`, not a `View` per mark. Each
`View` is an element the host lays out every frame, while a canvas is one image. See
[Canvas](primitives.md#canvas).

## Keyboard shortcuts

App-wide shortcuts go on `<svelte:window onkeydown>`. Keys reach the app before a
focused field handles them, so call `preventDefault()` on a key you use, or the field
types it as well:

```svelte
<script lang="ts">
  const sections = ["home", "search", "settings"];
  let current = $state("home");

  function shortcut(event: KeyboardEvent): void {
    if (!event.ctrlKey) return;
    const index = Number(event.key) - 1;
    if (index >= 0 && index < sections.length) {
      event.preventDefault();
      current = sections[index];
    }
  }
</script>

<svelte:window onkeydown={shortcut} />
```

A shortcut that only applies inside part of the app goes in `onkey` on a `View` around
that part. It hears keys while anything inside has focus.

## Icons

Any SVG icon set whose files are on disk works. `@phosphor-icons/core` ships plain SVG
files:

```ts
import gear from "@phosphor-icons/core/regular/gear.svg";
import gearFill from "@phosphor-icons/core/fill/gear-fill.svg";
```

Icons take the text colour around them, so a button's `color` and
`hover={{ color }}` recolour its icon and label together.

## State that outlives components

Put it in a `.svelte.ts` module with runes, as on the web:

```ts
// settings.svelte.ts
export const settings = $state({ theme: "dark", fontSize: 14 });
```

## Clipboard

```ts
import { clipboard } from "@neoworks/svelte-gpui";

await clipboard.write("copied");
const text = await clipboard.read();
```

Fields copy, cut and paste by themselves. `clipboard.read()` answers with what the system
clipboard held when the host last looked: at the last Ctrl+V or Cmd+V, before the app hears
the key, and at the last paste into a field. A paste shortcut of the app's own, such as
pasting onto a canvas, reads it in its key handler.
