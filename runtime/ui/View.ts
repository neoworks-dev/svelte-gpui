// A box that lays out its children: a column unless `style.direction` says "row".
// `hover`, `active` and `focus` are styles laid over `style` while the pointer is
// over it, while it is pressed, and while it has focus.

import type { Snippet } from "svelte";
import { encode, type Style } from "./style";
import type { FileDragEvent, KeyEvent, Layout, PointerEvent, WheelEvent } from "./events";
import { definePrimitive, flag, focusable, put, type Focusable, type ListenerSpec } from "./primitive";

export interface ViewProps {
  style?: Style;
  hover?: Style;
  active?: Style;
  focus?: Style;
  /** Takes keyboard focus when pressed. */
  focusable?: boolean;
  /** A press here does not take focus from the field that has it, nor give it to anything. */
  keepFocus?: boolean;
  onpress?: (event: PointerEvent) => void;
  onhover?: (hovered: boolean) => void;
  /** A button went down here; `onmove` and `onup` follow wherever the pointer goes until release. */
  ondown?: (event: PointerEvent) => void;
  /** The buttons `ondown` takes. A press of another button passes to what is around the view. The default is all three. */
  buttons?: ("left" | "middle" | "right")[];
  onmove?: (event: PointerEvent) => void;
  onup?: (event: PointerEvent) => void;
  onwheel?: (event: WheelEvent) => void;
  onlayout?: (layout: Layout) => void;
  /** Like `onlayout`, but only when its width or height changed, not when it merely moved. */
  onresize?: (layout: Layout) => void;
  onkey?: (event: KeyEvent) => void;
  onfocus?: () => void;
  onblur?: () => void;
  /** Files dragged in from another program entered, left, moved over this, or were dropped on it. */
  ondragenter?: (event: FileDragEvent) => void;
  ondragleave?: (event: FileDragEvent) => void;
  ondragover?: (event: FileDragEvent) => void;
  ondrop?: (event: FileDragEvent) => void;
  children?: Snippet;
}

const listeners: ListenerSpec<ViewProps>[] = [
  { event: "press", prop: "onpress" },
  { event: "hover", prop: "onhover", argument: "hovered" },
  { event: "down", prop: "ondown" },
  { event: "move", prop: "onmove" },
  { event: "up", prop: "onup" },
  { event: "wheel", prop: "onwheel" },
  { event: "layout", prop: "onlayout" },
  { event: "resize", prop: "onresize" },
  { event: "key", prop: "onkey" },
  { event: "focus", prop: "onfocus", argument: "none" },
  { event: "blur", prop: "onblur", argument: "none" },
  { event: "dragenter", prop: "ondragenter" },
  { event: "dragleave", prop: "ondragleave" },
  { event: "dragover", prop: "ondragover" },
  { event: "drop", prop: "ondrop" },
];

export default definePrimitive<ViewProps, Focusable>(
  {
    tag: "g-view",
    attributes(props, element) {
      put(element, "look", encode(props.style));
      put(element, "hover", encode(props.hover));
      put(element, "active", encode(props.active));
      put(element, "focus", encode(props.focus));
      put(element, "focusable", flag(props.focusable));
      put(element, "keepfocus", flag(props.keepFocus));
      put(element, "buttons", props.buttons?.join(","));
    },
    listeners,
  },
  focusable,
);
