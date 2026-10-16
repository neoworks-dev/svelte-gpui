<!--
  A view whose content scrolls: vertically, or sideways when `horizontal` is set.
  Give it a bounded size, such as `flex: 1` inside a column, or it grows to fit.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import { itself, listener, type Layout, type ScrollPosition } from "./events";

  interface Props {
    style?: Style;
    horizontal?: boolean;
    onscroll?: (position: ScrollPosition) => void;
    onlayout?: (layout: Layout) => void;
    children?: Snippet;
  }

  let props: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    horizontal: props.horizontal ? "" : undefined,
    onscroll: listener(props.onscroll, itself<ScrollPosition>),
    onlayout: listener(props.onlayout, itself<Layout>),
  });

  let element: any;

  /** Scrolls so that `y` (or `x` when horizontal) is at the top (or left) edge. */
  export function scrollTo(offset: number): void {
    if (props.horizontal) {
      element?.scrollTo(offset, 0);
    } else {
      element?.scrollTo(0, offset);
    }
  }
</script>

<g-scroll bind:this={element} {...attributes}>{@render props.children?.()}</g-scroll>
