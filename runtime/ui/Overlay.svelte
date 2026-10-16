<!--
  Content drawn above everything else, placed against its parent's box. It takes no room
  where it sits, is not cut off by an ancestor that clips, and is drawn after the rest of
  the window, so nothing covers it. It is placed after layout, against the box the host has
  just measured: on `side` of the parent, lined up by `align`, `offset` pixels away, moved to
  the other side when there is too little room, and shifted back inside the window.

  `at` anchors it to a point in the window instead, such as where the pointer was. It draws nothing
  while it has no children. A `View` with extra manners; use `Popover` for the usual open and
  close behaviour.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import { itself, listener, type KeyEvent, type Layout, type OutsideEvent, type PointerEvent } from "./events";

  interface Props {
    style?: Style;
    hover?: Style;
    active?: Style;
    focus?: Style;
    /** Which edge of the parent it sits against. The default is `"bottom"`. */
    side?: "top" | "bottom" | "left" | "right";
    /** Where along that edge it lines up. The default is `"start"`. */
    align?: "start" | "center" | "end";
    /** The gap between the parent and the overlay, along `side`. */
    offset?: number;
    /** Slides it along the edge, toward the end. */
    crossOffset?: number;
    /** A point in the window to place it against, `[x, y]`, instead of the parent's box. */
    at?: [number, number];
    /** Moves to the opposite side when the chosen one has too little room. The default is on. */
    flip?: boolean;
    /** Slides back inside the window when it would stick out. The default is on. */
    shift?: boolean;
    /** The least distance kept from the window's edges. The default is 4. */
    margin?: number;
    /** At least as wide as the parent. */
    matchWidth?: boolean;
    /** Lets the pointer through to what is behind it, instead of blocking it. */
    passthrough?: boolean;
    /** Overlays with a higher layer are drawn over those with a lower one. */
    layer?: number;
    /** Takes keyboard focus when pressed, and when `focus()` is called. */
    focusable?: boolean;
    /** A press here does not take focus from the field that has it, nor give it to anything. */
    keepFocus?: boolean;
    /** A button went down outside it (and outside any overlay opened from inside it). */
    onoutside?: (event: OutsideEvent) => void;
    onpress?: (event: PointerEvent) => void;
    onhover?: (hovered: boolean) => void;
    onlayout?: (layout: Layout) => void;
    onkey?: (event: KeyEvent) => void;
    onfocus?: () => void;
    onblur?: () => void;
    children?: Snippet;
  }

  let props: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    hover: encode(props.hover),
    active: encode(props.active),
    focus: encode(props.focus),
    side: props.side,
    align: props.align,
    offset: props.offset === undefined ? undefined : String(props.offset),
    "cross-offset": props.crossOffset === undefined ? undefined : String(props.crossOffset),
    at: props.at === undefined ? undefined : `${props.at[0]},${props.at[1]}`,
    flip: props.flip === undefined ? undefined : String(props.flip),
    shift: props.shift === undefined ? undefined : String(props.shift),
    margin: props.margin === undefined ? undefined : String(props.margin),
    "match-width": props.matchWidth ? "" : undefined,
    passthrough: props.passthrough ? "" : undefined,
    layer: props.layer === undefined ? undefined : String(props.layer),
    focusable: props.focusable ? "" : undefined,
    keepfocus: props.keepFocus ? "" : undefined,
    onoutside: listener(props.onoutside, itself<OutsideEvent>),
    onpress: listener(props.onpress, itself<PointerEvent>),
    onhover: listener(props.onhover, (event: { hovered: boolean }) => event.hovered),
    onlayout: listener(props.onlayout, itself<Layout>),
    onkey: listener(props.onkey, itself<KeyEvent>),
    onfocus: listener(props.onfocus, () => undefined),
    onblur: listener(props.onblur, () => undefined),
  });

  let element: any;

  /** Gives it keyboard focus. It must be `focusable`. */
  export function focus(): void {
    element?.focus();
  }
</script>

<g-overlay bind:this={element} {...attributes}>{@render props.children?.()}</g-overlay>
