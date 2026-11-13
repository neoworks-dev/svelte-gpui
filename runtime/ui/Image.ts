// An image from a file path, a `data:` URL, or an `http(s)` URL, which the host
// fetches and caches. It loads when it comes near the window and is decoded no
// larger than it is shown. Until then its box keeps the image's proportions.

import { encode, type Style } from "./style";
import { definePrimitive, nothing, put } from "./primitive";

export interface ImageProps {
  src: string;
  /** How the image fills its box. */
  fit?: "contain" | "cover" | "fill";
  style?: Style;
  onload?: () => void;
  onerror?: () => void;
}

export default definePrimitive<ImageProps, object>(
  {
    tag: "g-image",
    attributes(props, element) {
      put(element, "src", props.src);
      put(element, "fit", props.fit);
      put(element, "look", encode(props.style));
    },
    listeners: [
      { event: "load", prop: "onload", argument: "none" },
      { event: "error", prop: "onerror", argument: "none" },
    ],
  },
  nothing,
);
