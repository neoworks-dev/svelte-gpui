<!--
  A list of actions with keyboard control: Up and Down move the highlight, Home and End jump,
  Enter or Space choose it, and typing a letter jumps to the next label that starts with it. The
  pointer highlights what it is over and chooses on press. It takes focus when it appears so the
  keys work at once.

  Put it in a `Popover` for a menu under a button, or use `ContextMenu`. It does not close
  itself: `onclose` is called after a choice, which is where the popover is closed.

    <Menu items={[{ label: "Copy", shortcut: "Ctrl+C", onselect: copy }, { separator: true }, { label: "Delete", danger: true }]}
          onclose={close} />
-->
<script lang="ts">
  import { untrack } from "svelte";
  import Icon from "./Icon.svelte";
  import Scroll from "./Scroll.svelte";
  import Text from "./Text.svelte";
  import View from "./View.svelte";
  import { check } from "./icons";
  import { colors } from "./overlay-look";
  import type { MenuItem } from "./menu";
  import type { KeyEvent, Layout, ScrollPosition } from "./events";
  import type { Style } from "./style";

  interface Props {
    items: MenuItem[];
    /** Called with the item and its index after the item's own `onselect`. */
    onselect?: (item: MenuItem, index: number) => void;
    /** Called after any choice. Close the popover here. */
    onclose?: () => void;
    /** The item highlighted when it appears. The default is the first one that is not disabled. */
    initial?: number;
    /** Takes keyboard focus when it appears. The default is on. */
    autofocus?: boolean;
    /** Scrolls when the rows are taller than this. */
    maxHeight?: number;
    /** The width, in pixels. The default is as wide as the widest row, and at least the popover's. */
    width?: number;
    style?: Style;
    /** Laid over every row's own style. */
    itemStyle?: Style;
  }

  let { items, onselect, onclose, initial, autofocus = true, maxHeight, width, style, itemStyle }: Props = $props();

  const ROW = 28;
  const SEPARATOR = 9;

  const selectable = (index: number): boolean => !items[index].separator && !items[index].disabled;

  /** The next item that can be chosen, wrapping around; `step` is 1 for down and -1 for up. */
  function neighbour(step: 1 | -1): number {
    const count = items.length;
    const from = active < 0 ? (step === 1 ? -1 : 0) : active;
    for (let distance = 1; distance <= count; distance++) {
      const index = (from + step * distance + count * 2) % count;
      if (selectable(index)) {
        return index;
      }
    }
    return -1;
  }

  /** The first (`1`) or last (`-1`) item that can be chosen. */
  function end(step: 1 | -1): number {
    for (let index = step === 1 ? 0 : items.length - 1; index >= 0 && index < items.length; index += step) {
      if (selectable(index)) {
        return index;
      }
    }
    return -1;
  }

  let active = $state(untrack(() => initial ?? end(1)));
  let root: ReturnType<typeof View> | undefined = $state();
  let scroller: ReturnType<typeof Scroll> | undefined = $state();
  let top = 0;
  let viewport = 0;

  const reserveIconColumn = $derived(items.some((item) => item.icon !== undefined || item.checked !== undefined));

  $effect(() => {
    if (autofocus) {
      root?.focus();
    }
  });

  function offsetOf(index: number): number {
    let offset = 0;
    for (let at = 0; at < index; at++) {
      offset += items[at].separator ? SEPARATOR : ROW;
    }
    return offset;
  }

  /** Scrolls the highlighted row into view. */
  function reveal(index: number): void {
    if (maxHeight === undefined || viewport === 0) {
      return;
    }
    const start = offsetOf(index);
    const bottom = start + ROW;
    if (start < top) {
      scroller?.scrollTo(start);
    } else if (bottom > top + viewport) {
      scroller?.scrollTo(bottom - viewport);
    }
  }

  function move(index: number): void {
    if (index >= 0) {
      active = index;
      reveal(index);
    }
  }

  function choose(index: number): void {
    const item = items[index];
    if (item === undefined || item.separator || item.disabled) {
      return;
    }
    item.onselect?.();
    onselect?.(item, index);
    onclose?.();
  }

  let typed = "";
  let typedAt = 0;

  /** Jumps to the next enabled item whose label starts with what was typed. */
  function typeahead(character: string): void {
    const now = Date.now();
    typed = now - typedAt > 700 ? character : typed + character;
    typedAt = now;
    const wanted = typed.toLowerCase();
    for (let step = 1; step <= items.length; step++) {
      const index = (Math.max(active, 0) + (typed.length === 1 ? step : step - 1)) % items.length;
      const item = items[index];
      if (selectable(index) && (item.label ?? "").toLowerCase().startsWith(wanted)) {
        move(index);
        return;
      }
    }
  }

  function key(event: KeyEvent): void {
    switch (event.key) {
      case "ArrowDown":
        move(neighbour(1));
        break;
      case "ArrowUp":
        move(neighbour(-1));
        break;
      case "Home":
        move(end(1));
        break;
      case "End":
        move(end(-1));
        break;
      case "Enter":
      case " ":
        choose(active);
        break;
      default:
        if (event.key.length === 1 && !event.ctrl && !event.alt && !event.meta) {
          typeahead(event.key);
          break;
        }
        return;
    }
    event.stopPropagation();
  }

  const rootStyle: Style = $derived({
    padding: 4,
    gap: 0,
    ...(width === undefined ? { minWidth: 160 } : { width }),
    ...style,
  });
</script>

<View bind:this={root} focusable style={rootStyle} onkey={key}>
  <Scroll
    bind:this={scroller}
    style={maxHeight === undefined ? undefined : { maxHeight }}
    onscroll={(position: ScrollPosition) => (top = position.y)}
    onlayout={(layout: Layout) => (viewport = layout.height)}
  >
    {#each items as item, index}
      {#if item.separator}
        <View style={{ height: SEPARATOR, shrink: 0, justify: "center", paddingX: 4 }}>
          <View style={{ height: 1, background: colors.border }} />
        </View>
      {:else}
        {@const highlighted = index === active && !item.disabled}
        <View
          style={{
            direction: "row",
            align: "center",
            gap: 8,
            height: ROW,
            shrink: 0,
            paddingX: 8,
            radius: 5,
            cursor: "default",
            background: highlighted ? colors.highlight : "transparent",
            color: item.disabled ? "#6b6b74" : item.danger ? colors.danger : colors.text,
            ...itemStyle,
          }}
          onhover={(hovered: boolean) => hovered && !item.disabled && (active = index)}
          onpress={() => choose(index)}
        >
          {#if reserveIconColumn}
            <View style={{ width: 16, height: 16, shrink: 0, align: "center", justify: "center" }}>
              {#if item.icon}
                <Icon svg={item.icon} size={14} />
              {:else if item.checked}
                <Icon svg={check} size={14} />
              {/if}
            </View>
          {/if}
          <Text style={{ flex: 1, minWidth: 0, truncate: true }}>{item.label}</Text>
          {#if item.shortcut}
            <Text style={{ color: colors.dim, fontSize: 12, shrink: 0 }}>{item.shortcut}</Text>
          {/if}
        </View>
      {/if}
    {/each}
  </Scroll>
</View>
