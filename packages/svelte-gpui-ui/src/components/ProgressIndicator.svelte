<!--
  Shows how far something has got. Give `value` for work that has a known end; leave it out when the duration is
  unknown and the indicator keeps moving. `linear` suits a task on the page, `circular` compact spots such as buttons
  and cards. The circular one is drawn on a canvas, so its colour comes from `tone` or `color`, not from the text around it.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { cubicInOut, cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import { Canvas, View, type Color, type Layout, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import {
    linearBarLook,
    linearTrackLook,
    progressColor,
    ringLook,
    ringTrackLook,
    type ProgressTone,
  } from "./progress-look";
  import {
    arcDots,
    cycleFraction,
    indeterminateArcPercent,
    indeterminateBarLeft,
    indeterminateBarMilliseconds,
    indeterminateBarShare,
    indeterminateRingMilliseconds,
    percentOf,
    ringStrokeWidth,
  } from "./progressModel";

  type Variant = "linear" | "circular";

  interface Props {
    /** `linear` for a task with a known place on the page, `circular` for compact spots such as buttons and cards. */
    variant?: Variant;
    /** Progress between 0 and `max`. Leave out for an indeterminate indicator when the duration is unknown. */
    value?: number;
    max?: number;
    /** What is in progress, such as "Uploading 12 photos". For assistive tools, which the host does not have yet, so it is not drawn. */
    label: string;
    /** `accent` for work that ends well, such as finishing an import. `inherit` takes `color`. */
    tone?: ProgressTone;
    /** The colour `tone="inherit"` draws in, standing in for the web's surrounding text colour. The text colour when left out. */
    color?: Color;
    /** Circular only: diameter in pixels. */
    size?: number;
    /** Circular only: stroke width in viewBox units out of 48. */
    thickness?: number;
    /** Layout only, such as the width or margins. */
    style?: Style;
  }

  let {
    variant = "linear",
    value,
    max = 100,
    label,
    tone = "default",
    color,
    size = 40,
    thickness = 4,
    style,
  }: Props = $props();

  const fillMilliseconds = 500;

  const theme = $derived(currentTheme());
  const isIndeterminate = $derived(value === undefined);
  const percent = $derived(percentOf(value, max));
  const drawColor = $derived(progressColor(theme, tone, variant, color));
  const cycleMilliseconds = $derived.by(() => {
    if (variant === "linear") {
      return indeterminateBarMilliseconds;
    }
    return indeterminateRingMilliseconds;
  });

  const shownPercent = new Tween(untrack(() => percent), { duration: fillMilliseconds, easing: cubicOut });
  let cycle = $state(0);
  let trackWidth = $state(0);
  let canvas: ReturnType<typeof Canvas> | undefined = $state();

  $effect(() => {
    const target = percent;
    untrack(() => shownPercent.set(target));
  });

  $effect(() => {
    if (!isIndeterminate) {
      return;
    }
    const startedAt = performance.now();
    const period = cycleMilliseconds;
    let handle = requestAnimationFrame(function step() {
      cycle = cycleFraction(performance.now(), startedAt, period);
      handle = requestAnimationFrame(step);
    });
    return () => cancelAnimationFrame(handle);
  });

  $effect(() => {
    if (variant !== "circular" || canvas === undefined) {
      return;
    }
    const geometry = { size, thickness };
    let startDegrees = 0;
    let sweepPercent = shownPercent.current;
    if (isIndeterminate) {
      startDegrees = cycle * 360;
      sweepPercent = indeterminateArcPercent;
    }
    canvas.clear();
    for (const dot of arcDots(geometry, startDegrees, sweepPercent * 3.6)) {
      canvas.fillCircle(dot.x, dot.y, ringStrokeWidth(geometry) / 2, drawColor);
    }
  });

  function barStyle(): Style {
    if (isIndeterminate) {
      return {
        ...linearBarLook(drawColor),
        position: "absolute",
        top: 0,
        width: trackWidth * indeterminateBarShare,
        left: indeterminateBarLeft(cubicInOut(cycle), trackWidth),
      };
    }
    return { ...linearBarLook(drawColor), width: `${shownPercent.current}%` };
  }
</script>

{#if variant === "linear"}
  <View style={{ ...linearTrackLook(theme), ...style }} onlayout={(layout: Layout) => (trackWidth = layout.width)}>
    <View style={barStyle()} />
  </View>
{:else}
  <View style={{ ...ringLook(size), ...style }}>
    <View style={ringTrackLook(size, thickness, drawColor)} />
    <Canvas bind:this={canvas} style={{ position: "absolute", left: 0, top: 0, size }} />
  </View>
{/if}
