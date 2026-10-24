# Primitives

Everything on screen is built from these components, imported from
`@neoworks/svelte-gpui/ui`:

```ts
import { Button, Canvas, Icon, Image, Input, List, Presence, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
```

Built on those are components for floating content (`Overlay`, `Popover`, `Tooltip`, `Menu`,
`ContextMenu`, `Select`), for dragging inside the app (`Draggable`, `DropTarget`) and for
trees (`Tree`), described after the primitives.

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
| `keepFocus` | `boolean` | A press on it, or on anything inside it, does not move focus: the field that has it keeps it, and nothing inside takes it. See [Events](events.md#keyboard-and-focus). |
| `onpress` | `(e: PointerEvent) => void` | A click: a button pressed and released on it. |
| `onhover` | `(hovered: boolean) => void` | The pointer entered (`true`) or left (`false`). |
| `ondown` | `(e: PointerEvent) => void` | A button went down on it. This captures the pointer: `onmove` and `onup` follow it anywhere until release. |
| `buttons` | `("left" \| "middle" \| "right")[]` | The buttons `ondown` takes. A press of any other passes to what is around the view, which can take it with its own `ondown`. The default is all three. |
| `onmove` | `(e: PointerEvent) => void` | The pointer moved over it, or anywhere while captured. |
| `onup` | `(e: PointerEvent) => void` | The button was released after `ondown`, wherever the pointer is. |
| `onwheel` | `(e: WheelEvent) => void` | Wheel or touchpad scrolling over it. |
| `onlayout` | `(l: Layout) => void` | Where it was laid out, whenever that changes. |
| `onresize` | `(l: Layout) => void` | Like `onlayout`, but only when its width or height changed, not when it merely moved. Cheaper for things that scroll. |
| `onkey` | `(e: KeyEvent) => void` | A key, while it or something inside it has focus. |
| `onfocus`, `onblur` | `() => void` | |
| `ondragenter`, `ondragleave`, `ondragover`, `ondrop` | `(e: FileDragEvent) => void` | Files dragged in from another program. See [Events](events.md#files-dragged-in). |

| Method | |
|---|---|
| `focus()` | Gives it keyboard focus. It must be `focusable`. |
| `blur()` | Gives focus up, if it has it. It goes back to the element that had it before, when that is still there, and otherwise to nothing. |

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
| `blur()` | Gives focus up, if it has it. It goes back to the element that had it before, when that is still there, and otherwise to nothing. |

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
| `onselect` | `(s: { start, end }) => void` | The cursor or the selection moved, whether by keys, the pointer, typing, or `select()`. `start` and `end` are UTF-16 offsets, equal when nothing is selected. It does not fire when the app sets `value`; ask `getSelection()` for that. |
| `onkey` | `(e: KeyEvent) => void` | Comes before the field handles the key. `preventDefault()` stops the field from handling it. |
| `onfocus`, `onblur` | `() => void` | |

| Method | |
|---|---|
| `focus()` | Gives it keyboard focus. |
| `blur()` | Gives focus up, if it has it. It goes back to the element that had it before, when that is still there, and otherwise to nothing. |
| `select(start?, end?)` | Selects a range in UTF-16 offsets, which is how JavaScript string indices count. Selects everything by default. |
| `getSelection()` | Where the cursor is right now: `{ start, end, firstRow, lastRow }`, or `null` until the field has been drawn. `start` and `end` are UTF-16 offsets as for `select()`. `firstRow` and `lastRow` say whether the cursor is on the first and the last row of the text as drawn, so a long line that wraps counts as several rows. |

Reading the cursor in `onkey` gives where it is before the field handles the key, which is what a field
that walks a history with Up and Down needs: take the arrow only when the cursor cannot move any further.

```svelte
<Input bind:this={field} multiline onkey={(e) => {
  if (e.key === "ArrowUp" && field.getSelection()?.firstRow) {
    e.preventDefault();
    showOlderEntry();
  }
}} />
```

The rows are those of the last frame, so they are right for any key that comes after the text on screen has
changed, which is every key a person types.

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
| `anchor` | `"content" \| "end"` | What the host keeps in view while it lays out. A vertical scroller only; see below. |
| `onscroll` | `(p: ScrollPosition) => void` | The offset and how far it can go: `{ x, y, maxX, maxY }`. It fires when the offset changes, and also when the content grows or shrinks and `maxY` changes with it. |
| `onlayout` | `(l: Layout) => void` | |

| Method | |
|---|---|
| `scrollTo(offset)` | Moves the top (or left) edge to `offset` pixels. |
| `scrollBy(delta)` | Moves by `delta` pixels, down or right when positive, and stops at the ends. |
| `position()` | `{ x, y, maxX, maxY }`: where it is and how far it can go. |

`position()` is known without an `onscroll` handler, and is right straight after `scrollTo` or `scrollBy`. Read in a
template or an effect, it updates the way state does. It reads zeros until the host has laid the content out, so
`scrollBy` does nothing before the first frame. Together they make keyboard scrolling short:

```svelte
<script lang="ts">
  let scroller: Scroll;
</script>

<View focusable onkey={(e) => {
  if (e.key === "ArrowDown") scroller.scrollBy(40);
  if (e.key === "ArrowUp") scroller.scrollBy(-40);
}}>
  <Scroll bind:this={scroller} style={{ flex: 1 }}>…</Scroll>
</View>
```

### Anchoring

With `anchor`, the host keeps what you are looking at in place while the content changes, during its own layout
pass, so the app does not measure, scroll and correct in rounds. Only the scroller's own children are kept in
place, so put what should hold still directly in it (`List` does).

- `anchor="content"`: when what is above the window changes size (a row taller than guessed, rows added above), the
  child at the top of the window stays where it is on screen, as `overflow-anchor` does in a browser.
- `anchor="end"`: the same, and the scroller opens scrolled to its end, on its first frame. While it is at the end,
  content that grows or shrinks (a row changing height, rows added, text wrapping again after a resize) keeps the
  end in view. Scrolling up lets go of it. Scrolling back to the end, or to an offset at or past `maxY` with
  `scrollTo`, takes it again.

```svelte
<Scroll anchor="end" style={{ flex: 1 }}>
  {#each messages as message (message.id)}
    <Message {message} />
  {/each}
</Scroll>
```

`examples/stick-to-end` is a long chat of rows of unknown height, opened at its end, with buttons that grow the last
row and a row above the window.

## List

A long list of rows, built only near the window. 100,000 items cost what a screenful does.
Rows may all have one height (`rowHeight`), or differ. A row of its own height is drawn once it
has been measured, and the built rows sit one under the other between two empty spacers that stand for the rows
above and below, so the host lays out their real heights at once.

| Prop | Type | |
|---|---|---|
| `items` | `T[]` | Read as a whole: pass a new array when it changes, as `$derived` and `.filter()` do. |
| `rowHeight` | `number` | Every row's height, in pixels. Leave it out for rows that differ. |
| `estimatedRowHeight` | `number` | Without `rowHeight`: what a row not measured yet counts as, before any has been. The default is 40. |
| `row` | `Snippet<[T, number]>` | Draws one row, from the item and its index. |
| `key` | `(item: T, index: number) => unknown` | Keeps a row's state, and its measured height, with its item when the list changes. |
| `overscan` | `number` | How many rows past each edge are built anyway. The default is 8. |
| `stickToEnd` | `boolean` | Opens at the end, and stays there as items are added and rows change height, as a chat does. Scrolling up lets go of it; scrolling back to the end takes it again. Needs no `scrollToIndex` to start at the end. |
| `style` | `Style` | Give it a bounded height, as for `Scroll`. |
| `onscroll` | `(p: ScrollPosition) => void` | `y` counts from the top of the whole list. |

| Method | |
|---|---|
| `scrollToIndex(index, { align? })` | Brings a row into view. `align` is `"start"` (the default, row at the top), `"end"`, `"center"` or `"nearest"` (only as far as it takes). |

```svelte
<List items={files} rowHeight={28} key={(file) => file.id} style={{ flex: 1 }}>
  {#snippet row(file, index)}
    <View style={{ direction: "row", align: "center", paddingX: 8, height: 28 }}>
      <Text>{file.name}</Text>
    </View>
  {/snippet}
</List>
```

**Rows of different heights.** Without `rowHeight`, each row is measured as it is drawn: its height is
whatever its content comes to, wrapped text included. Rows not drawn yet count as the average of those
measured. This is what lets it hold a hundred thousand chat messages:

```svelte
<List items={messages} key={(message) => message.id} stickToEnd style={{ flex: 1 }}>
  {#snippet row(message)}
    <View style={{ padding: 8 }}><Text>{message.text}</Text></View>
  {/snippet}
</List>
```

- **The window does not move when rows above it change.** The built rows sit in an anchored `Scroll`, so when a row
  above the window turns out taller or shorter than guessed, the host keeps the row at the top of the window where
  it is on screen, in the same frame. That includes inserting items above it, when `key` is given.
- **`stickToEnd` opens at the end** on the first frame, the built rows being the last ones, and the host keeps the
  end in view while rows are measured, items are added and the width changes. The app sends no scroll request for
  any of it.
- **`key` matters here.** Without it a row's height stays with its position; with it, with its item. A list that
  inserts at the start or reorders wants `key`. Adding at the end is cheap either way.
- **`scrollToIndex`** lands on the row exactly, even though rows around it are measured only after it
  gets there.
- **Width changes forget what was measured**, since text wraps differently. The rows drawn at that moment measure themselves again, so only the others fall back to the average.
- **Very long lists.** The host cannot draw at millions of pixels from the top, so the list lays out a stretch of
  a million pixels at a time and moves the stretch along as you scroll. You do not see it, and `onscroll` still
  counts from the top of the whole list.

Each row's size is read with `onresize` on a view around it, so a row costs one extra measuring layer.

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

## Overlay

Content drawn above everything else, placed against its parent's box. It takes no room where it sits, no
ancestor's `overflow` clips it, and nothing draws over it. It is placed after layout, against the box the host
just measured, so it follows the parent as it scrolls and never lags a frame. It draws nothing while it has no
children. Other components build on it; use it directly for anything `Popover` does not fit.

| Prop | Type | |
|---|---|---|
| `side` | `"top"` \| `"bottom"` \| `"left"` \| `"right"` | Which edge of the parent it sits against. The default is `"bottom"`. |
| `align` | `"start"` \| `"center"` \| `"end"` | Where along that edge. The default is `"start"`. |
| `offset` | `number` | The gap between the parent and the overlay. |
| `crossOffset` | `number` | Slides it along the edge, toward the end. |
| `at` | `[x, y]` | A point in the window to place it against instead of the parent's box, such as the pointer. |
| `flip` | `boolean` | Moves to the opposite side when the chosen one has too little room and the other has more. On by default. With `at`, the alignment flips too. |
| `shift` | `boolean` | Slides back inside the window when it would stick out. On by default. |
| `margin` | `number` | The least distance kept from the window's edges. The default is 4. |
| `matchWidth` | `boolean` | At least as wide as the parent. |
| `passthrough` | `boolean` | Lets the pointer through to what is behind it. Otherwise it blocks it. |
| `layer` | `number` | Overlays with a higher layer are drawn over those with a lower one. |
| `focusable` | `boolean` | Takes keyboard focus when pressed or when `focus()` is called. |
| `keepFocus` | `boolean` | As on `View`: a press inside it does not move focus. For a list of suggestions under a field. |
| `onoutside` | `(e: OutsideEvent) => void` | A button went down outside it (and outside any overlay opened from inside it). `e.anchor` says it went down on the parent. |
| `style`, `hover`, `active`, `focus` | `Style` | As on `View`. |
| `onpress`, `onhover`, `onlayout`, `onkey`, `onfocus`, `onblur` | | As on `View`. |

| Method | |
|---|---|
| `focus()` | Gives it keyboard focus. |
| `blur()` | Gives focus up, if it has it. It goes back to the element that had it before, when that is still there, and otherwise to nothing. |

An overlay is placed against its **parent's** box, so put it inside the element it belongs to: an overlay that
is a child of a `Button` is placed against that button. `Popover` and `Tooltip` wrap their trigger in a view for this.

Limits: an overlay inside another is drawn in place after its content, so it is clipped by an
`overflow: "hidden"` on the content around it. The window's size is read when the overlay is laid out; an overlay
inside a part of the window that did not redraw is not moved when the window is resized.

## Popover

Content that floats next to a trigger: a menu, a date picker, a card of details. It is drawn above everything,
placed against the trigger's box, flipped and shifted to stay inside the window, and closed by an outside press
or Escape. Focus moves into it when it opens and goes back when it closes.

| Prop | Type | |
|---|---|---|
| `open` | `boolean`, bindable | |
| `trigger` | `Snippet<[PopoverApi]>` | The element that opens it, drawn in a wrapper whose box is the anchor. Without one, the popover is placed against its parent, or at `at`. |
| `children` | `Snippet<[PopoverApi]>` | The content. |
| `side`, `align`, `crossOffset`, `flip`, `shift`, `margin`, `matchWidth`, `at`, `layer` | | As on `Overlay`. |
| `offset` | `number` | The gap to the trigger. The default is 6. |
| `style` | `Style` | The panel. The default is a dark card (`overlayLook.panel`). |
| `anchorStyle` | `Style` | The wrapper around the trigger. It is `alignSelf: "start"` by default, so it is as wide as the trigger. |
| `closeOnOutside`, `closeOnEscape` | `boolean` | Both on by default. |
| `focusPanel` | `boolean` | Moves focus to the panel when it opens. On by default; turn it off when the content takes focus itself, as `Menu` does. |
| `restoreFocus` | `boolean` | Gives focus back to what had it when it closes by Escape or a choice. On by default. After an outside press, focus stays where that press put it. |
| `onclose` | `(reason: CloseReason) => void` | `reason` is `"escape"`, `"outside"`, `"select"`, `"toggle"` or `"api"`. |
| `onkey` | `(e: KeyEvent) => void` | Keys while focus is inside. |

`PopoverApi` is `{ open, show(), toggle(), close(reason?) }`.

```svelte
<Popover bind:open side="bottom" align="start">
  {#snippet trigger({ toggle })}
    <Button onpress={toggle}><Text>Options</Text></Button>
  {/snippet}
  {#snippet children({ close })}
    <View style={{ width: 220, padding: 8, gap: 8 }}>
      <Text>Anything goes here</Text>
      <Button onpress={() => close("select")}><Text>Done</Text></Button>
    </View>
  {/snippet}
</Popover>
```

A press on the trigger itself does not count as outside, so the same press that closes the popover is not
followed by one that opens it again. Escape closes only the newest open popover, so a select inside a popover
closes first.

## Tooltip

A label that appears next to its content after the pointer has rested on it, and goes when it leaves. After
one has shown, the next appears at once. It never takes the pointer.

| Prop | Type | |
|---|---|---|
| `text` | `string` | |
| `content` | `Snippet` | Instead of `text`, for anything more than a line. |
| `delay` | `number` | Milliseconds of rest before it shows. The default is 500. |
| `side`, `align`, `offset` | | As on `Overlay`. The defaults are `"bottom"`, `"center"` and 6. |
| `disabled` | `boolean` | |
| `style`, `anchorStyle` | `Style` | The tooltip, and the wrapper around its content. |

```svelte
<Tooltip text="Save the document" side="top">
  <Button onpress={save}><Icon svg={floppy} /></Button>
</Tooltip>
```

## Menu

A list of actions with keyboard control: Up and Down move the highlight, Home and End jump, Enter or Space
choose, a letter jumps to the next label that starts with it. The pointer highlights what it is over and chooses
on press. It takes focus when it appears. It does not close itself: `onclose` is called after a choice, which is
where a popover is closed.

| Prop | Type | |
|---|---|---|
| `items` | `MenuItem[]` | |
| `onselect` | `(item, index) => void` | After the item's own `onselect`. |
| `onclose` | `() => void` | After any choice. |
| `initial` | `number` | The item highlighted at first. The default is the first one that is not disabled. |
| `autofocus` | `boolean` | On by default. |
| `maxHeight` | `number` | Scrolls when the rows are taller than this. |
| `width` | `number` | |
| `style`, `itemStyle` | `Style` | |

A `MenuItem` is `{ label?, onselect?, icon?, shortcut?, disabled?, danger?, checked?, separator?, value? }`:
`icon` is SVG markup, `shortcut` is text shown at the end (it does not bind the key), `checked` shows a check
mark, and `{ separator: true }` draws a line.

```svelte
<Popover bind:open focusPanel={false}>
  {#snippet trigger({ toggle })}<Button onpress={toggle}><Text>Actions</Text></Button>{/snippet}
  {#snippet children({ close })}
    <Menu items={actions} onclose={() => close("select")} />
  {/snippet}
</Popover>
```

## ContextMenu

A region with a menu that opens at the pointer when it is pressed with the right button. It flips and shifts to
stay inside the window, closes on an outside press or Escape, and gives focus back.

| Prop | Type | |
|---|---|---|
| `items` | `MenuItem[]` \| `(e: PointerEvent) => MenuItem[]` | A function makes the menu depend on what is under the pointer. Optional when `openAt` is always given its own. |
| `onselect` | `(item, index) => void` | |
| `onopen` | `(e: PointerEvent) => void` | Called when the menu's own right press opens it, not by `openAt`. |
| `style`, `menuStyle`, `panelStyle` | `Style` | The region, the menu, and the card around it. |

| Method | |
|---|---|
| `openAt(x, y, items?)` | Opens the menu at a point in the window, such as a pointer event's `windowX` and `windowY`. It uses `items`, or the `items` prop without them (a function prop is called with a pointer at that point). It flips and shifts like the pointer's own menu, and moves there when it is already open. |
| `close()` | Closes it. |

```svelte
<ContextMenu items={[{ label: "Rename", onselect: rename }, { label: "Delete", danger: true, onselect: remove }]}>
  <Text>Right-press me</Text>
</ContextMenu>
```

**From code.** A `ContextMenu` with no children is not a region: it draws nothing, takes no room and listens for
nothing, so it can sit anywhere. Call `openAt` from the right press your app handles itself, such as a list row that
selects first and then decides which menu to show:

```svelte
<script lang="ts">
  let menu: ContextMenu;
</script>

<View ondown={(e) => e.button === 2 && menu.openAt(e.windowX, e.windowY, itemsFor(row))}>…</View>
<ContextMenu bind:this={menu} />
```

Focus goes into the menu and comes back as for the pointer's own menu. A right press outside an open menu closes it first,
so pressing another row opens that row's menu in place of the first.

## Select

A dropdown: a button showing the chosen option, and a list under it as wide as the button. It opens on press, or
with Down, Enter or Space while the button has focus, has a `Menu`'s keys, and scrolls past `maxHeight`.

| Prop | Type | |
|---|---|---|
| `value` | `T`, bindable | The chosen option's value. |
| `options` | `(T \| { value, label, disabled? })[]` | Plain values use `String(value)` as the label. |
| `placeholder` | `string` | |
| `disabled` | `boolean` | |
| `maxHeight` | `number` | The default is 260. |
| `onchange` | `(value: T) => void` | |
| `style`, `hover`, `focus` | `Style` | The button. |
| `panelStyle` | `Style` | The list's card. |

```svelte
<Select bind:value={size} options={["Small", "Medium", "Large"]} />
```

## Draggable and DropTarget

Dragging inside the app. A `Draggable` starts a drag when the left button, held down on it, has moved a few pixels.
A copy of it follows the pointer, drawn above everything, and the original dims. Release over a `DropTarget` that
accepts the data to drop it; Escape, or releasing anywhere else, cancels. A press that does not move is still a
press, so buttons inside a `Draggable` work. A right or middle press is not the `Draggable`'s: it goes to what is
around it, so a `ContextMenu` or a view with `ondown` around a `Draggable` hears it.

| `Draggable` prop | Type | |
|---|---|---|
| `data` | `T` | What a target is given when this is dropped on it. |
| `disabled` | `boolean` | |
| `threshold` | `number` | Pixels the pointer moves before the drag starts. The default is 4. |
| `style`, `hover` | `Style` | |
| `draggingStyle` | `Style` | Laid over `style` while it is being dragged. The default is `{ opacity: 0.35 }`. |
| `preview` | `Snippet<[T]>` | What follows the pointer. The default is a copy of the children, in the same style and width. |
| `previewStyle` | `Style` | |
| `onstart` | `(data) => void` | |
| `onend` | `({ data, dropped }) => void` | The drag ended, dropped on a target or not. |

| `DropTarget` prop | Type | |
|---|---|---|
| `accept` | `(data: T) => boolean` | Which data it takes. A drag it does not accept passes over it. |
| `ondragenter`, `ondragover` | `(info: DragInfo<T>) => void` | |
| `ondragleave` | `() => void` | |
| `ondrop` | `(info: DragInfo<T>) => void` | |
| `style` | `Style` | |
| `over` | `Style` | Laid over `style` while an accepted drag is over it. |
| `active` | `Style` | Laid over `style` while any accepted drag is under way. |
| `children` | `Snippet<[{ over, dragging }]>` | |

`DragInfo<T>` is `{ data, x, y, windowX, windowY, width, height }`: the pointer inside the target, in the window,
and the target's size, for working out which half the pointer is in. Targets nest: the smallest one under the pointer
that accepts the data hears it. `dragging` (from `@neoworks/svelte-gpui/ui`) is reactive state, `{ active, data, x, y }`,
for showing a hint everywhere while a drag is under way.

```svelte
<DropTarget ondragover={(info) => (before = info.y < info.height / 2)} ondrop={({ data }) => insert(data, before)}>
  <Draggable data={card}><Text>{card.title}</Text></Draggable>
</DropTarget>
```

Targets are found by where they were laid out, which is reported after each frame, so a target that is
scrolled out of view but still under a clipping ancestor may be found. A drag does not scroll anything near the edge.

## Tree

Rows that expand and collapse: a file browser, an outline. Only the rows of expanded branches exist, and of those
only the ones near the window are built, because it is a `List` underneath. A branch's children are
asked for only when it is expanded, so a tree can be as large as you can describe.

| Prop | Type | |
|---|---|---|
| `items` | `T[]` | The top level. |
| `key` | `(node: T) => string \| number` | Identifies a node, for what is expanded and selected. |
| `childrenOf` | `(node: T) => T[] \| undefined \| null` | A node's children, called for expanded branches only. |
| `isBranch` | `(node: T) => boolean` | Whether a node can be expanded. The default is that it has children, which a branch loaded lazily does not yet. |
| `row` | `Snippet<[T, TreeRow]>` | Draws one row, to the right of the arrow. |
| `expanded` | `Set`, bindable | The keys of expanded nodes. Replace it to change it. |
| `selected` | bindable | The key of the selected node. |
| `rowHeight` | `number` \| `"auto"` | The default is 28. `"auto"` measures rows that differ. |
| `indent` | `number` | Per level. The default is 16. |
| `onselect`, `onactivate` | `(node) => void` | Selected by the pointer or the keys; Enter, or a second press. |
| `onexpand`, `oncollapse` | `(node) => void` | Load a branch's children in `onexpand`. |
| `style`, `rowStyle`, `selectedStyle` | `Style` | Give the tree a bounded height. |

`TreeRow` is `{ depth, index, branch, expanded, selected, toggle(), select() }`.

Click a row to select it, the arrow to expand it, a row twice to activate it. With the tree focused: Down and
Up move the selection, Right expands a branch and then goes into it, Left collapses it and then goes to its
parent, Home and End jump, and Enter activates the row (which also toggles a branch).

| Method | |
|---|---|
| `reveal(key)` | Selects the row of a node that is showing, and scrolls to it. |

```svelte
<Tree items={roots} key={(n) => n.path} childrenOf={(n) => children[n.path]} isBranch={(n) => n.folder}
      bind:selected onexpand={load} style={{ height: 320 }}>
  {#snippet row(node)}<Text>{node.name}</Text>{/snippet}
</Tree>
```

## Helpers

From `@neoworks/svelte-gpui/ui`:

- `merge(...styles)` lays styles over each other, later ones winning, and skips `false`,
  `null` and `undefined`: `merge(button, primary && accent, disabled && muted)`.
- The types `Style`, `Size`, `Color`, `Sides`, `Align`, `Shadow`, `Transition` and
  `Pen`, and the event types `PointerEvent`, `WheelEvent`, `KeyEvent`, `Layout`,
  `ScrollPosition`, `Modifiers`, `FileDragEvent` and `OutsideEvent`, and `MenuItem`,
  `TreeRow`, `DragInfo`, `PopoverApi` and `CloseReason`.
- `overlayLook`: the default look of floating panels (`panel`, `tip`, `colors`), to spread into your own.
- `dragging`: reactive state for the drag in progress.

From `@neoworks/svelte-gpui`:

- `start(Component, props?)`.
- `frameStats()` returns `{ fps, buildMs }`: the frames drawn in the last second, and
  the average time building their elements took. GPUI only draws when something
  changed, so an idle window reports few frames.
- `clipboard.write(text)` and `clipboard.read()`, which return promises.
- `files`, `FileError` and their types. See [Network and files](io.md).
