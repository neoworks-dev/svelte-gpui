<!--
  Text. Nested `Text` elements are runs of the same paragraph, each with its own
  style, and the paragraph wraps as one. Whitespace collapses unless `pre` is set.
  With `selectable`, the paragraph can be selected with the pointer and copied; set it on
  the outermost `Text`, which its nested runs are part of.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import { itself, listener, type Layout } from "./events";

  interface Props {
    style?: Style;
    /** Keep whitespace and line breaks as written. */
    pre?: boolean;
    /**
     * Let the pointer select this paragraph's text: drag to select, double-click for a word,
     * triple-click for the paragraph, and Ctrl+C (Cmd+C on macOS) to copy. Not inherited:
     * nested `Text` runs are already part of the paragraph, and a `Text` inside a box in it is a
     * paragraph of its own that needs its own.
     */
    selectable?: boolean;
    onlayout?: (layout: Layout) => void;
    children?: Snippet;
  }

  let props: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    pre: props.pre ? "" : undefined,
    selectable: props.selectable ? "" : undefined,
    onlayout: listener(props.onlayout, itself<Layout>),
  });
</script>

<g-text {...attributes}>{@render props.children?.()}</g-text>
