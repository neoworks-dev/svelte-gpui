// Something to press: by the pointer, or with Enter or Space while it has focus.
// Its children sit in a row.

import type { Snippet } from "svelte";
import { encode, type Style } from "./style";
import type { KeyEvent, PointerEvent } from "./events";
import { definePrimitive, flag, focusable, put, type Focusable, type ListenerSpec } from "./primitive";

export interface ButtonProps {
  style?: Style;
  hover?: Style;
  active?: Style;
  focus?: Style;
  /** Laid over `style` while the button is disabled. */
  disabledStyle?: Style;
  disabled?: boolean;
  onpress?: (event: PointerEvent) => void;
  onhover?: (hovered: boolean) => void;
  onkey?: (event: KeyEvent) => void;
  onfocus?: () => void;
  onblur?: () => void;
  children?: Snippet;
}

const listeners: ListenerSpec<ButtonProps>[] = [
  { event: "press", prop: "onpress", unless: "disabled" },
  { event: "hover", prop: "onhover", argument: "hovered" },
  { event: "key", prop: "onkey" },
  { event: "focus", prop: "onfocus", argument: "none" },
  { event: "blur", prop: "onblur", argument: "none" },
];

export default definePrimitive<ButtonProps, Focusable>(
  {
    tag: "g-button",
    attributes(props, element) {
      const style = props.disabled ? { cursor: "default", ...props.style, ...props.disabledStyle } : props.style;
      put(element, "look", encode(style as Style | undefined));
      put(element, "hover", encode(props.hover));
      put(element, "active", encode(props.active));
      put(element, "focus", encode(props.focus));
      put(element, "disabled", flag(props.disabled));
    },
    listeners,
  },
  focusable,
);
