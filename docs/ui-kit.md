# The NeoWorks UI kit

`packages/svelte-gpui-ui` is the NeoWorks design system, ported to svelte-gpui. It is a
set of ready-made components (buttons, text fields, menus, dialogs, tabs, lists, date
pickers and more) and the NeoWorks theme in light and dark. Every component is built from
the [primitives](primitives.md) and drawn by the host like any other, so an app that uses
it needs nothing beyond the package.

Use it instead of building these controls by hand from `View` and `Text`. The components
already handle hover, press, keyboard focus, disabled states and the keyboard keys a control
is expected to answer to.

```svelte
<script lang="ts">
  import { View } from "@neoworks/svelte-gpui/ui";
  import { Button, currentTheme, TextField } from "@neoworks/svelte-gpui-ui";

  const theme = $derived(currentTheme());
  let name = $state("");
</script>

<View style={{ size: "full", background: theme.colors.canvas, padding: 24, gap: 16 }}>
  <TextField bind:value={name} label="Name" />
  <Button variant="primary" onpress={() => console.log(name)}>Save</Button>
</View>
```

## Contents

1. [Importing](#importing)
2. [The theme](#the-theme)
3. [What every component shares](#what-every-component-shares)
4. [Foundation](#foundation): Button, IconButton, Chip, Badge, Divider, Card, ListRow.
5. [Forms](#forms): TextField, Checkbox, Radio, RadioGroup, Switch, Slider, Search, SegmentedButton.
6. [Overlays](#overlays): Menu, Select, Tooltip, Dialog, BottomSheet, SideSheet, Snackbar.
7. [Navigation](#navigation): TopAppBar, NavBar, NavigationRail, NavigationDrawer, Tabs, Fab.
8. [Lists and data](#lists-and-data): List, ListItem, Table, SectionHeader, Disclosure, DisclosureGroup, StatusBadge, IconCircle.
9. [Pickers and feedback](#pickers-and-feedback): DatePicker, TimePicker, WheelColumn, ProgressIndicator, LoadingSpinner.
10. [What differs from the web version](#what-differs-from-the-web-version)
11. [Example apps](#example-apps)

## Importing

Everything comes from one entry point.

```ts
import { Button, currentTheme, Dialog, typeStyle } from "@neoworks/svelte-gpui-ui";
```

The package lists `@neoworks/svelte-gpui` as a peer dependency, and its components import the
primitives from `@neoworks/svelte-gpui/ui`. Several components share a name with a primitive:
`Button`, `Tooltip`, `Menu`, `Select` and `List` exist in both packages. Import the one you
want, and rename the other if you need both.

```ts
import { Text, View } from "@neoworks/svelte-gpui/ui";
import { Button } from "@neoworks/svelte-gpui-ui";
import { List as VirtualList } from "@neoworks/svelte-gpui/ui";
import { List } from "@neoworks/svelte-gpui-ui";
```

Types are exported next to the components: `ButtonVariant`, `NavItem`, `TableColumn`,
`MenuEntry`, `SelectOption`, `TabItem`, `SegmentedOption` and so on. The look functions
(`buttonLook`, `chipLook`, `cardLook`, `listRowLook`, `iconButtonLook`) are exported too, for
drawing something that should match a component without using it.

## The theme

The theme is plain data in four scopes: `light`, `dark`, and the two inverse scopes
`lightInverse` and `darkInverse`, which are the dark block on a light page and the light block
on a dark page. A theme has these parts:

- `colors` holds the named roles: `canvas`, `surface`, `sunken`, `hover`, `pressed`, `line`,
  `lineStrong`, `text`, `muted`, `faint`, `fill`, `fillHover`, `onFill`, `accent`,
  `accentHover`, `onAccent`, `focus`, `danger`, `dangerSoft`, `success`, `warning`, `info`
  and their soft variants, plus the named hues `lake`, `larch`, `rock` and `pine`.
- `scales` holds the twelve step scales `grey`, `lake`, `larch`, `rock` and `pine`, where steps
  1 and 2 are page backgrounds, 3 to 5 fills, 6 to 8 borders, 9 and 10 solid fills, and 11 and
  12 text.
- `shadows` holds `lift` and `overlay`, each a list of layers to put in a `shadow` style.
- `radii` holds `control` (10), `card` (16) and `media` (20).
- `typeScale` holds the type roles `display`, `statement`, `headline`, `heading`, `title`,
  `bodyLarge`, `body`, `bodySmall`, `nav`, `caption` and `label`.

### Reading and changing the mode

The window shows one mode at a time, which starts as light. `currentTheme()` returns the theme
for it. Reading it inside a component, a `$derived` or an `$effect` tracks the mode, so
everything drawn from it redraws when the mode changes.

| Function | |
|---|---|
| `currentTheme()` | The `Theme` for the current mode. |
| `themeMode()` | `"light"` or `"dark"`. |
| `setThemeMode(mode)` | Sets the mode. |
| `toggleThemeMode()` | Switches between light and dark. |
| `inverseTheme(theme)` | The inverse scope for a theme: `darkInverse` for a light page and `lightInverse` for a dark one. |
| `typeStyle(role, viewportWidth?)` | A `Style` with `fontFamily`, `fontSize`, `lineHeight` and `fontWeight` for a type role, to spread into a `Text`. |
| `resolveFontSize(role, viewportWidth?)` | Only the font size of a role, in pixels. |

The type scale is fluid on the web, a `clamp()` between a minimum and a maximum that follows the
viewport width. `typeStyle` resolves it against `viewportWidth`, which is 1440 when left out
(`referenceViewportWidth`). Pass `window.size.width` to follow the window. Letter spacing is
kept in the data but `typeStyle` leaves it out, because `Style` has no such property.

The components read `currentTheme()` themselves, so they take no theme prop and cannot be
shown in the inverse scope. `inverseTheme` is for your own views, such as a dark band on a
light page.

```svelte
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { Button, currentTheme, inverseTheme, themeMode, toggleThemeMode, typeStyle } from "@neoworks/svelte-gpui-ui";

  const theme = $derived(currentTheme());
  const band = $derived(inverseTheme(theme));
</script>

<View style={{ size: "full", background: theme.colors.canvas, padding: 24, gap: 16 }}>
  <Text style={{ ...typeStyle("heading"), color: theme.colors.text }}>Mode: {themeMode()}</Text>
  <Button onpress={toggleThemeMode}>Switch mode</Button>
  <View style={{ background: band.colors.canvas, radius: theme.radii.card, padding: 20 }}>
    <Text style={{ ...typeStyle("body"), color: band.colors.text }}>An inverse block</Text>
  </View>
</View>
```

### Regenerating the tokens

`src/tokens.generated.ts` is generated from the web design system's `tokens.css` and is never
edited by hand. The script turns the `oklch()` and `color-mix()` colours of the stylesheet into
the hex and `rgba()` strings the style parser reads. Run it from the package directory, with the
path of the stylesheet:

```
cd packages/svelte-gpui-ui
bun scripts/generate.mjs PATH/TO/packages/ui/src/styles/tokens.css
```

It prints a warning for each colour that falls outside the display gamut and had to be clipped.
`tests/color.test.ts` checks the conversion against the values Chromium resolves for every
colour in the stylesheet.

## What every component shares

**Icons are SVG strings.** Every `icon` prop takes SVG markup, not a component. Import
the icons from `@phosphor-icons/core` as `.svg` files, which the build loads as text, and pass
them on. Components that have an active or selected state take a second string for it, such as
`selectedIcon`, so a filled icon can replace the regular one.

```ts
import house from "@phosphor-icons/core/regular/house.svg";
import houseFill from "@phosphor-icons/core/fill/house-fill.svg";
```

**Press is `onpress`.** Pressing a component calls `onpress`, never `onclick`, as with the
primitives. Components that wrap a value report changes in the style of the control: `onchange`
for most, `oninput` for text, and `onChange` for `Select`, `DatePicker`, `TimePicker` and
`WheelColumn`. Values you would bind (`value`, `checked`, `open`) are `$bindable`, so `bind:value` works
as well as a callback. The tables below mark each bindable prop.

**`style` is for layout.** The `style` prop of a component takes margins, widths, `flex`,
`alignSelf` and the like. The look of the component comes from the theme, and `style` does not
restyle it. Components that have several parts, such as a `TextField`, apply `style` to the
outermost box.

**Looks live in `*-look.ts`.** Each component keeps its colours, sizes and state styles in a
pure function in a file next to it, such as `button-look.ts`, that takes the theme and the
props and returns `Style` objects. The component only wires them to a primitive. This keeps
the visual rules testable without drawing, and `tests/ui-looks.test.ts` does that.

**Keyboard behaviour is built in.** Controls take focus with Tab, answer Enter or Space as
buttons do, and move between their parts with the arrow keys where that is the convention:
radios, segments, tabs, menu items, select options, sliders and pickers.

**Layers float above the page.** `Menu`, `Select` and `Tooltip` are placed against their
trigger and never clipped by an ancestor. A modal `Dialog`, `BottomSheet` or `SideSheet` draws a
window-sized layer with a scrim, and `Snackbar` floats at the bottom of the window. All of them,
except the tooltip, are opened by an `open` prop that you bind.

## Foundation

### Button

A pill button. Its children are drawn in the variant's text colour.

| Prop | Type | Default | |
|---|---|---|---|
| `variant` | `"primary" \| "secondary" \| "ghost" \| "accent" \| "danger"` | `"secondary"` | `primary` is the one main action of a screen, `accent` confirms something good. |
| `size` | `"sm" \| "md" \| "lg"` | `"md"` | |
| `icon` | `string` | | SVG before the text. |
| `trailingIcon` | `string` | | SVG after the text. |
| `disabled` | `boolean` | `false` | |
| `full` | `boolean` | `false` | Fills the width of the parent. |
| `round` | `boolean` | `false` | A circle sized to the height, for an icon-only button. |
| `style` | `Style` | | Layout only. |
| `onpress` | `(event: PointerEvent) => void` | | |
| `children` | `Snippet` | | |

### IconButton

A round button with only an icon. With `selected` set it is a toggle. It shows `label` as a
tooltip, so the label is required even though it is not drawn on the button.

| Prop | Type | Default | |
|---|---|---|---|
| `icon` | `string` | required | SVG. |
| `label` | `string` | required | The tooltip text. |
| `variant` | `"standard" \| "filled" \| "tonal" \| "outlined"` | `"standard"` | |
| `size` | `"sm" \| "md" \| "lg"` | `"md"` | |
| `selected` | `boolean` | | `true` or `false` makes it a toggle that you flip in `onpress`; leave it out for a plain button. |
| `selectedIcon` | `string` | | Drawn while selected. |
| `disabled` | `boolean` | `false` | |
| `style` | `Style` | | Layout only. |
| `onpress` | `(event: PointerEvent) => void` | | |

### Chip

A small pill. `assist` starts an action, `filter` toggles a way to narrow a list, `input` is a
piece of text the person entered, and `suggestion` offers a reply or a query.

| Prop | Type | Default | |
|---|---|---|---|
| `variant` | `"assist" \| "filter" \| "input" \| "suggestion"` | `"assist"` | |
| `label` | `string` | required | |
| `icon` | `string` | | SVG before the label. An applied filter chip shows a check instead. |
| `trailingIcon` | `string` | | SVG after the label. |
| `selected` | `boolean` | `false` | Bindable. Filter chips only: whether the filter is applied. A press toggles it. |
| `elevated` | `boolean` | `false` | A shadow instead of an outline. |
| `disabled` | `boolean` | `false` | |
| `style` | `Style` | | Layout only. |
| `onpress` | `(event: PointerEvent) => void` | | |
| `onremove` | `() => void` | | Input chips only. Adds a remove button, also reached with Backspace and Delete. |

### Badge

A count or a dot. Given children, it sits on their top right corner; without them it stands
alone.

| Prop | Type | Default | |
|---|---|---|---|
| `count` | `number` | | Zero or nothing hides the badge, unless `dot` is set. |
| `max` | `number` | `99` | Counts above it show as "99+". |
| `dot` | `boolean` | `false` | A small dot without a number. |
| `tone` | `"danger" \| "neutral" \| "accent"` | `"danger"` | |
| `style` | `Style` | | Layout only. |
| `children` | `Snippet` | | What the badge is attached to. |

### Divider

A thin line. A vertical one stretches to the height of its row.

| Prop | Type | Default | |
|---|---|---|---|
| `orientation` | `"horizontal" \| "vertical"` | `"horizontal"` | |
| `inset` | `"none" \| "start" \| "both"` | `"none"` | `start` leaves 64 pixels free so the line starts at the text of a row with a leading icon. |
| `strong` | `boolean` | `false` | A darker line for larger areas. |
| `style` | `Style` | | Layout only. |

### Card

A container for a group of content. With `onpress` the whole card is a button, so put no other
buttons inside it. Use `actions` for buttons instead and leave `onpress` out.

| Prop | Type | Default | |
|---|---|---|---|
| `variant` | `"outlined" \| "filled" \| "elevated"` | `"outlined"` | |
| `padding` | `"none" \| "sm" \| "md" \| "lg"` | `"md"` | |
| `headline` | `string` | | |
| `subhead` | `string` | | A short line under the headline. |
| `supportingText` | `string` | | |
| `onpress` | `() => void` | | Makes the whole card a button. |
| `style` | `Style` | | Layout only. |
| `media` | `Snippet` | | Full-bleed image at the top, outside the padding. |
| `actions` | `Snippet` | | Buttons at the bottom right. |
| `children` | `Snippet` | | Shown after the text. |

### ListRow

A single outlined row, for a link card on a settings page. For several rows in a stack use
[List](#list) and [ListItem](#listitem).

| Prop | Type | Default | |
|---|---|---|---|
| `title` | `string` | required | |
| `subtitle` | `string` | | |
| `onpress` | `() => void` | | Makes the whole row a button. |
| `chevron` | `boolean` | `false` | A caret that says the row opens something. |
| `leading` | `Snippet` | | |
| `trailing` | `Snippet` | | |
| `style` | `Style` | | Layout only. |

```svelte
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { Badge, Button, Card, Chip, Divider, IconButton, ListRow } from "@neoworks/svelte-gpui-ui";
  import bell from "@phosphor-icons/core/regular/bell.svg";
  import bellFill from "@phosphor-icons/core/fill/bell-fill.svg";
  import plus from "@phosphor-icons/core/regular/plus.svg";

  let muted = $state(false);
  let favourites = $state(false);
</script>

<View style={{ gap: 12, padding: 24 }}>
  <View style={{ direction: "row", gap: 8, align: "center" }}>
    <Button variant="primary" icon={plus} onpress={() => console.log("new")}>New album</Button>
    <Button variant="ghost" size="sm" disabled>Skip</Button>
    <Badge count={3}>
      <IconButton icon={bell} selectedIcon={bellFill} label="Notifications" selected={muted} onpress={() => (muted = !muted)} />
    </Badge>
  </View>
  <View style={{ direction: "row", gap: 8 }}>
    <Chip variant="filter" label="Favourites" bind:selected={favourites} />
    <Chip variant="input" label="Berlin" onremove={() => console.log("removed")} />
  </View>
  <Divider inset="start" />
  <Card headline="Summer" subhead="2026-07-14" supportingText="48 photos" style={{ width: 280 }}>
    {#snippet actions()}
      <Button variant="ghost" size="sm">Share</Button>
    {/snippet}
  </Card>
  <ListRow title="Storage" subtitle="12.4 GB used" chevron onpress={() => console.log("open")} />
</View>
```

## Forms

### TextField

A text field with the label inside it. The label moves up and shrinks once the field has focus or
text. `filled` suits a page, `outlined` a surface that is already filled.

| Prop | Type | Default | |
|---|---|---|---|
| `value` | `string` | `""` | Bindable. |
| `label` | `string` | required | |
| `variant` | `"outlined" \| "filled"` | `"filled"` | |
| `type` | `"text" \| "password" \| "number" \| "email" \| "search" \| "tel" \| "url"` | `"text"` | `password` hides the text and `number` takes only digits. |
| `placeholder` | `string` | | Shown while the field is focused and empty. |
| `supportingText` | `string` | | Help under the field, in the danger colour while `error` is on. |
| `error` | `boolean` | `false` | |
| `icon` | `string` | | SVG at the start. |
| `trailingIcon` | `string` | | A decorative SVG at the end. |
| `trailing` | `Snippet` | | Interactive content at the end, such as a clear button. |
| `prefix` | `string` | | Fixed text before the value. |
| `suffix` | `string` | | Fixed text after the value. |
| `multiline` | `boolean` | `false` | |
| `rows` | `number` | `3` | |
| `maxlength` | `number` | | Shows a counter and stops input at this length. |
| `disabled` | `boolean` | `false` | |
| `readonly` | `boolean` | `false` | |
| `style` | `Style` | | Layout of the whole field. |
| `oninput` | `(value: string) => void` | | |
| `onsubmit` | `(value: string) => void` | | |
| `onfocus` | `() => void` | | |
| `onblur` | `() => void` | | |

### Checkbox

A choice that is on or off, or partly on with `indeterminate`. Pressing it, or Space, toggles it
and clears `indeterminate`.

| Prop | Type | Default | |
|---|---|---|---|
| `checked` | `boolean` | `false` | Bindable. |
| `indeterminate` | `boolean` | `false` | Bindable. |
| `label` | `string` | | |
| `supportingText` | `string` | | |
| `error` | `boolean` | `false` | |
| `disabled` | `boolean` | `false` | |
| `size` | `"sm" \| "md"` | `"md"` | |
| `inline` | `boolean` | `false` | Takes only the width of its content, on one line, so checkboxes can sit side by side. |
| `onchange` | `(checked: boolean) => void` | | |

### Radio and RadioGroup

A `RadioGroup` names a question and holds the `Radio` options that answer it. The group's
`value` is the chosen option's value. The arrow keys move the choice between options.

| RadioGroup prop | Type | Default | |
|---|---|---|---|
| `value` | `string` | | Bindable. `undefined` while nothing is chosen. |
| `label` | `string` | required | Names the question. |
| `supportingText` | `string` | | Help text, or the reason for the error. |
| `error` | `boolean` | `false` | |
| `disabled` | `boolean` | `false` | Disables every option. |
| `orientation` | `"vertical" \| "horizontal"` | `"vertical"` | |
| `style` | `Style` | | Layout of the whole group. |
| `onchange` | `(value: string) => void` | | |
| `children` | `Snippet` | required | The `Radio` options. |

| Radio prop | Type | Default | |
|---|---|---|---|
| `value` | `string` | required | What the group's value becomes when this option is chosen. |
| `label` | `string` | required | |
| `supportingText` | `string` | | |
| `disabled` | `boolean` | `false` | |

### Switch

A setting that is on or off and takes effect at once. Use a `Checkbox` when a form needs a Save
button.

| Prop | Type | Default | |
|---|---|---|---|
| `checked` | `boolean` | `false` | Bindable. |
| `label` | `string` | required | |
| `supportingText` | `string` | | |
| `hideLabel` | `boolean` | `false` | Leaves the label out of the layout, for rows that show the name elsewhere. |
| `showIcon` | `boolean` | `false` | Draws a check in the thumb while on. |
| `disabled` | `boolean` | `false` | |
| `inline` | `boolean` | `false` | Takes only the width of its content. |
| `onchange` | `(checked: boolean) => void` | | |

### Slider

Picks a number by dragging a thumb, or with the arrow keys, Page Up and Down, Home and End
(Shift moves ten steps). With `range` two thumbs pick a span, and the value lives in
`rangeValue`.

| Prop | Type | Default | |
|---|---|---|---|
| `value` | `number` | `0` | Bindable. Ignored when `range` is on. |
| `rangeValue` | `[number, number]` | `[0, 100]` | Bindable. Low and high when `range` is on. |
| `range` | `boolean` | `false` | |
| `min` | `number` | `0` | |
| `max` | `number` | `100` | |
| `step` | `number` | `1` | |
| `disabled` | `boolean` | `false` | |
| `showValue` | `boolean` | `false` | Shows the value above the thumb while it is dragged or focused. |
| `showTicks` | `boolean` | `false` | A mark at every step. |
| `formatValue` | `(value: number) => string` | | Turns a value into the text shown. |
| `icon` | `string` | | SVG before the track. |
| `oninput` | `(value: number) => void` | | Every change while dragging or pressing keys. |
| `onchange` | `(value: number) => void` | | Once, when a drag is released or a key is pressed. |
| `onrangeinput` | `(value: [number, number]) => void` | | |
| `onrangechange` | `(value: [number, number]) => void` | | |

### Search

A search bar. While it has focus, `suggestions` hang under it in a panel. The arrow keys move
through them, Enter chooses the highlighted one or submits the text, and Escape clears the text.

| Prop | Type | Default | |
|---|---|---|---|
| `value` | `string` | `""` | Bindable. |
| `placeholder` | `string` | `"Search"` | |
| `suggestions` | `SearchSuggestion[]` | `[]` | Each has `value`, and optionally `label`, `icon` and `supportingText`. |
| `heading` | `string` | | A small title above the suggestions. |
| `onsubmit` | `(value: string) => void` | | Enter without a highlighted suggestion. |
| `onselect` | `(suggestion: SearchSuggestion) => void` | | The field is filled with the suggestion's value first. |
| `leading` | `Snippet` | | Replaces the magnifier. |
| `trailing` | `Snippet` | | |
| `size` | `"md" \| "sm"` | `"md"` | `sm` fits a header row next to icon buttons. |
| `style` | `Style` | | Layout of the whole bar. |

### SegmentedButton

Two to five choices that belong together, joined in one pill. One segment is chosen (`value`), or
with `multiple` any number are on (`values`).

| Prop | Type | Default | |
|---|---|---|---|
| `options` | `SegmentedOption[]` | required | Each has `value`, and optionally `label`, `icon`, `selectedIcon` and `disabled`. A segment without a label is icon-only. |
| `value` | `string` | | Bindable. |
| `values` | `string[]` | `[]` | Bindable. Used when `multiple` is on. |
| `multiple` | `boolean` | `false` | |
| `size` | `"sm" \| "md"` | `"md"` | |
| `full` | `boolean` | `false` | Stretches to the container, with equal segments. |
| `onchange` | `(selection: string \| string[]) => void` | | |

```svelte
<script lang="ts">
  import { View } from "@neoworks/svelte-gpui/ui";
  import {
    Checkbox,
    Radio,
    RadioGroup,
    Search,
    SegmentedButton,
    Slider,
    Switch,
    TextField,
    type SearchSuggestion,
  } from "@neoworks/svelte-gpui-ui";

  let email = $state("");
  let query = $state("");
  let volume = $state(40);
  let span = $state<[number, number]>([20, 80]);
  let agreed = $state(false);
  let syncOnMobile = $state(true);
  let plan = $state("free");
  let view = $state("grid");
  const recent: SearchSuggestion[] = [{ value: "beach" }, { value: "birthday", supportingText: "12 photos" }];
</script>

<View style={{ gap: 16, padding: 24, width: 360 }}>
  <TextField bind:value={email} label="Email" type="email" supportingText="We never share it" maxlength={60} />
  <Search bind:value={query} suggestions={recent} heading="Recent" onsubmit={(text) => console.log(text)} />
  <Slider bind:value={volume} showValue formatValue={(value) => `${value}%`} />
  <Slider range bind:rangeValue={span} min={0} max={100} step={5} showTicks />
  <Checkbox bind:checked={agreed} label="I accept the terms" />
  <Switch bind:checked={syncOnMobile} label="Sync over mobile data" supportingText="Uses your data plan" />
  <RadioGroup bind:value={plan} label="Plan" orientation="horizontal">
    <Radio value="free" label="Free" />
    <Radio value="pro" label="Pro" supportingText="Billed monthly" />
  </RadioGroup>
  <SegmentedButton
    options={[{ value: "grid", label: "Grid" }, { value: "list", label: "List" }]}
    bind:value={view}
  />
</View>
```

## Overlays

### Menu

A menu of actions that opens under a trigger. Up and Down move the highlight, Home and End
jump, Enter or Space choose, and typing jumps to the next label that starts with it. Escape and
an outside press close it.

| Prop | Type | Default | |
|---|---|---|---|
| `open` | `boolean` | `false` | Bindable. |
| `items` | `MenuEntry[]` | required | A `MenuItem` (`label`, and optionally `icon`, `shortcut`, `disabled`, `danger`, `selected`, `onselect`), `{ type: "divider" }` or `{ type: "heading", label }`. |
| `trigger` | `Snippet<[MenuTriggerProps]>` | required | Receives `{ onpress, expanded }`. Give the `onpress` to a button. |
| `placement` | `FloatingPlacement` | `"bottom-start"` | A side (`"top"`, `"bottom"`, `"left"`, `"right"`), optionally followed by `-start` or `-end`. |

### Select

A dropdown. `surface` and `ghost` are compact pills for toolbars and rows, `outlined` and `filled`
are full-height form fields with the label inside. With `filter` the list gets a search box.

| Prop | Type | Default | |
|---|---|---|---|
| `value` | `string \| string[]` | `""` | Bindable. A string, or a list of strings when `multiple`. |
| `onChange` | `(value: SelectValue) => void` | | |
| `options` | `SelectOption[]` | required | Each has `value`, `label`, and optionally `icon`, `description` and `disabled`. |
| `label` | `string` | | |
| `placeholder` | `string` | `"Select"` | |
| `supportingText` | `string` | | Only shown by `outlined` and `filled`. |
| `error` | `boolean` | `false` | |
| `disabled` | `boolean` | `false` | |
| `multiple` | `boolean` | `false` | The list stays open on each pick. |
| `filter` | `(option: SelectOption, query: string) => boolean` | | Adds the search box. |
| `searchPlaceholder` | `string` | `"Search"` | |
| `size` | `"sm" \| "md"` | `"md"` | Height of the compact variants. |
| `variant` | `"surface" \| "ghost" \| "outlined" \| "filled"` | `"surface"` | |
| `iconOnly` | `boolean` | `false` | Shows only the chosen option's icon. |
| `style` | `Style` | | Layout of the whole select. |

### Tooltip

A small label that appears next to its content after the pointer has rested on it. Give `title`,
`content` or `actions` to make it a rich tooltip that stays open while the pointer is on it.
`IconButton` already wraps itself in one.

| Prop | Type | Default | |
|---|---|---|---|
| `text` | `string` | `""` | A few words. |
| `title` | `string` | | Makes it a rich tooltip. |
| `placement` | `FloatingPlacement` | `"top"` | Moves to the opposite side when there is no room. |
| `delay` | `number` | `300` | Milliseconds the pointer rests before it shows. |
| `disabled` | `boolean` | `false` | |
| `content` | `Snippet` | | Rich body that replaces `text`. |
| `actions` | `Snippet` | | Buttons at the bottom of a rich tooltip. |
| `anchorStyle` | `Style` | | Layout of the wrapper the pointer rests on. |
| `children` | `Snippet` | required | The trigger. |

### Dialog

A dialog over a dimmed page. `basic` is a small centred card for one decision, `fullscreen` fills
the window for a longer task. `actions` holds the buttons, at the bottom right of a basic dialog
and in the top bar of a fullscreen one. The children are the supporting text or the form.

| Prop | Type | Default | |
|---|---|---|---|
| `open` | `boolean` | `false` | Bindable. |
| `title` | `string` | required | |
| `icon` | `string` | | SVG above the title of a basic dialog. |
| `variant` | `"basic" \| "fullscreen"` | `"basic"` | |
| `dismissible` | `boolean` | `true` | Whether Escape and a press outside close it. |
| `modal` | `boolean` | `true` | `false` shows it in the page instead of over it. |
| `onclose` | `() => void` | | |
| `children` | `Snippet` | required | |
| `actions` | `Snippet` | | |

### BottomSheet

A sheet that rises from the bottom of the window. A modal one dims and blocks the page. A standard
one sits at the bottom of its parent and leaves the page usable. Dragging the handle down, or
pressing it, closes it.

| Prop | Type | Default | |
|---|---|---|---|
| `open` | `boolean` | `false` | Bindable. |
| `title` | `string` | required | |
| `hideTitle` | `boolean` | `false` | |
| `modal` | `boolean` | `true` | |
| `dragHandle` | `boolean` | `true` | Shows the grab bar. |
| `dismissible` | `boolean` | `true` | Whether Escape and a press outside close a modal sheet. |
| `onclose` | `() => void` | | |
| `children` | `Snippet` | required | |

### SideSheet

A panel attached to the left or right edge of the window, for details or a form that belongs to
the page. A standard one takes its own column next to the page content.

| Prop | Type | Default | |
|---|---|---|---|
| `open` | `boolean` | `false` | Bindable. |
| `title` | `string` | required | |
| `side` | `"start" \| "end"` | `"end"` | |
| `modal` | `boolean` | `true` | |
| `dismissible` | `boolean` | `true` | |
| `onclose` | `() => void` | | |
| `children` | `Snippet` | required | |
| `actions` | `Snippet` | | Buttons pinned to the bottom. |

### Snackbar

A short message at the bottom of the window about something that just happened, with at most one
action. It closes itself after `duration`, and stays open while the pointer is over it.

| Prop | Type | Default | |
|---|---|---|---|
| `open` | `boolean` | `false` | Bindable. |
| `message` | `string` | required | |
| `actionLabel` | `string` | | A word or two, such as "Undo". |
| `onaction` | `() => void` | | |
| `dismissible` | `boolean` | `false` | Adds a close button. |
| `duration` | `number` | `5000` | Milliseconds before it closes. 0 keeps it until dismissed. |
| `onclose` | `() => void` | | |

```svelte
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { BottomSheet, Button, Dialog, Menu, Select, SideSheet, Snackbar, Tooltip, type MenuEntry } from "@neoworks/svelte-gpui-ui";
  import trash from "@phosphor-icons/core/regular/trash.svg";
  import warning from "@phosphor-icons/core/regular/warning.svg";

  let menuOpen = $state(false);
  let confirmOpen = $state(false);
  let sheetOpen = $state(false);
  let detailsOpen = $state(false);
  let undoOpen = $state(false);
  let sort = $state("date");

  const items: MenuEntry[] = [
    { type: "heading", label: "Album" },
    { label: "Rename", shortcut: "F2", onselect: () => console.log("rename") },
    { type: "divider" },
    { label: "Delete", icon: trash, danger: true, onselect: () => (confirmOpen = true) },
  ];
</script>

<View style={{ gap: 12, padding: 24, align: "start" }}>
  <Menu bind:open={menuOpen} {items} placement="bottom-start">
    {#snippet trigger({ onpress })}
      <Button {onpress}>Actions</Button>
    {/snippet}
  </Menu>
  <Select
    bind:value={sort}
    label="Sort by"
    variant="outlined"
    options={[{ value: "date", label: "Date" }, { value: "name", label: "Name" }]}
    onChange={(value) => console.log(value)}
    style={{ width: 200 }}
  />
  <Tooltip text="Opens the album" placement="right">
    <Button onpress={() => (sheetOpen = true)}>Share</Button>
  </Tooltip>
  <Button onpress={() => (detailsOpen = true)}>Details</Button>
</View>

<Dialog bind:open={confirmOpen} title="Delete this album?" icon={warning}>
  The photos in it stay in your library.
  {#snippet actions()}
    <Button variant="ghost" onpress={() => (confirmOpen = false)}>Cancel</Button>
    <Button variant="primary" onpress={() => ((confirmOpen = false), (undoOpen = true))}>Delete</Button>
  {/snippet}
</Dialog>
<BottomSheet bind:open={sheetOpen} title="Share album"><Text>Copy link</Text></BottomSheet>
<SideSheet bind:open={detailsOpen} title="Event details" side="end">
  <Text>48 photos</Text>
  {#snippet actions()}<Button variant="primary" onpress={() => (detailsOpen = false)}>Done</Button>{/snippet}
</SideSheet>
<Snackbar bind:open={undoOpen} message="Album deleted" actionLabel="Undo" onaction={() => console.log("undo")} />
```

## Navigation

### TopAppBar

The bar at the top of a screen: a menu or back button at the start, the title, and up to three
actions at the end. `medium` and `large` show a bigger title under the buttons until the page
scrolls. Set `scrolled` once content has moved under the bar and it turns to glass.

| Prop | Type | Default | |
|---|---|---|---|
| `title` | `string` | required | |
| `variant` | `"small" \| "centered" \| "medium" \| "large"` | `"small"` | |
| `scrolled` | `boolean` | `false` | |
| `leading` | `Snippet` | | A menu or back `IconButton`. |
| `trailing` | `Snippet` | | Up to three actions, or one overflow `Menu`. |
| `style` | `Style` | | Layout only. |

### NavBar

The bottom bar of a phone-sized layout: three to five destinations of equal importance in a
floating pill.

| Prop | Type | Default | |
|---|---|---|---|
| `items` | `NavItem[]` | required | Each has `label`, `value`, `icon`, and optionally `selectedIcon` and `badge`. |
| `current` | `string` | required | The `value` of the active item. Sub-pages (`/library/2` under `/library`) keep it active. |
| `onselect` | `(value: string) => void` | | |
| `style` | `Style` | | Layout only. |

### NavigationRail

A narrow column of three to seven destinations for tablets and desktops.

| Prop | Type | Default | |
|---|---|---|---|
| `items` | `NavItem[]` | required | |
| `current` | `string` | required | |
| `onselect` | `(value: string) => void` | | |
| `header` | `Snippet` | | Above the destinations, such as a menu button or a `Fab`. |
| `footer` | `Snippet` | | Pinned to the bottom. |
| `style` | `Style` | | |

### NavigationDrawer

A column of destinations grouped in sections. Docked, it is a permanent column. With `modal` it
slides over the page with a scrim behind it, and a press on the scrim or Escape sets `open` to
false.

| Prop | Type | Default | |
|---|---|---|---|
| `sections` | `NavSection[]` | required | Each has `items` and an optional `heading`. An item has `label`, `value`, and optionally `icon`, `selectedIcon` and `badge`. |
| `current` | `string` | required | |
| `modal` | `boolean` | `false` | |
| `open` | `boolean` | `true` | Bindable. Whether a modal drawer is showing. |
| `title` | `string` | | |
| `header` | `Snippet` | | |
| `footer` | `Snippet` | | |
| `onselect` | `(value: string) => void` | | |
| `style` | `Style` | | |

### Tabs

A row of tabs that switch between views. `primary` tabs switch the main sections of a screen with
the icon above the label, and `secondary` tabs divide the content inside a section. The arrow
keys, Home and End move between tabs and choose them.

| Prop | Type | Default | |
|---|---|---|---|
| `tabs` | `TabItem[]` | required | Each has `id`, `label`, and optionally `icon`, `selectedIcon`, `count` and `disabled`. |
| `value` | `string` | `""` | Bindable. The `id` of the active tab. |
| `variant` | `"primary" \| "secondary"` | `"primary"` | |
| `fullWidth` | `boolean` | `false` | Spreads the tabs across the full width. |
| `panel` | `Snippet<[string]>` | | Content of the active tab, which receives its `id`. |
| `onchange` | `(id: string) => void` | | |

### Fab

The one main action of a screen, floating above the content. Collapse the `extended` form while
the page scrolls down to keep the content free.

| Prop | Type | Default | |
|---|---|---|---|
| `icon` | `string` | required | |
| `label` | `string` | required | Shown next to the icon when `extended`. |
| `variant` | `"primary" \| "surface" \| "accent"` | `"primary"` | |
| `size` | `"sm" \| "md" \| "lg"` | `"md"` | |
| `extended` | `boolean` | `false` | |
| `style` | `Style` | | Layout only, such as the fixed position in a corner. |
| `onpress` | `(event: PointerEvent) => void` | | |

```svelte
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import {
    Fab,
    IconButton,
    NavBar,
    NavigationDrawer,
    NavigationRail,
    Tabs,
    TopAppBar,
    type NavItem,
    type NavSection,
    type TabItem,
  } from "@neoworks/svelte-gpui-ui";
  import house from "@phosphor-icons/core/regular/house.svg";
  import houseFill from "@phosphor-icons/core/fill/house-fill.svg";
  import gear from "@phosphor-icons/core/regular/gear.svg";
  import list from "@phosphor-icons/core/regular/list.svg";
  import plus from "@phosphor-icons/core/regular/plus.svg";

  let current = $state("/home");
  let drawerOpen = $state(false);
  let tab = $state("all");

  const destinations: NavItem[] = [
    { label: "Home", value: "/home", icon: house, selectedIcon: houseFill },
    { label: "Settings", value: "/settings", icon: gear, badge: 2 },
  ];
  const sections: NavSection[] = [{ heading: "Library", items: [{ label: "Albums", value: "/albums" }] }];
  const tabs: TabItem[] = [{ id: "all", label: "All" }, { id: "shared", label: "Shared", count: 4 }];
</script>

<View style={{ size: "full" }}>
  <TopAppBar title="Photos" variant="medium">
    {#snippet leading()}<IconButton icon={list} label="Menu" onpress={() => (drawerOpen = true)} />{/snippet}
  </TopAppBar>
  <View style={{ direction: "row", flex: 1 }}>
    <NavigationRail items={destinations} {current} onselect={(value) => (current = value)} />
    <View style={{ flex: 1 }}>
      <Tabs {tabs} bind:value={tab} variant="secondary">
        {#snippet panel(id)}<Text>Showing {id}</Text>{/snippet}
      </Tabs>
    </View>
  </View>
  <NavBar items={destinations} {current} onselect={(value) => (current = value)} />
  <Fab icon={plus} label="New album" extended style={{ position: "absolute", right: 16, bottom: 88 }} />
  <NavigationDrawer {sections} {current} modal bind:open={drawerOpen} title="NeoWorks" onselect={(value) => (current = value)} />
</View>
```

## Lists and data

### List and ListItem

A column of `ListItem` rows. `card` puts them in an outlined surface, as on a settings page, and
`plain` leaves them on the page. A row with `onpress` is a button. For thousands of rows use the
virtualised `List` primitive instead, which is why this one needs a different import name in an
app that uses both.

| List prop | Type | Default | |
|---|---|---|---|
| `variant` | `"card" \| "plain"` | `"card"` | |
| `dividers` | `boolean` | `false` | A hairline between rows. |
| `style` | `Style` | | Layout only. |
| `children` | `Snippet` | required | The `ListItem` rows. |

| ListItem prop | Type | Default | |
|---|---|---|---|
| `headline` | `string` | required | The main line. |
| `overline` | `string` | | A small line above the headline. |
| `supportingText` | `string` | | |
| `lines` | `1 \| 2 \| 3` | `2` | How many lines the supporting text may use. |
| `icon` | `string` | | SVG in a soft circle. For avatars and images use `leading`. |
| `leading` | `Snippet` | | |
| `trailing` | `Snippet` | | Do not combine with `onpress`. |
| `trailingText` | `string` | | Short text at the end, such as a size or a time. |
| `chevron` | `boolean` | `false` | |
| `onpress` | `() => void` | | Makes the whole row a button. |
| `selected` | `boolean` | `false` | Marks the current row. |
| `disabled` | `boolean` | `false` | |

### Table

A table of text with a heading row and a line between rows.

| Prop | Type | Default | |
|---|---|---|---|
| `columns` | `TableColumn[]` | required | Each has `key`, `label`, and optionally `align` (`"start"` or `"end"`), `numeric` and `width` in pixels. |
| `rows` | `TableRow[]` | required | Objects with a string or number for each column `key`. |
| `caption` | `string` | | |
| `density` | `"comfortable" \| "compact"` | `"comfortable"` | |
| `rowHeader` | `boolean` | `false` | Makes the first column the label of each row. |
| `style` | `Style` | | Layout only. |

### SectionHeader

The heading of a section of a page, with a line saying what it holds and an optional action.

| Prop | Type | Default | |
|---|---|---|---|
| `title` | `string` | required | |
| `description` | `string` | | |
| `action` | `Snippet` | | A link or button at the end. |
| `style` | `Style` | | Layout only. |

### Disclosure and DisclosureGroup

A `Disclosure` is a title that opens and closes the content under it. A `DisclosureGroup` stacks
several: `plain` draws lines between them and `card` puts them in an outlined surface. With
`exclusive`, opening one closes the one that was open.

| Disclosure prop | Type | Default | |
|---|---|---|---|
| `title` | `string` | required | |
| `supportingText` | `string` | | |
| `open` | `boolean` | `false` | Bindable. |
| `icon` | `string` | | SVG in a soft circle. |
| `children` | `Snippet` | required | The content that opens and closes. |

| DisclosureGroup prop | Type | Default | |
|---|---|---|---|
| `variant` | `"plain" \| "card"` | `"plain"` | |
| `exclusive` | `boolean` | `false` | |
| `style` | `Style` | | Layout only. |
| `children` | `Snippet` | required | The `Disclosure` rows. |

### StatusBadge

A status in one or two words, in a soft colour. Add an `icon` or a `dot` so the meaning does not
rest on colour alone.

| Prop | Type | Default | |
|---|---|---|---|
| `tone` | `"success" \| "danger" \| "warning" \| "info" \| "neutral"` | `"neutral"` | The older names `green`, `red`, `amber`, `blue` and `violet` also work. |
| `icon` | `string` | | SVG before the text, best in bold. |
| `dot` | `boolean` | `false` | A coloured dot before the text. |
| `style` | `Style` | | Layout only. |
| `children` | `Snippet` | | The status text. |

### IconCircle

An icon on a soft tinted tile, to mark a row or a card by what it is about.

| Prop | Type | Default | |
|---|---|---|---|
| `icon` | `string` | required | SVG, best in bold. |
| `tone` | `StatusTone` | `"neutral"` | |
| `size` | `"sm" \| "md" \| "lg"` | `"md"` | |
| `style` | `Style` | | Layout only. |

```svelte
<script lang="ts">
  import { View } from "@neoworks/svelte-gpui/ui";
  import {
    Disclosure,
    DisclosureGroup,
    IconCircle,
    List,
    ListItem,
    SectionHeader,
    StatusBadge,
    Table,
    type TableColumn,
    type TableRow,
  } from "@neoworks/svelte-gpui-ui";
  import cloud from "@phosphor-icons/core/bold/cloud-bold.svg";
  import folder from "@phosphor-icons/core/regular/folder.svg";

  let selectedName = $state("Photos");
  const columns: TableColumn[] = [
    { key: "name", label: "Name" },
    { key: "size", label: "Size", numeric: true, width: 90 },
  ];
  const rows: TableRow[] = [{ name: "Photos", size: "12.4 GB" }, { name: "Documents", size: "1.1 GB" }];
</script>

<View style={{ gap: 16, padding: 24, width: 420 }}>
  <SectionHeader title="Storage" description="Where your files live" />
  <List dividers>
    {#each ["Photos", "Documents"] as name (name)}
      <ListItem
        headline={name}
        supportingText="Folder"
        icon={folder}
        chevron
        selected={selectedName === name}
        onpress={() => (selectedName = name)}
      />
    {/each}
    <ListItem headline="Backup" trailingText="3 min">
      {#snippet leading()}<IconCircle icon={cloud} tone="info" />{/snippet}
      {#snippet trailing()}<StatusBadge tone="success" dot>Done</StatusBadge>{/snippet}
    </ListItem>
  </List>
  <Table {columns} {rows} caption="Folders by size" rowHeader density="compact" />
  <DisclosureGroup exclusive variant="card">
    <Disclosure title="Can I share an album?">Yes, with a link.</Disclosure>
    <Disclosure title="How is it stored?" supportingText="Encrypted">On your devices first.</Disclosure>
  </DisclosureGroup>
</View>
```

## Pickers and feedback

### DatePicker

Chooses a day from a calendar. In the calendar, the arrows and Page keys move the day that has
focus, the title steps from days to months to years, and Escape closes it. The `value` and
`onChange` are not optional: the picker does not keep the date itself.

| Prop | Type | Default | |
|---|---|---|---|
| `value` | `Date \| null` | required | `null` for no day yet. |
| `onChange` | `(date: Date) => void` | required | |
| `variant` | `"title" \| "field"` | `"title"` | `title` is a heading-sized button for page headers, `field` is a form field you can also type in as mm/dd/yyyy. |
| `label` | `string` | | Text of the `title` button instead of the month and year. |
| `fieldLabel` | `string` | `"Date"` | The label inside the `field` variant. |
| `placeholder` | `string` | `"Select date"` | |
| `supportingText` | `string` | | Under the `field` variant. |
| `error` | `boolean` | `false` | |
| `weekStartsOn` | `0 \| 1` | `1` | 0 is Sunday, 1 is Monday. |
| `min` | `Date` | | Earliest day that can be chosen. |
| `max` | `Date` | | Latest day that can be chosen. |

### TimePicker

A time field that opens two wheels, hours and minutes, and on a twelve hour clock a switch for AM
and PM.

| Prop | Type | Default | |
|---|---|---|---|
| `value` | `string` | required | `HH:mm` on a 24 hour clock, however it is shown. |
| `onChange` | `(value: string) => void` | required | |
| `label` | `string` | `"Time"` | |
| `supportingText` | `string` | | |
| `error` | `boolean` | `false` | |
| `twelveHour` | `boolean` | `false` | |
| `minuteStep` | `number` | `1` | Spacing of the minutes wheel, such as 5 or 15. |

### WheelColumn

A column of numbers to turn like a dial, with the chosen one in the middle band. Turn it with the
mouse wheel, drag it, press a row, or use Up, Down, PageUp and PageDown. It is what `TimePicker`
is made of, and works alone for a number such as a year.

| Prop | Type | Default | |
|---|---|---|---|
| `values` | `number[]` | required | The numbers on the wheel, in order. |
| `value` | `number` | required | One of `values`. |
| `onChange` | `(value: number) => void` | required | |
| `wrap` | `boolean` | `true` | Loops endlessly. Turn off for a bounded range such as years. |
| `format` | `(value: number) => string` | | Without it, two digits with a leading zero. |
| `label` | `string` | `"Value"` | Not drawn, see below. |
| `style` | `Style` | | Layout only. |

### ProgressIndicator

Shows how far something has got. Give `value` for work that has a known end, and leave it out when
the duration is unknown, so the indicator keeps moving.

| Prop | Type | Default | |
|---|---|---|---|
| `variant` | `"linear" \| "circular"` | `"linear"` | |
| `value` | `number` | | Between 0 and `max`. Leave out for an indeterminate indicator. |
| `max` | `number` | `100` | |
| `label` | `string` | required | What is in progress. Not drawn, see below. |
| `tone` | `"default" \| "accent" \| "inherit"` | `"default"` | `inherit` draws in `color`. |
| `color` | `Color` | text colour | Used when `tone` is `"inherit"`. |
| `size` | `number` | `40` | Circular only: diameter in pixels. |
| `thickness` | `number` | `4` | Circular only: stroke width in viewBox units out of 48. |
| `style` | `Style` | | Layout only. |

### LoadingSpinner

A small circle that keeps turning while something loads and nothing is known about how long it
takes. For work with a known end use `ProgressIndicator` with a `value`.

| Prop | Type | Default | |
|---|---|---|---|
| `size` | `number` | `24` | Diameter in pixels. |
| `thickness` | `number` | `5` | Stroke width in viewBox units out of 48. |
| `label` | `string` | `"Loading"` | Not drawn, see below. |
| `color` | `Color` | text colour | |
| `style` | `Style` | | Layout only. |

The `label` of these three components is meant for assistive tools. The host has none yet, so the
label is accepted and not drawn.

```svelte
<script lang="ts">
  import { View } from "@neoworks/svelte-gpui/ui";
  import { DatePicker, LoadingSpinner, ProgressIndicator, TimePicker, WheelColumn } from "@neoworks/svelte-gpui-ui";

  let day = $state<Date | null>(null);
  let birthday = $state<Date | null>(null);
  let time = $state("09:30");
  let year = $state(2026);
  const years = Array.from({ length: 21 }, (_, index) => 2016 + index);
</script>

<View style={{ gap: 16, padding: 24, width: 320 }}>
  <DatePicker value={day} onChange={(date) => (day = date)} weekStartsOn={1} />
  <DatePicker
    variant="field"
    fieldLabel="Birthday"
    value={birthday}
    onChange={(date) => (birthday = date)}
    max={new Date()}
  />
  <TimePicker value={time} onChange={(next) => (time = next)} label="Starts" twelveHour minuteStep={5} />
  <WheelColumn values={years} value={year} onChange={(next) => (year = next)} wrap={false} format={String} />
  <ProgressIndicator value={42} label="Uploading 12 photos" tone="accent" />
  <ProgressIndicator variant="circular" label="Importing" />
  <LoadingSpinner size={20} />
</View>
```

## What differs from the web version

The components look like the web design system, drawn by a host that does less than a browser.
These are the differences you will notice.

- **No backdrop blur.** The glass panels of menus, popovers, the search suggestions and the
  scrolled top bar are the canvas colour at 96% instead of 80% over a blurred page, so they read
  well but lose the frosted look. This waits for [#58](https://github.com/neoworks-dev/svelte-gpui/issues/58).
- **Hard-edged shadows use a blur of 1.** The host draws no shadow with a blur of 0, which the
  theme's hairline rings and the focus rings need. The components use a blur of 1 instead, and the
  theme keeps the exact value. This waits for [#57](https://github.com/neoworks-dev/svelte-gpui/issues/57).
- **A focus style with `color` shrinks text.** A `focus` style that sets `color` makes the host
  drop the element's own font size and weight while it has focus. The pickers avoid it, and a
  ghost `Button` or a standard `IconButton` with a custom font size would show it. See
  [#60](https://github.com/neoworks-dev/svelte-gpui/issues/60).
- **No scale and no outline.** The web presses a solid fill down to 96% and draws focus as an
  outline. Here a pressed fill fades to 88% opacity, and focus is a ring drawn as a shadow or a
  border.
- **Transitions only cover colour and opacity.** The `Switch` thumb and the `TextField` label
  jump to their new place instead of sliding, and `Disclosure` opens without animating its
  height. Letter spacing, bezier easings and the web fonts are kept in the theme but not applied.
- **Text can wrap in a row that shrinks to fit.** In a row that is only as wide as its content,
  the host can lay text out narrower than it needs and wrap it. The components set `nowrap: true`
  where it matters, and your own `Text` inside such a row needs it too. `Checkbox` and `Switch`
  do this when `inline` is on.
- **Two and three line text has no ellipsis.** `ListItem` with `lines` set to 2 or 3 clips the
  supporting text without a trailing ellipsis.
- **Table columns share width by their longest text.** Rows are laid out one at a time, so a
  column cannot take the width of the widest cell. Give a column a `width` to fix it.
- **No keyboard trap and no exit animation in overlays.** A dialog or sheet does not keep Tab
  inside itself, closes without animating, and a tooltip does not show on keyboard focus.
- **`List` is two components.** The package's `List` is a simple column of `ListItem` rows. The
  runtime's `List` is virtualised for long data. Import them under different names when you need
  both.
- **No `href`.** There are no links. `NavBar`, `NavigationRail` and `NavigationDrawer` take a
  `value` per item and call `onselect`, and `current` is compared against the value, so a router
  of your own decides what each destination shows.
- **No `Intl`.** `DatePicker` shows English month and weekday names and reads dates as
  mm/dd/yyyy, because the runtime has no `Intl`. `TimePicker` takes its twelve hour clock from the
  `twelveHour` prop and not from the locale.
- **Not ported.** `Carousel`, `FloatingScrollbar`, `PegboardCanvas`, `Polaroid`, `ScrapItem` and
  the `scene` and `schema-diagram` folders of the web package are web only.

## Example apps

Each group has an example that shows every component of it in every state, in light and dark. The
button at the top right switches the mode. Build one with
`bun examples/ui-forms/build.mjs`, then run the host on `examples/ui-forms/dist/ui-forms.js`.

| Example | Shows |
|---|---|
| `examples/ui-theme` | The colours, scales, type scale, radii and shadows. |
| `examples/ui-foundation` | Button, IconButton, Chip, Badge, Divider, Card and ListRow. |
| `examples/ui-forms` | TextField, Checkbox, Radio, Switch, Slider, Search and SegmentedButton. |
| `examples/ui-overlays` | Menu, Select, Tooltip, Dialog, the sheets and Snackbar. |
| `examples/ui-navigation` | TopAppBar, NavBar, NavigationRail, NavigationDrawer, Tabs and Fab. |
| `examples/ui-lists` | List, Table, SectionHeader, Disclosure, StatusBadge and IconCircle. |
| `examples/ui-pickers` | DatePicker, TimePicker, WheelColumn, ProgressIndicator and LoadingSpinner. |
