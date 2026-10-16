<!--
  Text. Nested `Text` elements are runs of the same paragraph, each with its own
  style, and the paragraph wraps as one. Whitespace collapses unless `pre` is set.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import { itself, listener, type Layout } from "./events";

  interface Props {
    style?: Style;
    /** Keep whitespace and line breaks as written. */
    pre?: boolean;
    onlayout?: (layout: Layout) => void;
    children?: Snippet;
  }

  let props: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    pre: props.pre ? "" : undefined,
    onlayout: listener(props.onlayout, itself<Layout>),
  });
</script>

<g-text {...attributes}>{@render props.children?.()}</g-text>
