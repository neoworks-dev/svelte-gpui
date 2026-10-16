<!--
  A long list that only builds the rows near the window. Rows have one fixed
  height, so where each sits is known without measuring it. Scrolling swaps rows
  in and out; a list of a hundred thousand items costs what a screenful does.
-->
<script lang="ts" generics="T">
  import type { Snippet } from "svelte";
  import Scroll from "./Scroll.svelte";
  import View from "./View.svelte";
  import type { Style } from "./style";

  interface Props {
    items: T[];
    /** The height of every row, in pixels. */
    rowHeight: number;
    row: Snippet<[T, number]>;
    /** Keeps a row's state with its item as the list changes. */
    key?: (item: T, index: number) => unknown;
    /** How many rows past each edge of the window are built anyway. */
    overscan?: number;
    style?: Style;
  }

  let { items, rowHeight, row, key, overscan = 8, style }: Props = $props();

  let top = $state(0);
  let height = $state(0);

  const first = $derived(Math.max(0, Math.floor(top / rowHeight) - overscan));
  const last = $derived(Math.min(items.length, Math.ceil((top + Math.max(height, rowHeight * 20)) / rowHeight) + overscan));
  const shown = $derived(items.slice(first, last).map((item, offset) => ({ item, index: first + offset })));

  let scroller: ReturnType<typeof Scroll> | undefined = $state();

  /** Scrolls the row at `index` to the top. */
  export function scrollToIndex(index: number): void {
    scroller?.scrollTo(index * rowHeight);
  }
</script>

<Scroll
  bind:this={scroller}
  {style}
  onscroll={(position) => (top = position.y)}
  onlayout={(layout) => (height = layout.height)}
>
  <View style={{ height: first * rowHeight, shrink: 0 }} />
  {#each shown as entry (key ? key(entry.item, entry.index) : entry.index)}
    <View style={{ height: rowHeight, shrink: 0 }}>{@render row(entry.item, entry.index)}</View>
  {/each}
  <View style={{ height: (items.length - last) * rowHeight, shrink: 0 }} />
</Scroll>
