<!--
  Text. Nested `Text` elements are runs of the same paragraph, each with its own style, and the
  paragraph wraps as one. Whitespace collapses unless `pre` is set.
-->
<script lang="ts">
  import type { TextProps } from "../../ui/props";
  import { cssText, toCss } from "./css";
  import { watchLayout } from "./layout";

  let props: TextProps = $props();

  const css = $derived.by(() => {
    const css = toCss(props.style);
    if (props.pre) {
      // Spaces and line breaks as written, wrapping unless `nowrap` says not to.
      css["white-space"] = css["white-space"] === "nowrap" ? "pre" : "pre-wrap";
    }
    return cssText(css);
  });
</script>

<span
  class="g-text"
  style={css}
  {@attach (node) => (props.onlayout ? watchLayout(node, (layout) => props.onlayout?.(layout)) : undefined)}
>{@render props.children?.()}</span>
