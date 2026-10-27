<!--
  A long list that only builds the rows near the window; a list of a hundred thousand items costs
  what a screenful does.

  With `rowHeight`, every row has that height, so where each sits is known without looking at it.
  Without it, rows may differ in height. Each row is measured as it is drawn (a row's height is
  what its content comes to), and rows not measured yet count as the average of those that are.
  The rows that are built sit in the scroller one under the other, between two empty spacers that
  stand for the rows above and below them, so the host lays out their real heights at once. The
  scroller keeps the end in view (`stickToEnd`) or the row at the top of the window (otherwise) while
  it does, so a row above the window that turns out taller or shorter than guessed does not move
  what you are looking at, and the list never waits for the app to measure before it is right.

    <List items={messages} key={(message) => message.id} stickToEnd style={{ flex: 1 }}>
      {#snippet row(message)}<Text>{message.text}</Text>{/snippet}
    </List>

  `items` is read as a whole: pass a new array when it changes, as `$derived` and `.filter()` do.
  Without `key` a row's measured height stays with its position; with it, with the item, which is what a
  list that inserts or reorders wants. Adding items at the end is cheap either way.
-->
<script lang="ts" generics="T">
  import { untrack, type Snippet } from "svelte";
  import Scroll from "./Scroll.svelte";
  import View from "./View.svelte";
  import { Heights } from "./heights";
  import type { Layout, ScrollPosition } from "./events";
  import type { Style } from "./style";

  interface Props {
    items: T[];
    /** The height of every row, in pixels. Leave it out for rows of different heights. */
    rowHeight?: number;
    /** What a row not measured yet counts as, before any has been. The default is 40. */
    estimatedRowHeight?: number;
    row: Snippet<[T, number]>;
    /** Keeps a row's state, and its measured height, with its item as the list changes. */
    key?: (item: T, index: number) => unknown;
    /** How many rows past each edge of the window are built anyway, within a window's height of it. The default is 8. */
    overscan?: number;
    /**
     * Opens at the end, and stays there as items are added and rows change height, as a chat does, for as long
     * as the list is scrolled to its end. Scrolling up lets go of it; scrolling back to the end takes it again.
     */
    stickToEnd?: boolean;
    style?: Style;
    onscroll?: (position: ScrollPosition) => void;
  }

  /** How tall a stretch of rows is laid out at once, in pixels. */
  const STRETCH = 1_000_000;

  let { items, rowHeight, estimatedRowHeight = 40, row, key, overscan = 8, stickToEnd = false, style, onscroll }: Props = $props();

  let scroller: ReturnType<typeof Scroll> | undefined = $state();
  /** How far the list is scrolled, counted from the top of the whole list. */
  let top = $state(0);
  let viewport = $state(0);
  let width = 0;
  /**
   * The scroll position of the host is counted from `origin`, not from the top of the list. A long
   * list is taller than the host can draw at one go (millions of pixels), so only a stretch of it,
   * `STRETCH` pixels, is laid out at a time, and the stretch moves along as the window nears its
   * end, without the window seeing it.
   */
  let origin = $state(0);

  /** Whether the rows are measured. This is decided once, by whether `rowHeight` was given. */
  const measured = untrack(() => rowHeight === undefined);

  // Where rows are: by arithmetic when they share a height, from `heights` when they are measured.
  // `heights` is not reactive; `version` says when it changed.
  let heights = new Heights(untrack(() => items.length), untrack(() => estimatedRowHeight));
  let version = $state(0);
  const topOf = (index: number): number => (measured ? heights.top(index) : index * (rowHeight ?? 0));
  const indexAt = (offset: number): number =>
    measured ? heights.indexAt(offset) : Math.min(Math.max(Math.floor(offset / (rowHeight || 1)), 0), Math.max(items.length - 1, 0));
  const heightOf = (index: number): number => (measured ? heights.height(index) : (rowHeight ?? 0));

  const known = new Map<unknown, number>();
  /** The row at the top of the window and how far the window is below its top, which scrolling keeps. */
  let anchorIndex = 0;
  let anchorDelta = 0;
  let anchorKey: unknown = undefined;
  /** Where the list last scrolled itself to, to tell its own scroll events from the user's. */
  let expected: number | null = null;
  /** Whether the list is at its end, where `stickToEnd` keeps it. The host decides; this is what it last said. */
  let pinned = $state(untrack(() => stickToEnd));
  let seen: { length: number; first: unknown; last: unknown } = { length: 0, first: undefined, last: undefined };

  /** What the scroller keeps in view: the end, or, where heights are measured, the row at the top of the window. */
  const anchoring = $derived(stickToEnd ? "end" : measured ? "content" : undefined);

  const keyOf = (item: T, index: number): unknown => (key ? key(item, index) : index);

  /**
   * Reading `version` is how a derived value learns that `heights` changed. It has to be an argument,
   * because a bare `version;` or `void version;` is dropped by the build's TypeScript step before Svelte
   * sees it.
   */
  const heightsAt = (_version: number): Heights => heights;

  /** The height of every row together, as known right now. */
  const totalHeight = (): number => (measured ? heights.total : items.length * (rowHeight ?? 0));
  /**
   * How far past the top of the window the rows are built: the window's height, or twenty rows before the
   * list knows it.
   */
  const reach = (): number => (viewport > 0 ? viewport : (measured ? heights.estimate : (rowHeight ?? 1)) * 20);
  /** Where the window starts, counted from the top of the whole list: at the end, when it is there. */
  const windowTop = (): number => (pinned ? Math.max(totalHeight() - reach(), 0) : top);
  // `overscan` rows are built past each edge, but no further than a window's height, so tall rows do not
  // build the list out to its ends.
  const first = $derived.by(() => {
    heightsAt(version);
    const start = windowTop();
    const byCount = indexAt(start) - overscan;
    if (viewport <= 0) {
      return Math.max(0, byCount);
    }
    return Math.max(0, byCount, indexAt(start - viewport));
  });
  const last = $derived.by(() => {
    heightsAt(version);
    if (pinned) {
      return items.length;
    }
    const end = top + reach();
    const byCount = indexAt(end) + 1 + overscan;
    if (viewport <= 0) {
      return Math.min(items.length, byCount);
    }
    return Math.min(items.length, byCount, indexAt(end + viewport) + 1);
  });
  const shown = $derived.by(() => {
    heightsAt(version);
    const out: { item: T; index: number }[] = [];
    for (let index = first; index < last; index++) {
      out.push({ item: items[index], index });
    }
    return out;
  });
  /** The height of the rows above the first one built, and of those below the last, as the list counts them. */
  const spaceAbove = $derived.by(() => {
    heightsAt(version);
    return Math.max(topOf(first) - origin, 0);
  });
  const spaceBelow = $derived.by(() => {
    heightsAt(version);
    return Math.max(Math.min(totalHeight() - origin, STRETCH) - spaceAbove - (topOf(last) - topOf(first)), 0);
  });

  // The items changed: the rows keep what is known about them, and the window keeps its place.
  $effect.pre(() => {
    const list = items;
    const length = list.length;
    untrack(() => itemsChanged(list, length));
  });

  function itemsChanged(list: T[], length: number): void {
    if (!measured) {
      version++;
      rebase();
      return;
    }
    const appended =
      length >= seen.length &&
      seen.length > 0 &&
      keyOf(list[0], 0) === seen.first &&
      keyOf(list[seen.length - 1], seen.length - 1) === seen.last;
    if (!key || appended || seen.length === 0) {
      heights.resize(length);
    } else {
      const remembered = new Float64Array(length);
      for (let index = 0; index < length; index++) {
        remembered[index] = known.get(key(list[index], index)) ?? 0;
      }
      heights.replace(length, remembered);
      relocateAnchor(list);
    }
    seen = { length, first: length > 0 ? keyOf(list[0], 0) : undefined, last: length > 0 ? keyOf(list[length - 1], length - 1) : undefined };
    version++;
    keepPlace();
    rebase();
  }

  /** After the items were rearranged, finds the row the window was on. */
  function relocateAnchor(list: T[]): void {
    if (key === undefined || list.length === 0 || (anchorIndex < list.length && key(list[anchorIndex], anchorIndex) === anchorKey)) {
      anchorIndex = Math.min(anchorIndex, Math.max(list.length - 1, 0));
      return;
    }
    const found = list.findIndex((item, index) => key(item, index) === anchorKey);
    anchorIndex = found >= 0 ? found : Math.min(anchorIndex, list.length - 1);
  }

  /** Remembers which row the window is on. */
  function record(): void {
    anchorIndex = indexAt(top + 0.01);
    anchorDelta = top - topOf(anchorIndex);
    anchorKey = anchorIndex < items.length ? keyOf(items[anchorIndex], anchorIndex) : undefined;
  }

  /**
   * After the items changed or a row was measured, moves the window to where the anchor row now is, so the rows
   * built stay around the one being looked at. The host keeps that row where it is on screen when the rows around
   * it change size, so this only tells the list where the row will be; the host's next scroll report confirms it.
   */
  function keepPlace(): void {
    if (pinned) {
      return;
    }
    const furthest = Math.max(totalHeight() - viewport, 0);
    top = Math.min(Math.max(heights.top(anchorIndex) + anchorDelta, 0), furthest);
  }

  /** The origin that keeps `offset` well inside the stretch that is laid out. */
  function originFor(offset: number): number {
    const everything = totalHeight();
    const furthest = Math.max(everything - STRETCH, 0);
    if (everything <= STRETCH) {
      return 0;
    }
    const inside = offset - origin;
    const margin = STRETCH * 0.2;
    const farEnough = inside >= margin && inside <= STRETCH - viewport - margin;
    const atStart = origin === 0 && inside >= 0 && inside <= STRETCH - viewport - margin;
    const atEnd = origin >= furthest - 0.5 && origin <= furthest + 0.5 && inside >= margin && inside <= STRETCH - viewport;
    if (farEnough || atStart || atEnd) {
      return origin;
    }
    return Math.min(Math.max(Math.round(offset - STRETCH / 2), 0), furthest);
  }

  /**
   * Moves the stretch when the window has come near its end. The window does not move. At the end of the list
   * the host is pinned there already, so the stretch is only moved under it.
   */
  function rebase(): void {
    if (pinned) {
      origin = originFor(Math.max(totalHeight() - viewport, 0));
      return;
    }
    if (originFor(top) !== origin) {
      moveTo(top);
    }
  }

  function moveTo(offset: number): void {
    origin = originFor(offset);
    top = offset;
    expected = offset - origin;
    scroller?.scrollTo(expected);
  }

  function measure(index: number, height: number, item: T): void {
    if (key) {
      known.set(key(item, index), height);
    }
    if (heights.set(index, height)) {
      version++;
      keepPlace();
      rebase();
    }
  }

  function scrolled(position: ScrollPosition): void {
    top = position.y + origin;
    onscroll?.({ ...position, y: top, maxY: Math.max(totalHeight() - viewport, 0) });
    // The host stops at the end of what it has, so an offset beyond that comes back as the end.
    if (expected !== null && Math.abs(position.y - Math.min(expected, position.maxY)) < 1) {
      expected = null;
      return;
    }
    expected = null;
    pinned = stickToEnd && position.y >= position.maxY - 1;
    if (measured) {
      record();
    }
    rebase();
  }

  function laidOut(layout: Layout): void {
    viewport = layout.height;
    if (!measured) {
      return;
    }
    // Rows wrap differently at another width, so what was measured no longer holds. The rows that are
    // built are the exception: they have the new width too, so each has reported its height already
    // or is about to, and this layout may arrive before or after that. What they reported is kept.
    if (Math.abs(layout.width - width) >= 0.5) {
      if (width !== 0) {
        const built = new Set<unknown>();
        for (let index = first; index < last; index++) {
          built.add(keyOf(items[index], index));
        }
        for (const remembered of known.keys()) {
          if (!built.has(remembered)) {
            known.delete(remembered);
          }
        }
        heights.forgetOutside(first, last);
        version++;
      }
      width = layout.width;
    }
  }

  /**
   * Scrolls row `index` into view: to the top of the list (`"start"`, the default), the bottom
   * (`"end"`), the middle (`"center"`), or only as far as it takes to show it (`"nearest"`).
   */
  export function scrollToIndex(index: number, options: { align?: "start" | "center" | "end" | "nearest" } = {}): void {
    const count = items.length;
    if (count === 0) {
      return;
    }
    const target = Math.min(Math.max(Math.round(index), 0), count - 1);
    const align = options.align ?? "start";
    const rowTop = topOf(target);
    const rowSize = heightOf(target);
    let offset: number;
    if (align === "start") {
      offset = rowTop;
    } else if (align === "end") {
      offset = rowTop + rowSize - viewport;
    } else if (align === "center") {
      offset = rowTop + rowSize / 2 - viewport / 2;
    } else if (rowTop < top) {
      offset = rowTop;
    } else if (rowTop + rowSize > top + viewport) {
      offset = rowTop + rowSize - viewport;
    } else {
      return;
    }
    offset = Math.min(Math.max(offset, 0), Math.max(totalHeight() - viewport, 0));
    if (stickToEnd && align === "end" && target === count - 1) {
      scrollToEnd();
      return;
    }
    pinned = false;
    if (measured) {
      top = offset;
      record();
      // `align: "start"` should leave the row exactly at the top, whatever is learned about its neighbours.
      if (align === "start") {
        anchorIndex = target;
        anchorDelta = 0;
        anchorKey = keyOf(items[target], target);
      }
    }
    moveTo(offset);
  }

  /** Scrolls to the very end, which the host pins, however tall the rows turn out to be. */
  function scrollToEnd(): void {
    pinned = true;
    top = Math.max(totalHeight() - viewport, 0);
    origin = originFor(top);
    expected = null;
    scroller?.scrollTo(Number.MAX_SAFE_INTEGER);
  }
</script>

<Scroll bind:this={scroller} {style} anchor={anchoring} onscroll={scrolled} onlayout={laidOut}>
  <View style={{ height: spaceAbove, shrink: 0 }} />
  {#each shown as entry (key ? key(entry.item, entry.index) : entry.index)}
    {#if measured}
      <View style={{ shrink: 0 }} onresize={(layout: Layout) => measure(entry.index, layout.height, entry.item)}>
        {@render row(entry.item, entry.index)}
      </View>
    {:else}
      <View style={{ height: rowHeight, shrink: 0 }}>
        {@render row(entry.item, entry.index)}
      </View>
    {/if}
  {/each}
  <View style={{ height: spaceBelow, shrink: 0 }} />
</Scroll>
