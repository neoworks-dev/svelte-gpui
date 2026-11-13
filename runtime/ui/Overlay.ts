// Content drawn above everything else, placed against its parent's box. It takes no room
// where it sits, is not cut off by an ancestor that clips, and is drawn after the rest of
// the window, so nothing covers it. It is placed after layout, against the box the host has
// just measured: on `side` of the parent, lined up by `align`, `offset` pixels away, moved to
// the other side when there is too little room, and shifted back inside the window.
//
// `at` anchors it to a point in the window instead, such as where the pointer was. It draws nothing
// while it has no children. A `View` with extra manners; use `Popover` for the usual open and
// close behaviour.

import type { Snippet } from "svelte";
import { encode, type Style } from "./style";
import type { KeyEvent, Layout, OutsideEvent, PointerEvent } from "./events";
import { definePrimitive, flag, focusable, put, type Focusable, type ListenerSpec } from "./primitive";

export interface OverlayProps {
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

const listeners: ListenerSpec<OverlayProps>[] = [
  { event: "outside", prop: "onoutside" },
  { event: "press", prop: "onpress" },
  { event: "hover", prop: "onhover", argument: "hovered" },
  { event: "layout", prop: "onlayout" },
  { event: "key", prop: "onkey" },
  { event: "focus", prop: "onfocus", argument: "none" },
  { event: "blur", prop: "onblur", argument: "none" },
];

/** A number as an attribute, or nothing when the prop is absent. */
function number(value: number | undefined): string | undefined {
  return value === undefined ? undefined : String(value);
}

export default definePrimitive<OverlayProps, Focusable>(
  {
    tag: "g-overlay",
    attributes(props, element) {
      put(element, "look", encode(props.style));
      put(element, "hover", encode(props.hover));
      put(element, "active", encode(props.active));
      put(element, "focus", encode(props.focus));
      put(element, "side", props.side);
      put(element, "align", props.align);
      put(element, "offset", number(props.offset));
      put(element, "cross-offset", number(props.crossOffset));
      put(element, "at", props.at === undefined ? undefined : `${props.at[0]},${props.at[1]}`);
      put(element, "flip", props.flip === undefined ? undefined : String(props.flip));
      put(element, "shift", props.shift === undefined ? undefined : String(props.shift));
      put(element, "margin", number(props.margin));
      put(element, "match-width", flag(props.matchWidth));
      put(element, "passthrough", flag(props.passthrough));
      put(element, "layer", number(props.layer));
      put(element, "focusable", flag(props.focusable));
      put(element, "keepfocus", flag(props.keepFocus));
    },
    listeners,
  },
  focusable,
);
