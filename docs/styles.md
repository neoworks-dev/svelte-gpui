# Styles

A style is a plain object passed to a primitive's `style` prop. Its type is `Style`
from `@neoworks/svelte-gpui/ui`, so editors complete property names and
`svelte-check` rejects unknown ones.

```svelte
<View style={{ direction: "row", align: "center", gap: 8, padding: [6, 12], radius: 6, background: "#27272a" }} />
```

The object crosses to the host as JSON, and the host parses each distinct style once.
Writing styles inline is cheap, and so is building them in `$derived`. A property the
host does not understand is ignored.

There is no CSS: no selectors, no cascade beyond the inherited text properties, no
`class` and no units in strings. Numbers are logical pixels.

## Value types

| Type | Accepts |
|---|---|
| `number` | Logical pixels. |
| `Size` | Pixels (`120`), a share of the parent (`"50%"`), `"full"` (the same as `"100%"`) or `"auto"`. |
| `Color` | `"#rgb"`, `"#rrggbb"`, `"#rrggbbaa"`, `"rgb(r, g, b)"`, `"rgba(r, g, b, a)"` or `"transparent"`. |
| `Sides<T>` | One value for every side, `[vertical, horizontal]`, or `[top, right, bottom, left]`. |
| `Align` | `"start"`, `"center"`, `"end"`, `"stretch"` or `"baseline"`. |

## Layout

Every primitive is a flex container. Layout follows flexbox as React Native and Yoga
do: children stretch across by default, and `flex: 1` grows from nothing.

| Property | Values | |
|---|---|---|
| `direction` | `"row"` \| `"column"` | `View` defaults to column. `Button` and `Input` default to row. |
| `wrap` | `boolean` | Wraps children onto more lines. |
| `flex` | `number` | Grows to fill what is left, from a basis of 0: `flex: 1` shares space equally. |
| `grow`, `shrink` | `number` | The separate flex factors. |
| `basis` | `Size` | |
| `align` | `Align` | Children across the main direction (`align-items`). |
| `alignSelf` | `Align` | This element across its parent's direction. |
| `justify` | `"start"` \| `"center"` \| `"end"` \| `"between"` \| `"around"` | Children along the main direction. |
| `gap` | `number` | Space between children. |
| `rowGap`, `columnGap` | `number` | |
| `padding` | `Sides<number>` | |
| `paddingX`, `paddingY` | `number` | Left and right, or top and bottom. |
| `paddingTop`, `paddingRight`, `paddingBottom`, `paddingLeft` | `number` | |
| `margin` | `Sides<Size>` | `"auto"` margins push an element over. |
| `marginX`, `marginY`, `marginTop`, `marginRight`, `marginBottom`, `marginLeft` | `Size` | |
| `display` | `"flex"` \| `"none"` | `"none"` removes it from layout and drawing, but keeps it mounted. |

### Size and position

| Property | Values | |
|---|---|---|
| `width`, `height` | `Size` | |
| `size` | `Size` | Width and height together. |
| `minWidth`, `minHeight`, `maxWidth`, `maxHeight` | `Size` | |
| `aspectRatio` | `number` | Width divided by height. |
| `position` | `"relative"` \| `"absolute"` | Absolute elements are placed against their parent and leave layout. |
| `inset` | `Sides<Size>` | `top`, `right`, `bottom` and `left` at once. `inset: 0` covers the parent. |
| `top`, `right`, `bottom`, `left` | `Size` | |
| `overflow` | `"visible"` \| `"hidden"` | `"hidden"` clips children, including to rounded corners. |
| `translate` | `[x, y]` | Moves the element after layout, in pixels. Use it for drags and offsets. |
| `zoom` | `number` | Multiplies every length in the element and everything inside it. See below. |

Later siblings draw over earlier ones; there is no `zIndex`. To put something on top,
place it last, usually with `position: "absolute"`.

A row of fixed-width items next to one that should take the rest:

```svelte
<View style={{ direction: "row", height: "full" }}>
  <View style={{ width: 220, shrink: 0 }}>Sidebar</View>
  <View style={{ flex: 1, minWidth: 0 }}>Content</View>
</View>
```

`minWidth: 0` lets a flexible child become narrower than its content, for example so
long text inside it can truncate instead of pushing the layout wider.

### Zoom

`zoom`, as in CSS, lays out the element and everything inside it with every length
multiplied: sizes, positions, `translate`, padding, margins, gaps, borders, radii,
shadows, font sizes and line heights. Shares of the parent stay shares, and zooms
inside a zoom multiply. Text is shaped at the zoomed size, so it stays sharp, and a
`Canvas` inside draws in its own units at the zoomed size.

A canvas the user zooms sets it once on the layer that holds the content, instead of
multiplying every child's sizes, so a zoom step changes one style:

```svelte
<View style={{ position: "absolute", left: panX, top: panY, zoom }}>
  {#each cards as card (card.id)}
    <Card style={{ position: "absolute", left: card.x, top: card.y, width: 240 }} />
  {/each}
</View>
```

Pointer positions and the sizes `onresize` reports stay in window pixels, zoom
included.

## Paint

| Property | Values | |
|---|---|---|
| `background` | `Color` | |
| `opacity` | `number` | 0 to 1, for the element and everything inside it. |
| `borderWidth` | `Sides<number>` | |
| `borderTopWidth`, `borderRightWidth`, `borderBottomWidth`, `borderLeftWidth` | `number` | |
| `borderColor` | `Color` | |
| `radius` | `number` \| `[topLeft, topRight, bottomRight, bottomLeft]` | Corner radius. |
| `shadow` | `Shadow` \| `Shadow[]` \| `null` | `{ x?, y?, blur?, spread?, color? }`. The colour defaults to black at 30%. |
| `cursor` | `"default"` \| `"pointer"` \| `"text"` \| `"grab"` \| `"grabbing"` \| `"crosshair"` \| `"col-resize"` \| `"row-resize"` \| `"nwse-resize"` \| `"nesw-resize"` \| `"not-allowed"` | The pointer's shape over the element. `nwse-resize` points from top left to bottom right and `nesw-resize` from top right to bottom left, for resizing from a corner. |

## Text

Text properties apply to the text inside an element, and children inherit them, so
setting `color` or `fontSize` on a `Button` styles its label.

| Property | Values | |
|---|---|---|
| `color` | `Color` | Also tints `Icon`s inside. |
| `fontSize` | `number` | The default is 14. |
| `fontWeight` | `number` \| `"normal"` \| `"medium"` \| `"semibold"` \| `"bold"` | 400, 500, 600 and 700. |
| `fontFamily` | `"sans"` \| `"mono"` \| an installed font's name | `"sans"` is Geist and `"mono"` is JetBrains Mono, both bundled with the host. Geist is the default. |
| `lineHeight` | `number` | In pixels. |
| `italic`, `underline`, `strikethrough`, `uppercase` | `boolean` | |
| `textAlign` | `"left"` \| `"center"` \| `"right"` | |
| `nowrap` | `boolean` | Keeps text on one line. |
| `overflowWrap` | `"normal"` \| `"anywhere"` | `"anywhere"` breaks a word longer than the line between letters, so a long URL or path wraps inside its box. By default a text is never narrower than its longest word, and such a word widens its box. |
| `truncate` | `boolean` | One line, cut off with an ellipsis. |
| `maxLines` | `number` | At most that many lines, the last cut off with an ellipsis. |
| `verticalAlign` | `number` | For an element inside a `Text`: how far it sits above the baseline, in pixels. Negative lowers it. Not inherited. |

The window starts with white-ish text (`#f4f4f5`) on a dark background (`#18181b`).
Set `background` on the root `View` to change it.

## States

`View`, `Button` and `Input` take styles for their states. Each is laid over `style`
while the state holds, and sets only what it names:

| Prop | Holds while | On |
|---|---|---|
| `hover` | The pointer is over the element. | `View`, `Button`, `Input` |
| `active` | A button is held down on it. | `View`, `Button` |
| `focus` | It has keyboard focus. | `View` (with `focusable`), `Button`, `Input` |
| `disabledStyle` | `disabled` is set. | `Button` |

```svelte
<Button
  style={{ paddingX: 12, paddingY: 6, radius: 6, background: "#27272a", color: "#a1a1aa", transition: 120 }}
  hover={{ background: "#3f3f46", color: "#f4f4f5" }}
  active={{ background: "#52525b" }}
  focus={{ borderWidth: 1, borderColor: "#8b5cf6" }}
>
  <Text>Save</Text>
</Button>
```

The host applies these states without asking the app, so hovering costs no JavaScript.
When the app needs to know, for example to show something else on hover, use
`onhover` and app state.

## Transitions

`transition` eases changes to `background`, `color`, `borderColor` and `opacity`. It
covers entering and leaving `hover`, and any change to `style` itself, such as a
selected row changing colour. `active` and `focus` apply at once.

It takes one of these:

| Value | |
|---|---|
| `number` | The duration in ms, with `ease`. |
| `{ duration?, delay?, easing? }` | `easing` is `"linear"`, `"ease"`, `"ease-in"`, `"ease-out"` or `"ease-in-out"`. The duration defaults to 150. |

Sizes, positions and `translate` are not eased by `transition`. Animate them with a
`Tween` or `Spring` instead; see [Motion](motion.md).

## Themes

A theme is plain data: colours and style objects, imported where they are used and
spread into each other. Nothing is global.

```ts
// theme.ts
import type { Style } from "@neoworks/svelte-gpui/ui";

export const colors = { surface: "#16161a", raised: "#1f1f24", text: "#f4f4f5", accent: "#8b5cf6" };

export const card: Style = { background: colors.surface, radius: 10, padding: 16, gap: 12 };
export const button: Style = { paddingX: 12, paddingY: 6, radius: 6, background: colors.raised, transition: 120 };
export const mono: Style = { fontFamily: "mono", fontSize: 12 };
```

```svelte
<View style={{ ...card, direction: "row" }}>
  <Button style={merge(button, primary && { background: colors.accent })}>…</Button>
</View>
```

`merge` from `@neoworks/svelte-gpui/ui` skips `false`, `null` and `undefined`, so
conditional pieces read naturally.
