<!--
  A setting that is on or off and takes effect at once. Use a Checkbox when a form needs a Save
  button. Pressing it, or Space, toggles it.
-->
<script lang="ts">
  import { Icon, Text, View, type KeyEvent, type Shadow, type Style } from "@neoworks/svelte-gpui/ui";
  import checkIcon from "@phosphor-icons/core/bold/check-bold.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import { ring } from "./ring";

  interface Props {
    checked?: boolean;
    /** Names the setting, as in "Sync over mobile data". */
    label: string;
    /** A line under the label that explains what the setting does. */
    supportingText?: string;
    /** Leaves the label out of the layout, for rows that show the name elsewhere. */
    hideLabel?: boolean;
    /** Draws a check in the thumb while on, so state does not rely on colour alone. */
    showIcon?: boolean;
    disabled?: boolean;
    /** Takes only the width of its content, on one line, so switches can sit side by side in a row. */
    inline?: boolean;
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = $bindable(false),
    label,
    supportingText,
    hideLabel = false,
    showIcon = false,
    disabled = false,
    inline = false,
    onchange,
  }: Props = $props();

  const theme = $derived(currentTheme());
  let focusRing = $state(false);
  let pointerIsDown = false;

  function toggle(): void {
    if (disabled) {
      return;
    }
    checked = !checked;
    onchange?.(checked);
  }

  function handleKey(event: KeyEvent): void {
    if (event.key === " ") {
      event.preventDefault();
      toggle();
    }
  }

  function trackBackground(): string {
    if (checked) {
      return theme.colors.fill;
    }
    return theme.colors.sunken;
  }

  function trackBorder(): string {
    if (checked) {
      return "transparent";
    }
    return theme.colors.lineStrong;
  }

  function thumbBackground(): string {
    if (checked) {
      return theme.colors.onFill;
    }
    return theme.colors.muted;
  }

  function trackShadow(): Shadow | null {
    if (focusRing) {
      return ring(theme.colors.focus, 2);
    }
    return null;
  }

  function thumbOffset(): number {
    if (checked) {
      return 22;
    }
    return 4;
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
</script>

<View
  focusable={!disabled}
  style={{ direction: "row", gap: 12, cursor: rowCursor(), opacity: rowOpacity(), ...rowFit() }}
  onpress={toggle}
  ondown={() => (pointerIsDown = true)}
  onup={() => (pointerIsDown = false)}
  onkey={handleKey}
  onfocus={() => (focusRing = !pointerIsDown)}
  onblur={() => (focusRing = false)}
>
  <View style={{ paddingY: 2 }}>
    <View
      style={{
        width: 48,
        height: 28,
        radius: 14,
        direction: "row",
        align: "center",
        borderWidth: 1,
        borderColor: trackBorder(),
        shadow: trackShadow(),
        background: trackBackground(),
        transition: 300,
      }}
    >
      <View
        style={{
          size: 20,
          radius: 10,
          marginLeft: thumbOffset(),
          align: "center",
          justify: "center",
          background: thumbBackground(),
          shadow: theme.shadows.lift,
          transition: 300,
        }}
      >
        {#if showIcon && checked}
          <Icon svg={checkIcon} size={12} color={theme.colors.fill} />
        {/if}
      </View>
    </View>
  </View>
  {#if !hideLabel}
    <View style={{ paddingTop: 4, ...labelColumnFit() }}>
      <Text style={{ ...typeStyle("body"), lineHeight: 20, color: theme.colors.text, nowrap: inline }}>{label}</Text>
      {#if supportingText}
        <Text style={{ ...typeStyle("bodySmall"), marginTop: 2, color: theme.colors.muted, nowrap: inline }}>{supportingText}</Text>
      {/if}
    </View>
  {/if}
</View>
