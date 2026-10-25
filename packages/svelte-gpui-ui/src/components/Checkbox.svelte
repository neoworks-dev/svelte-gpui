<!--
  A choice that is on or off, or partly on (`indeterminate`) for a parent whose children are
  only partly chosen. Pressing it, or Space, toggles it and clears `indeterminate`.
-->
<script lang="ts">
  import { Icon, View } from "@neoworks/svelte-gpui/ui";
  import checkIcon from "@phosphor-icons/core/bold/check-bold.svg";
  import minusIcon from "@phosphor-icons/core/bold/minus-bold.svg";
  import { currentTheme } from "../theme.svelte";
  import ChoiceRow from "./ChoiceRow.svelte";

  type Size = "sm" | "md";

  interface Props {
    checked?: boolean;
    indeterminate?: boolean;
    /** Text beside the box. */
    label?: string;
    supportingText?: string;
    /** Marks the choice as needing attention, such as an unchecked terms box on submit. */
    error?: boolean;
    disabled?: boolean;
    size?: Size;
    /** Takes only the width of its content, on one line, so checkboxes can sit side by side in a row. */
    inline?: boolean;
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = $bindable(false),
    indeterminate = $bindable(false),
    label,
    supportingText,
    error = false,
    disabled = false,
    size = "md",
    inline = false,
    onchange,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const isMarked = $derived(checked || indeterminate);

  function boxSize(): number {
    if (size === "sm") {
      return 20;
    }
    return 24;
  }

  function iconSize(): number {
    if (size === "sm") {
      return 12;
    }
    return 14;
  }

  function boxBackground(): string {
    if (isMarked) {
      return theme.colors.fill;
    }
    return "transparent";
  }

  function boxBorder(): string {
    if (isMarked) {
      return "transparent";
    }
    if (error) {
      return theme.colors.danger;
    }
    return theme.colors.lineStrong;
  }

  function markIcon(): string {
    if (indeterminate) {
      return minusIcon;
    }
    return checkIcon;
  }

  function toggle(): void {
    checked = !checked;
    indeterminate = false;
    onchange?.(checked);
  }
</script>

<ChoiceRow {label} {supportingText} supportingError={error} {disabled} {inline} onpress={toggle}>
  {#snippet indicator()}
    <View
      style={{
        size: boxSize(),
        radius: theme.radii.control,
        align: "center",
        justify: "center",
        borderWidth: 2,
        borderColor: boxBorder(),
        background: boxBackground(),
        transition: 300,
      }}
    >
      {#if isMarked}
        <Icon svg={markIcon()} size={iconSize()} color={theme.colors.onFill} />
      {/if}
    </View>
  {/snippet}
</ChoiceRow>
