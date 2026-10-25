<!--
  Two to five choices that belong together, joined in one pill. One segment is chosen (`value`), or
  with `multiple` any number are on (`values`). The arrow keys move between segments.
-->
<script lang="ts">
  import { Icon, Text, View, type KeyEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import checkIcon from "@phosphor-icons/core/bold/check-bold.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import type { SegmentedOption } from "./types";
  import { nextFocusIndex } from "./rovingFocus";
  import { toggleValue } from "./segmentedSelection";

  type Size = "sm" | "md";

  interface Props {
    options: SegmentedOption[];
    /** The chosen segment when only one may be chosen. */
    value?: string;
    /** The chosen segments when `multiple` is on. */
    values?: string[];
    /** Lets several segments be on at once, such as the days a calendar repeats on. */
    multiple?: boolean;
    size?: Size;
    /** Stretches the group to the width of its container, with equal segments. */
    full?: boolean;
    onchange?: (selection: string | string[]) => void;
  }

  let {
    options,
    value = $bindable(),
    values = $bindable([]),
    multiple = false,
    size = "md",
    full = false,
    onchange,
  }: Props = $props();

  const theme = $derived(currentTheme());
  let segments: ReturnType<typeof View>[] = $state([]);
  let focusedIndex: number | null = $state(null);
  let pointerIsDown = false;

  const disabledStates = $derived(options.map((option) => option.disabled === true));

  function isSelected(option: SegmentedOption): boolean {
    if (multiple) {
      return values.includes(option.value);
    }
    return value === option.value;
  }

  function segmentHeight(): number {
    if (size === "sm") {
      return 32;
    }
    return 40;
  }

  function segmentPaddingX(): number {
    if (size === "sm") {
      return 14;
    }
    return 20;
  }

  function iconSize(): number {
    if (size === "sm") {
      return 14;
    }
    return 16;
  }

  function labelType() {
    if (size === "sm") {
      return typeStyle("bodySmall");
    }
    return typeStyle("body");
  }

  function segmentBackground(isOn: boolean): string {
    if (isOn) {
      return theme.colors.fill;
    }
    return "transparent";
  }

  function segmentColor(isOn: boolean): string {
    if (isOn) {
      return theme.colors.onFill;
    }
    return theme.colors.text;
  }

  function segmentHover(isOn: boolean, option: SegmentedOption): Style {
    if (option.disabled) {
      return {};
    }
    if (isOn) {
      return { background: theme.colors.fillHover };
    }
    return { background: theme.colors.hover };
  }

  function segmentBorder(index: number): string {
    if (focusedIndex === index) {
      return theme.colors.focus;
    }
    return "transparent";
  }

  function segmentRadius(index: number): [number, number, number, number] {
    const left = endRadius(index === 0);
    const right = endRadius(index === options.length - 1);
    return [left, right, right, left];
  }

  function endRadius(isEnd: boolean): number {
    if (isEnd) {
      return 999;
    }
    return 0;
  }

  function groupSize() {
    if (full) {
      return { width: "full" as const };
    }
    return { alignSelf: "start" as const };
  }

  function segmentFill(): Style {
    if (full) {
      return { flex: 1 };
    }
    return {};
  }

  function iconFor(option: SegmentedOption, isOn: boolean): string | undefined {
    if (isOn && option.selectedIcon) {
      return option.selectedIcon;
    }
    if (option.icon) {
      return option.icon;
    }
    if (isOn) {
      return checkIcon;
    }
    return undefined;
  }

  function select(option: SegmentedOption): void {
    if (option.disabled) {
      return;
    }
    if (multiple) {
      values = toggleValue(values, option.value);
      onchange?.(values);
      return;
    }
    value = option.value;
    onchange?.(option.value);
  }

  function handleFocus(index: number): void {
    if (!pointerIsDown) {
      focusedIndex = index;
    }
  }

  function segmentOpacity(option: SegmentedOption): number {
    if (option.disabled) {
      return 0.4;
    }
    return 1;
  }

  function segmentCursor(option: SegmentedOption): "pointer" | "not-allowed" {
    if (option.disabled) {
      return "not-allowed";
    }
    return "pointer";
  }

  function handleKey(event: KeyEvent, index: number): void {
    if (event.key === " " || event.key === "Enter") {
      event.preventDefault();
      select(options[index]);
      return;
    }
    const target = nextFocusIndex(event.key, index, disabledStates, "horizontal");
    if (target === undefined) {
      return;
    }
    event.preventDefault();
    segments[target]?.focus();
    if (!multiple) {
      select(options[target]);
    }
  }
</script>

<View
  style={{
    direction: "row",
    radius: 999,
    borderWidth: 1,
    borderColor: theme.colors.lineStrong,
    ...groupSize(),
  }}
>
  {#each options as option, index (option.value)}
    {@const isOn = isSelected(option)}
    {@const segmentIcon = iconFor(option, isOn)}
    {#if index > 0}
      <View style={{ width: 1, alignSelf: "stretch", background: theme.colors.lineStrong }} />
    {/if}
    <View
      bind:this={segments[index]}
      focusable={!option.disabled}
      style={{
        direction: "row",
        align: "center",
        justify: "center",
        gap: 8,
        height: segmentHeight(),
        paddingX: segmentPaddingX(),
        borderWidth: 2,
        borderColor: segmentBorder(index),
        radius: segmentRadius(index),
        background: segmentBackground(isOn),
        color: segmentColor(isOn),
        cursor: segmentCursor(option),
        opacity: segmentOpacity(option),
        transition: 300,
        ...segmentFill(),
      }}
      hover={segmentHover(isOn, option)}
      onpress={() => select(option)}
      ondown={() => (pointerIsDown = true)}
      onup={() => (pointerIsDown = false)}
      onkey={(event) => handleKey(event, index)}
      onfocus={() => handleFocus(index)}
      onblur={() => (focusedIndex = null)}
    >
      {#if segmentIcon}
        <Icon svg={segmentIcon} size={iconSize()} color={segmentColor(isOn)} />
      {/if}
      {#if option.label}
        <Text style={{ ...labelType(), fontWeight: 500, color: segmentColor(isOn), nowrap: true }}>{option.label}</Text>
      {/if}
    </View>
  {/each}
</View>
