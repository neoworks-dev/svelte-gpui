<!--
  The shared frame of Checkbox and Radio: a 40 pixel state layer around an indicator, with a label
  and supporting text beside it. The whole row presses, and Space activates it while it has focus.
  The layer shows hover, press and keyboard focus.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View, type KeyEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";

  interface Props {
    label?: string;
    supportingText?: string;
    /** Draws the supporting text in the danger colour. */
    supportingError?: boolean;
    disabled?: boolean;
    /** Takes only the width of its content, on one line, so options can sit side by side in a row. */
    inline?: boolean;
    onpress: () => void;
    onkey?: (event: KeyEvent) => void;
    indicator: Snippet;
  }

  let {
    label,
    supportingText,
    supportingError = false,
    disabled = false,
    inline = false,
    onpress,
    onkey,
    indicator,
  }: Props = $props();

  const theme = $derived(currentTheme());

  let hovered = $state(false);
  let pressed = $state(false);
  let focusRing = $state(false);
  let pointerIsDown = false;
  let root: ReturnType<typeof View> | undefined = $state();

  export function focus(): void {
    root?.focus();
  }

  function layerBackground(): string {
    if (disabled || !hovered) {
      return "transparent";
    }
    if (pressed) {
      return theme.colors.pressed;
    }
    return theme.colors.hover;
  }

  function layerBorder(): string {
    if (focusRing) {
      return theme.colors.focus;
    }
    return "transparent";
  }

  function rowCursor(): "pointer" | "not-allowed" {
    if (disabled) {
      return "not-allowed";
    }
    return "pointer";
  }

  function rowOpacity(): number {
    if (disabled) {
      return 0.4;
    }
    return 1;
  }

  function rowFit(): Style {
    if (inline) {
      return { alignSelf: "start" };
    }
    return {};
  }

  function labelColumnFit(): Style {
    if (inline) {
      return {};
    }
    return { flex: 1, minWidth: 0 };
  }

  function supportingColor(): string {
    if (supportingError) {
      return theme.colors.danger;
    }
    return theme.colors.muted;
  }

  function handleDown(): void {
    pointerIsDown = true;
    pressed = true;
  }

  function handleUp(): void {
    pointerIsDown = false;
    pressed = false;
  }

  function handleFocus(): void {
    focusRing = !pointerIsDown;
  }

  function handlePress(): void {
    if (!disabled) {
      onpress();
    }
  }

  function handleKey(event: KeyEvent): void {
    if (disabled) {
      return;
    }
    if (event.key === " ") {
      event.preventDefault();
      onpress();
      return;
    }
    onkey?.(event);
  }
</script>

<View
  bind:this={root}
  focusable={!disabled}
  style={{ direction: "row", gap: 4, cursor: rowCursor(), opacity: rowOpacity(), ...rowFit() }}
  onpress={handlePress}
  onhover={(isHovered) => (hovered = isHovered)}
  ondown={handleDown}
  onup={handleUp}
  onkey={handleKey}
  onfocus={handleFocus}
  onblur={() => (focusRing = false)}
>
  <View
    style={{
      size: 40,
      radius: 20,
      align: "center",
      justify: "center",
      borderWidth: 2,
      borderColor: layerBorder(),
      background: layerBackground(),
      transition: 300,
    }}
  >
    {@render indicator()}
  </View>
  {#if label}
    <View style={{ paddingY: 10, ...labelColumnFit() }}>
      <Text style={{ ...typeStyle("body"), lineHeight: 20, color: theme.colors.text, nowrap: inline }}>{label}</Text>
      {#if supportingText}
        <Text style={{ ...typeStyle("bodySmall"), marginTop: 2, color: supportingColor(), nowrap: inline }}>{supportingText}</Text>
      {/if}
    </View>
  {/if}
</View>
