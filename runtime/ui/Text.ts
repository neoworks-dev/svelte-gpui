// Text. Nested `Text` elements are runs of the same paragraph, each with its own
// style, and the paragraph wraps as one. Whitespace collapses unless `pre` is set.

import type { Snippet } from "svelte";
import { encode, type Style } from "./style";
import type { Layout } from "./events";
import { definePrimitive, flag, nothing, put } from "./primitive";

export interface TextProps {
  style?: Style;
  /** Keep whitespace and line breaks as written. */
  pre?: boolean;
  onlayout?: (layout: Layout) => void;
  children?: Snippet;
}

export default definePrimitive<TextProps, object>(
  {
    tag: "g-text",
    attributes(props, element) {
      put(element, "look", encode(props.style));
      put(element, "pre", flag(props.pre));
    },
    listeners: [{ event: "layout", prop: "onlayout" }],
  },
  nothing,
);
