<!--
  One option of a RadioGroup. Pressing it, or Space, chooses it; the arrow keys move the choice
  to a neighbouring option.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import ChoiceRow from "./ChoiceRow.svelte";
  import { useRadioGroup } from "./radioGroupContext";

  interface Props {
    /** What the surrounding RadioGroup's value becomes when this option is chosen. */
    value: string;
    label: string;
    supportingText?: string;
    disabled?: boolean;
  }

  let { value, label, supportingText, disabled = false }: Props = $props();

  const theme = $derived(currentTheme());
  const group = useRadioGroup();
  const checked = $derived(group !== undefined && group.value === value);
  const isDisabled = $derived(disabled || (group !== undefined && group.disabled));
  const hasError = $derived(group !== undefined && group.error);
  const isInline = $derived(group !== undefined && group.inline);
  let row: ReturnType<typeof ChoiceRow> | undefined = $state();

  // Registered once, so the group keeps the options in the order they were drawn.
  $effect(() =>
    untrack(() =>
      group?.register({
        get value() {
          return value;
        },
        isDisabled: () => isDisabled,
        focus: () => row?.focus(),
      }),
    ),
  );

  function ringColor(): string {
    if (checked) {
      return theme.colors.fill;
    }
    if (hasError) {
      return theme.colors.danger;
    }
    return theme.colors.lineStrong;
  }

  function dotOpacity(): number {
    if (checked) {
      return 1;
    }
    return 0;
  }

  function choose(): void {
    group?.select(value);
  }

  function handleKey(event: KeyEvent): void {
    if (group?.navigate(value, event.key)) {
      event.preventDefault();
    }
  }
</script>

<ChoiceRow bind:this={row} {label} {supportingText} disabled={isDisabled} inline={isInline} onpress={choose} onkey={handleKey}>
  {#snippet indicator()}
    <View
      style={{
        size: 24,
        radius: 12,
        align: "center",
        justify: "center",
        borderWidth: 2,
        borderColor: ringColor(),
        transition: 300,
      }}
    >
      <View style={{ size: 12, radius: 6, background: theme.colors.fill, opacity: dotOpacity(), transition: 300 }} />
    </View>
  {/snippet}
</ChoiceRow>
