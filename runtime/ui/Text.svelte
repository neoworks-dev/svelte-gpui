<!--
  Text. Nested `Text` elements are runs of the same paragraph, each with its own
  style, and the paragraph wraps as one. Whitespace collapses unless `pre` is set.
-->
<script lang="ts">
  import { encode } from "./style";
  import { itself, listener, type Layout } from "./events";
  import type { TextProps } from "./props";

  let props: TextProps = $props();

  const attributes = $derived({
    look: encode(props.style),
    pre: props.pre ? "" : undefined,
    onlayout: listener(props.onlayout, itself<Layout>),
  });
</script>

<g-text {...attributes}>{@render props.children?.()}</g-text>
