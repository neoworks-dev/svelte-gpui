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

**An overlay or dialog**: an absolutely positioned `View` with `inset: 0`, placed last
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
dialog is the element pressed.

**Positioning against something else**: read its position with `onlayout` and place an
absolute element from that.

## Long lists

Use `List` when rows share one height. It builds only the rows near the window, so a
100,000-item array is fine. Filter and sort the array with `$derived`; the list follows.

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

For rows of different heights, use `Scroll` with `{#each}`. It builds every row, which
is fine up to a few thousand.

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

Fields copy, cut and paste by themselves.
