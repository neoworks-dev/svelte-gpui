<!--
  A small pill. `assist` starts an action, `filter` toggles a way to narrow a list, `input` is a piece of text the
  person entered, `suggestion` offers a reply or query.
-->
<script lang="ts">
  import { Button as ButtonPrimitive, Icon, Text, View, type KeyEvent, type PointerEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { checkBold, xBold } from "../glyphs";
  import { currentTheme } from "../theme.svelte";
  import { wrapperAlignment } from "./look";
  import { chipIconSize, chipLook, chipRemoveLook, chipTrailingIconSize, type ChipVariant } from "./chip-look";

  interface Props {
    variant?: ChipVariant;
    label: string;
    /** SVG markup before the label. An applied filter chip shows a check instead. */
    icon?: string;
    /** SVG markup after the label, such as a caret for a filter chip that opens a menu. */
    trailingIcon?: string;
    /** Filter chips only: whether the filter is applied. */
    selected?: boolean;
    /** Lifts the chip above the page with a shadow instead of an outline. */
    elevated?: boolean;
    disabled?: boolean;
    /** Layout only, such as margins. */
    style?: Style;
    onpress?: (event: PointerEvent) => void;
    /** Input chips only: called by the remove button, and by Backspace and Delete. */
    onremove?: () => void;
  }

  let {
    variant = "assist",
    label,
    icon,
    trailingIcon,
    selected = $bindable(false),
    elevated = false,
    disabled = false,
    style,
    onpress,
    onremove,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const isFilter = $derived(variant === "filter");
  const removable = $derived(variant === "input" && onremove !== undefined);
  const showCheck = $derived(isFilter && selected);
  const leadingIcon = $derived.by(() => {
    if (showCheck) {
      return checkBold;
    }
    return icon;
  });
  const look = $derived(
    chipLook(theme, {
      applied: showCheck,
      elevated,
      hasLeading: leadingIcon !== undefined,
      hasTrailingIcon: trailingIcon !== undefined,
      removable,
    }),
  );
  const removeLook = $derived(chipRemoveLook(theme));

  function press(event: PointerEvent): void {
    if (isFilter) {
      selected = !selected;
    }
    onpress?.(event);
  }

  function removeOnBackspace(event: KeyEvent): void {
    if (!removable) {
      return;
    }
    if (event.key !== "Backspace" && event.key !== "Delete") {
      return;
    }
    event.preventDefault();
    onremove?.();
  }
</script>

<View style={{ direction: "row", ...wrapperAlignment(style), ...style }}>
  <ButtonPrimitive
    style={look.style}
    hover={look.hover}
    active={look.active}
    focus={look.focus}
    disabledStyle={look.disabledStyle}
    {disabled}
    onpress={press}
    onkey={removeOnBackspace}
  >
    {#if leadingIcon}
      <Icon svg={leadingIcon} size={chipIconSize} />
    {/if}
    <Text>{label}</Text>
    {#if trailingIcon}
      <Icon svg={trailingIcon} size={chipTrailingIconSize} />
    {/if}
  </ButtonPrimitive>
  {#if removable}
    <ButtonPrimitive
      style={removeLook.style}
      hover={removeLook.hover}
      active={removeLook.active}
      focus={removeLook.focus}
      disabledStyle={removeLook.disabledStyle}
      {disabled}
      onpress={onremove}
    >
      <Icon svg={xBold} size={chipTrailingIconSize} />
    </ButtonPrimitive>
  {/if}
</View>
