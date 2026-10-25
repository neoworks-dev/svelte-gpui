<!--
  A column of numbers to turn like a dial, with the chosen one in the middle band. Turn it with the mouse wheel, drag
  it, press a row, or use Up, Down, PageUp and PageDown while it has focus. With `wrap` the column loops without end,
  as hours and minutes do; without it the column stops at its first and last value, as a range of years does.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import { Text, View, type KeyEvent, type PointerEvent, type Style, type WheelEvent } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { createFocusSource } from "./focusSource";
  import { wheelColumnLook, wheelItemHover, wheelItemLook, wheelSelectionLook } from "./wheel-look";
  import {
    clampIndex,
    dragThreshold,
    indexAfterDrag,
    indexAfterTurn,
    rowTop,
    targetIndexFor,
    turnAfterWheel,
    valueAt,
    visibleIndexes,
  } from "./wheelModel";

  interface Props {
    /** The numbers on the wheel, in order. */
    values: number[];
    /** The chosen number, one of `values`. */
    value: number;
    onChange: (value: number) => void;
    /** Loops endlessly, as hours and minutes do. Turn off for a bounded range such as years. */
    wrap?: boolean;
    /** Turns a number into its text. Without it the number is shown with two digits and a leading zero. */
    format?: (value: number) => string;
    /** Names the column for assistive tools, which the host does not have yet, so it is not drawn. */
    label?: string;
    /** Layout only, such as the width. */
    style?: Style;
  }

  let { values, value, onChange, wrap = true, format, label = "Value", style }: Props = $props();

  const turnMilliseconds = 140;
  const wheelIdleMilliseconds = 160;
  const keyTurns: Record<string, number> = { ArrowUp: -1, ArrowDown: 1, PageUp: -5, PageDown: 5 };

  const theme = $derived(currentTheme());
  const focusSource = createFocusSource();

  // The centred row as a whole number that keeps counting past the ends of a wrapping column.
  let targetIndex = $state(untrack(() => Math.max(values.indexOf(value), 0)));
  const position = new Tween(untrack(() => targetIndex), { duration: turnMilliseconds, easing: cubicOut });
  let focused = $state(false);
  let dragStartY = 0;
  let dragStartPosition = 0;
  let dragDistance = 0;
  let pointerIsDown = false;
  let wheelAccumulated = 0;
  let lastWheelAt = 0;

  const indexes = $derived(visibleIndexes(position.current, values.length, wrap));

  $effect(() => {
    const valueIndex = values.indexOf(value);
    if (valueIndex === -1) {
      return;
    }
    untrack(() => {
      if (valueAt(values, targetIndex) === value) {
        return;
      }
      targetIndex = targetIndexFor(targetIndex, valueIndex, values.length, wrap);
      position.set(targetIndex);
    });
  });

  function textOf(item: number): string {
    if (format !== undefined) {
      return format(item);
    }
    return item.toString().padStart(2, "0");
  }

  function settleAt(index: number): void {
    let next = index;
    if (!wrap) {
      next = clampIndex(index, values.length);
    }
    targetIndex = next;
    position.set(next);
    const chosen = valueAt(values, next);
    if (chosen !== value) {
      onChange(chosen);
    }
  }

  function turnBy(steps: number): void {
    settleAt(indexAfterTurn(targetIndex, steps, values.length, wrap));
  }

  function handleKey(event: KeyEvent): void {
    const steps = keyTurns[event.key];
    if (steps === undefined) {
      return;
    }
    event.preventDefault();
    turnBy(steps);
  }

  function handleWheel(event: WheelEvent): void {
    const now = performance.now();
    const turn = turnAfterWheel(wheelAccumulated, event.dy, now - lastWheelAt > wheelIdleMilliseconds);
    lastWheelAt = now;
    wheelAccumulated = turn.accumulated;
    if (turn.steps !== 0) {
      turnBy(turn.steps);
    }
  }

  function handleDown(event: PointerEvent): void {
    focusSource.pointerDown();
    pointerIsDown = true;
    dragStartY = event.windowY;
    dragStartPosition = position.current;
    dragDistance = 0;
  }

  function followPointer(event: PointerEvent): void {
    if (!pointerIsDown) {
      return;
    }
    const distance = event.windowY - dragStartY;
    dragDistance = Math.max(dragDistance, Math.abs(distance));
    if (dragDistance < dragThreshold) {
      return;
    }
    position.set(indexAfterDrag(dragStartPosition, distance), { duration: 0 });
  }

  function handleUp(): void {
    if (!pointerIsDown) {
      return;
    }
    pointerIsDown = false;
    if (dragDistance >= dragThreshold) {
      settleAt(Math.round(position.current));
    }
  }

  function handleFocus(): void {
    focused = focusSource.focused();
  }

  function handleBlur(): void {
    focused = false;
    focusSource.blurred();
  }

  function pressRow(index: number): void {
    if (dragDistance >= dragThreshold) {
      return;
    }
    settleAt(index);
  }
</script>

<View
  focusable
  style={{ ...wheelColumnLook(theme), ...style }}
  onkey={handleKey}
  onwheel={handleWheel}
  ondown={handleDown}
  onmove={followPointer}
  onup={handleUp}
  onfocus={handleFocus}
  onblur={handleBlur}
>
  <View style={wheelSelectionLook(theme, focused)} />
  {#each indexes as index (index)}
    {@const selected = index === targetIndex}
    <View
      style={wheelItemLook(theme, rowTop(index, position.current), selected)}
      hover={wheelItemHover(theme, selected)}
      onpress={() => pressRow(index)}
    >
      <Text>{textOf(valueAt(values, index))}</Text>
    </View>
  {/each}
</View>
