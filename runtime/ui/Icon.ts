// An SVG icon, tinted with `color` or else the text colour around it. Pass the
// markup (`import gear from "./gear.svg"` gives it, with the build's text loader),
// or the path of a file.

import { encode, type Color, type Style } from "./style";
import { definePrimitive, nothing, put } from "./primitive";

export interface IconProps {
  svg?: string;
  src?: string;
  /** Width and height in pixels; the font size around it otherwise. */
  size?: number;
  color?: Color;
  style?: Style;
}

export default definePrimitive<IconProps, object>(
  {
    tag: "g-icon",
    attributes(props, element) {
      put(element, "svg", props.svg);
      put(element, "src", props.src);
      put(element, "look", encode({ size: props.size, color: props.color, ...props.style }));
    },
    listeners: [],
  },
  nothing,
);
