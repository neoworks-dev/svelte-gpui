<!--
  Picks a number by dragging a thumb along a track, or with the arrow keys, Page Up and Down, Home
  and End (Shift moves ten steps). With `range` two thumbs pick a span.
-->
<script lang="ts">
  import { Icon, Text, View, type PointerEvent, type KeyEvent, type Layout, type Shadow } from "@neoworks/svelte-gpui/ui";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import { withAlpha } from "./colorMix";
  import { ring } from "./ring";
  import {
    fractionOf,
    moveRangeThumb,
    nearestThumbIndex,
    tickFractions,
    valueAfterKey,
    valueAtOffset,
    type SliderScale,
  } from "./sliderMath";

  interface Props {
    /** Current value, between `min` and `max`. Ignored when `range` is on. */
    value?: number;
    /** Low and high value when `range` is on. */
    rangeValue?: [number, number];
    /** Two thumbs that select a span, such as a price. */
    range?: boolean;
    min?: number;
    max?: number;
    /** Granularity of pointer and keyboard changes. */
    step?: number;
    disabled?: boolean;
    /** Shows the value above the thumb while it is dragged or focused. */
    showValue?: boolean;
    /** Draws a mark at every step. Use with a small number of steps. */
    showTicks?: boolean;
    /** Turns a value into the text shown, such as "40 GB". */
    formatValue?: (value: number) => string;
    /** SVG markup of an icon before the track, such as a speaker for a volume slider. */
    icon?: string;
    /** Every change while dragging or pressing keys. */
    oninput?: (value: number) => void;
    /** Once when a drag is released or a key press is made. */
    onchange?: (value: number) => void;
    onrangeinput?: (value: [number, number]) => void;
    onrangechange?: (value: [number, number]) => void;
  }

  let {
    value = $bindable(0),
    rangeValue = $bindable([0, 100]),
    range = false,
    min = 0,
    max = 100,
    step = 1,
    disabled = false,
    showValue = false,
    showTicks = false,
    formatValue,
    icon,
    oninput,
    onchange,
    onrangeinput,
    onrangechange,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const sidePadding = 12;
  const thumbSize = 40;
  const scale: SliderScale = $derived({ minimum: min, maximum: max, step });

  let areaWidth = $state(0);
  let draggingIndex: number | null = $state(null);
  let focusedIndex: number | null = $state(null);
  let thumbs: ReturnType<typeof View>[] = $state([]);

  const thumbValues = $derived.by(() => {
    if (range) {
      return [rangeValue[0], rangeValue[1]];
    }
    return [value];
  });
  const activeStart = $derived.by(() => {
    if (range) {
      return fractionOf(scale, rangeValue[0]);
    }
    return 0;
  });
  const activeEnd = $derived(fractionOf(scale, thumbValues[thumbValues.length - 1]));
  const ticks = $derived.by(() => {
    if (showTicks) {
      return tickFractions(scale);
    }
    return [];
  });

  function textOf(current: number): string {
    if (formatValue) {
      return formatValue(current);
    }
    return String(current);
  }

  function percent(fraction: number): `${number}%` {
    return `${fraction * 100}%`;
  }

  function areaCursor(): "pointer" | "not-allowed" {
    if (disabled) {
      return "not-allowed";
    }
    return "pointer";
  }

  function rootOpacity(): number {
    if (disabled) {
      return 0.4;
    }
    return 1;
  }

  function tickColor(fraction: number): string {
    if (fraction >= activeStart && fraction <= activeEnd) {
      return theme.colors.onFill;
    }
    return theme.colors.faint;
  }

  function isEngaged(index: number): boolean {
    return draggingIndex === index || focusedIndex === index;
  }

  function haloBackground(index: number): string {
    if (draggingIndex === index || focusedIndex === index) {
      return withAlpha(theme.colors.text, 0.12);
    }
    return "transparent";
  }

  function knobSize(index: number): number {
    if (draggingIndex === index) {
      return 18;
    }
    return 20;
  }

  function knobShadow(index: number): Shadow[] {
    const lift = theme.shadows.lift;
    if (focusedIndex === index) {
      return [ring(theme.colors.focus, 2), ...lift];
    }
    return lift;
  }

  function setThumb(index: number, requested: number): void {
    if (!range) {
      updateSingle(requested);
      return;
    }
    const next = moveRangeThumb(rangeValue, index, requested);
    if (next[0] === rangeValue[0] && next[1] === rangeValue[1]) {
      return;
    }
    rangeValue = next;
    onrangeinput?.(next);
  }

  function updateSingle(requested: number): void {
    if (requested === value) {
      return;
    }
    value = requested;
    oninput?.(requested);
  }

  function commit(): void {
    if (range) {
      onrangechange?.([rangeValue[0], rangeValue[1]]);
      return;
    }
    onchange?.(value);
  }

  function valueAtPointer(event: PointerEvent): number {
    return valueAtOffset(scale, event.x - sidePadding, areaWidth - sidePadding * 2);
  }

  function handleDown(event: PointerEvent): void {
    if (disabled || event.button !== 0) {
      return;
    }
    const target = valueAtPointer(event);
    const index = nearestThumbIndex(target, range, rangeValue);
    draggingIndex = index;
    thumbs[index]?.focus();
    setThumb(index, target);
  }

  function handleMove(event: PointerEvent): void {
    if (draggingIndex === null) {
      return;
    }
    setThumb(draggingIndex, valueAtPointer(event));
  }

  function handleUp(): void {
    if (draggingIndex === null) {
      return;
    }
    draggingIndex = null;
    commit();
  }

  function handleKey(event: KeyEvent, index: number): void {
    if (disabled) {
      return;
    }
    const next = valueAfterKey(scale, event.key, thumbValues[index], event.shift);
    if (next === undefined) {
      return;
    }
    event.preventDefault();
    setThumb(index, next);
    commit();
  }

  function handleResize(layout: Layout): void {
    areaWidth = layout.width;
  }
</script>

<View style={{ direction: "row", align: "center", gap: 12, width: "full", opacity: rootOpacity() }}>
  {#if icon}
    <Icon svg={icon} size={20} color={theme.colors.muted} style={{ shrink: 0 }} />
  {/if}

  <View
    style={{ flex: 1, minWidth: 0, paddingX: sidePadding, cursor: areaCursor() }}
    ondown={handleDown}
    onmove={handleMove}
    onup={handleUp}
    onresize={handleResize}
  >
    <View style={{ height: thumbSize, justify: "center" }}>
      <View style={{ height: 4, width: "full", radius: 2, background: theme.colors.pressed }}>
        <View
          style={{
            position: "absolute",
            top: 0,
            height: 4,
            radius: 2,
            left: percent(activeStart),
            width: percent(activeEnd - activeStart),
            background: theme.colors.fill,
          }}
        />
        {#each ticks as tickFraction (tickFraction)}
          <View
            style={{
              position: "absolute",
              top: 0,
              size: 4,
              radius: 2,
              left: percent(tickFraction),
              marginLeft: -2,
              background: tickColor(tickFraction),
            }}
          />
        {/each}
      </View>

      {#each thumbValues as thumbValue, index (index)}
        <View
          bind:this={thumbs[index]}
          focusable={!disabled}
          style={{
            position: "absolute",
            top: 0,
            left: percent(fractionOf(scale, thumbValue)),
            marginLeft: -thumbSize / 2,
            size: thumbSize,
            radius: thumbSize / 2,
            align: "center",
            justify: "center",
            background: haloBackground(index),
            transition: 300,
          }}
          hover={{ background: withAlpha(theme.colors.text, 0.08) }}
          onfocus={() => (focusedIndex = index)}
          onblur={() => (focusedIndex = null)}
          onkey={(event) => handleKey(event, index)}
        >
          <View
            style={{
              size: knobSize(index),
              radius: knobSize(index) / 2,
              background: theme.colors.fill,
              shadow: knobShadow(index),
            }}
          />
          {#if showValue && isEngaged(index)}
            <View style={{ position: "absolute", bottom: "100%", left: -40, width: 120, align: "center", marginBottom: 4 }}>
              <View style={{ paddingX: 10, paddingY: 4, radius: 999, background: theme.colors.fill }}>
                <Text style={{ ...typeStyle("caption"), color: theme.colors.onFill, nowrap: true }}>{textOf(thumbValue)}</Text>
              </View>
            </View>
          {/if}
        </View>
      {/each}
    </View>
  </View>
</View>
