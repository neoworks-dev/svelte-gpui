# Primitives

Everything on screen is built from these components, imported from
`@neoworks/svelte-gpui/ui`:

```ts
import { Button, Canvas, Icon, Image, Input, List, Presence, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
```

Each one draws as a single GPUI element. HTML tags (`<div>`, `<span>`, `<button>`) are
not drawn at all. Neither are `class` or `style` strings: the host only knows the
primitives' own elements.

Every primitive is a flex container. `View` stacks its children in a column, and
`Button` and `Input` lay theirs out in a row; change either with `style.direction`.
The [style properties](styles.md) apply to all of them.

Props marked *state style* take a `Style` that is laid over `style` while the element
is in that state. See [States](styles.md#states).

## View

The general container, the equivalent of a `div`. It is the only primitive that takes
the full set of pointer events.

| Prop | Type | |
|---|---|---|
| `style` | `Style` | |
| `hover`, `active`, `focus` | `Style` | State styles. `active` applies while a button is held on it. |
| `focusable` | `boolean` | Takes keyboard focus when pressed, which lets it receive `onkey`. |
| `onpress` | `(e: PointerEvent) => void` | A click: a button pressed and released on it. |
| `onhover` | `(hovered: boolean) => void` | The pointer entered (`true`) or left (`false`). |
| `ondown` | `(e: PointerEvent) => void` | A button went down on it. This captures the pointer: `onmove` and `onup` follow it anywhere until release. |
| `onmove` | `(e: PointerEvent) => void` | The pointer moved over it, or anywhere while captured. |
| `onup` | `(e: PointerEvent) => void` | The button was released after `ondown`, wherever the pointer is. |
| `onwheel` | `(e: WheelEvent) => void` | Wheel or touchpad scrolling over it. |
| `onlayout` | `(l: Layout) => void` | Where it was laid out, whenever that changes. |
| `onkey` | `(e: KeyEvent) => void` | A key, while it or something inside it has focus. |
| `onfocus`, `onblur` | `() => void` | |

```svelte
<View
  style={{ padding: 12, radius: 8, background: "#27272a" }}
  hover={{ background: "#3f3f46" }}
  onpress={() => open = !open}
>
  <Text>Toggle</Text>
</View>
```

Text placed directly in a `View` is drawn, but wrap it in `Text` to style it.

## Text

A paragraph. A nested `Text` is a run inside the same paragraph, with its own colour,
weight, font, decoration or background. The paragraph wraps as one.

| Prop | Type | |
|---|---|---|
| `style` | `Style` | Text properties apply to this run and the runs inside it. |
| `pre` | `boolean` | Keep spaces and line breaks as written. Otherwise runs of whitespace collapse to one space. |
| `onlayout` | `(l: Layout) => void` | |

```svelte
<Text style={{ fontSize: 15, lineHeight: 22 }}>
  Plain, <Text style={{ fontWeight: "bold" }}>bold</Text>, and
  <Text style={{ fontFamily: "mono", background: "#27272a", color: "#c4b5fd" }}>code</Text>.
</Text>
```

To limit a paragraph, use `truncate: true` for one line with an ellipsis, or
`maxLines: n`. Text properties set on any element (`color`, `fontSize`, …) are
inherited by the text inside it, so a `Button` can set the colour of its label.

## Button

Something to press. It is focusable, and while focused, Enter and Space press it. The
pointer becomes a hand over it.

| Prop | Type | |
|---|---|---|
| `style`, `hover`, `active`, `focus` | `Style` | |
| `disabled` | `boolean` | No presses. The cursor goes back to the arrow. |
| `disabledStyle` | `Style` | Laid over `style` while disabled. |
| `onpress` | `(e: PointerEvent) => void` | |
| `onhover` | `(hovered: boolean) => void` | |
| `onkey` | `(e: KeyEvent) => void` | |
| `onfocus`, `onblur` | `() => void` | |

| Method | |
|---|---|
| `focus()` | Gives it keyboard focus. |

A button has no look of its own; style it. A button with no content is fine, for
example a colour swatch.

## Input

A text field. The host owns the text while it is edited: typing, input methods
(composition and preedit), selection by keyboard and pointer, word movement, and the
clipboard all work without app code. Each change reaches the app through
`bind:value` or `oninput`. Setting `value` replaces the text.

| Prop | Type | |
|---|---|---|
| `value` | `string`, bindable | |
| `placeholder` | `string` | Shown dimmed while the field is empty. |
| `multiline` | `boolean` | Several lines. Enter breaks the line instead of submitting. |
| `rows` | `number` | The least number of lines a multiline field shows. The default is 3. |
| `secure` | `boolean` | Shows dots instead of the text. |
| `numeric` | `boolean` | Accepts only digits, `.`, `-`, `+` and `e`. |
| `maxLength` | `number` | |
| `disabled`, `readonly` | `boolean` | |
| `style`, `hover`, `focus` | `Style` | |
| `oninput` | `(value: string) => void` | Every change to the text. |
| `onsubmit` | `(value: string) => void` | Enter in a single-line field. |
| `oncommit` | `(value: string) => void` | The text changed since the field took focus, and then focus left or Enter was pressed. |
| `onkey` | `(e: KeyEvent) => void` | Comes before the field handles the key. `preventDefault()` stops the field from handling it. |
| `onfocus`, `onblur` | `() => void` | |

| Method | |
|---|---|
| `focus()` | Gives it keyboard focus. |
| `select(start?, end?)` | Selects a range in UTF-16 offsets, which is how JavaScript string indices count. Selects everything by default. |

```svelte
<script lang="ts">
  let field: Input;
  let query = $state("");
</script>

<Input
  bind:this={field}
  bind:value={query}
  placeholder="Search"
  style={{ paddingX: 10, paddingY: 6, radius: 6, background: "#18181b", borderWidth: 1, borderColor: "#3f3f46" }}
  focus={{ borderColor: "#8b5cf6" }}
  onsubmit={(text) => search(text)}
/>
```

## Scroll

Content that scrolls. It needs a bounded size to scroll within, such as `flex: 1` in a
column or a fixed `height`.

| Prop | Type | |
|---|---|---|
| `style` | `Style` | |
| `horizontal` | `boolean` | Scroll sideways instead of down. |
| `onscroll` | `(p: ScrollPosition) => void` | The offset and how far it can go: `{ x, y, maxX, maxY }`. |
| `onlayout` | `(l: Layout) => void` | |

| Method | |
|---|---|
| `scrollTo(offset)` | Moves the top (or left) edge to `offset` pixels. |

## List

A long list of rows with the same height, built only near the window. 100,000 items
cost what a screenful does.

| Prop | Type | |
|---|---|---|
| `items` | `T[]` | |
| `rowHeight` | `number` | Every row's height, in pixels. |
| `row` | `Snippet<[T, number]>` | Draws one row, from the item and its index. |
| `key` | `(item: T, index: number) => unknown` | Keeps a row's state with its item when the list changes. |
| `overscan` | `number` | How many rows past each edge are built anyway. The default is 8. |
| `style` | `Style` | Give it a bounded height, as for `Scroll`. |

| Method | |
|---|---|
| `scrollToIndex(index)` | Moves that row to the top. |

```svelte
<List items={files} rowHeight={28} key={(file) => file.id} style={{ flex: 1 }}>
  {#snippet row(file, index)}
    <View style={{ direction: "row", align: "center", paddingX: 8, height: 28 }}>
      <Text>{file.name}</Text>
    </View>
  {/snippet}
</List>
```

Use `Scroll` with `{#each}` for rows of different heights. That builds every row.

## Image

| Prop | Type | |
|---|---|---|
| `src` | `string` | A path relative to the bundle (or to `--assets`), a `data:` URL, or an `http(s)` URL. URLs are fetched on a thread and cached on disk. |
| `fit` | `"contain"` \| `"cover"` \| `"fill"` | How the image fills its box. |
| `style` | `Style` | Give it a size. `radius` rounds the image itself. |
| `onload`, `onerror` | `() => void` | When the source has loaded or failed. |

Images load only once they come within a screen of the window, and are decoded no
larger than they are drawn. Until an image loads, its box keeps the image's
proportions where they are known.

## Icon

An SVG, tinted with one colour.

| Prop | Type | |
|---|---|---|
| `svg` | `string` | SVG markup. |
| `src` | `string` | Or an SVG file's path. |
| `size` | `number` | Width and height in pixels. The default is the font size around it. |
| `color` | `Color` | The default is the text colour around it, so a button's `color` and `hover={{ color }}` tint its icon. |
| `style` | `Style` | |

The build loads `.svg` imports as their text, so an icon set's files can be used as
they are:

```svelte
<script lang="ts">
  import gear from "@phosphor-icons/core/regular/gear.svg";
</script>

<Icon svg={gear} size={18} />
```

## Canvas

Pixels to paint into. The host keeps a buffer at the window's scale and rasterises each
call as it arrives, so a drawing costs the same however much is already on it. The
drawing is kept when the canvas resizes or the window moves to a screen with another
scale.

| Prop | Type | |
|---|---|---|
| `style` | `Style` | Give it a size. |
| `ondown`, `onmove`, `onup` | `(e: PointerEvent) => void` | As on `View`, with pointer capture. |
| `onhover`, `onwheel`, `onlayout` | | As on `View`. |

| Method | |
|---|---|
| `begin(x, y, pen?)` | Starts a stroke, drawing a dot at the point. `pen` is `{ color?, width? }`; the defaults are white and 2. |
| `lineTo(x, y)` | Continues the stroke. Joins and caps are round. |
| `end()` | Ends the stroke. |
| `line(x0, y0, x1, y1, pen?)` | One line segment. |
| `fillRect(x, y, width, height, color)` | |
| `fillCircle(x, y, radius, color)` | |
| `clear(color?)` | Clears to transparent, or fills with a colour. |

Coordinates are the canvas's own logical pixels from its top left, which is what its
pointer events report:

```svelte
<script lang="ts">
  let pad: Canvas;
</script>

<Canvas
  bind:this={pad}
  style={{ width: 400, height: 300, background: "#0c0c0e", cursor: "crosshair" }}
  ondown={(e) => pad.begin(e.x, e.y, { color: "#8b5cf6", width: 4 })}
  onmove={(e) => pad.lineTo(e.x, e.y)}
  onup={() => pad.end()}
/>
```

Canvas has no curves, text or images yet, and pixels cannot be read back.

## Presence

A `View` that plays Svelte transitions as it appears and disappears. Svelte only allows
`in:`, `out:` and `transition:` on elements, never on components, so `Presence` carries
them for you.

| Prop | Type | |
|---|---|---|
| `in` | `fn` or `[fn, params]` | The transition to play on entry, such as `fade` or `[fly, { y: 8, duration: 200 }]`. |
| `out` | `fn` or `[fn, params]` | The transition to play on exit. |
| `style` | `Style` | |

```svelte
{#if open}
  <Presence in={[fly, { y: -8, duration: 160 }]} out={fade} style={{ padding: 12 }}>
    <Text>Saved</Text>
  </Presence>
{/if}
```

See [Motion](motion.md) for which transitions draw.

## Helpers

From `@neoworks/svelte-gpui/ui`:

- `merge(...styles)` lays styles over each other, later ones winning, and skips `false`,
  `null` and `undefined`: `merge(button, primary && accent, disabled && muted)`.
- The types `Style`, `Size`, `Color`, `Sides`, `Align`, `Shadow`, `Transition` and
  `Pen`, and the event types `PointerEvent`, `WheelEvent`, `KeyEvent`, `Layout`,
  `ScrollPosition` and `Modifiers`.

From `@neoworks/svelte-gpui`:

- `start(Component, props?)`.
- `frameStats()` returns `{ fps, buildMs }`: the frames drawn in the last second, and
  the average time building their elements took. GPUI only draws when something
  changed, so an idle window reports few frames.
- `clipboard.write(text)` and `clipboard.read()`, which return promises.
- `files`, `FileError` and their types. See [Network and files](io.md).
