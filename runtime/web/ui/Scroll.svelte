<!--
  A view whose content scrolls: vertically, or sideways when `horizontal` is set. Give it a bounded
  size, such as `flex: 1` inside a column, or it grows to fit.

  With `anchor`, a vertical scroller keeps what is in view while its content changes size:
  `"content"` keeps the child at the top of the window where it is on screen, and `"end"` does that
  and opens scrolled to the end, staying there while it is at the end. The browser's own
  `overflow-anchor` is off, since not every browser has it and this must behave the same in all.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { ScrollProps } from "../../ui/props";
  import type { ScrollPosition } from "../../ui/events";
  import { offsetAfter } from "../../ui/scroll";
  import { cssText, toCss } from "./css";
  import { watchLayout } from "./layout";

  let props: ScrollProps = $props();

  let element: HTMLElement | undefined = $state();
  /** What was last reported, adjusted by the moves the app has asked for since. */
  let current = $state.raw<ScrollPosition>({ x: 0, y: 0, maxX: 0, maxY: 0 });

  const css = $derived(cssText(toCss(props.style)));

  function read(node: HTMLElement): ScrollPosition {
    return {
      x: node.scrollLeft,
      y: node.scrollTop,
      maxX: Math.max(node.scrollWidth - node.clientWidth, 0),
      maxY: Math.max(node.scrollHeight - node.clientHeight, 0),
    };
  }

  /** Tells the app where the scroller is, when that is not what it was last told. */
  function report(node: HTMLElement): void {
    const now = read(node);
    const before = untrack(() => current);
    const same = (a: number, b: number): boolean => Math.abs(a - b) < 0.5;
    if (same(now.x, before.x) && same(now.y, before.y) && same(now.maxX, before.maxX) && same(now.maxY, before.maxY)) {
      return;
    }
    current = now;
    props.onscroll?.(now);
  }

  let reportFrame: number | null = null;
  let reportTask: ReturnType<typeof setTimeout> | null = null;
  /**
   * Reports once the next frame is laid out and its resize observers have run. What `keepPlace` moved
   * is reported this way: the app may answer a report by changing the children, which calls
   * `keepPlace` again. Reported at once, that circle never leaves the microtasks and the page hangs;
   * reported in the frame's own animation callbacks, it still comes before the new children are
   * measured. Either way an app that places them by their measured height, as `List` does, never
   * learns it and swings between two places. The task queued from the frame runs after it.
   */
  function reportSoon(node: HTMLElement): void {
    if (reportFrame !== null || reportTask !== null) {
      return;
    }
    reportFrame = requestAnimationFrame(() => {
      reportFrame = null;
      reportTask = setTimeout(() => {
        reportTask = null;
        report(node);
      }, 0);
    });
  }

  /** Whether the scroller is held at its end, which `anchor="end"` keeps it at as the content grows. */
  let pinned = false;
  /** The child at the top of the window, and how far below the top edge it was. */
  let anchored: { child: Element; offset: number } | null = null;

  function remember(node: HTMLElement): void {
    const top = node.getBoundingClientRect().top;
    anchored = null;
    for (const child of node.children) {
      const box = child.getBoundingClientRect();
      if (box.bottom > top + 0.01) {
        anchored = { child, offset: box.top - top };
        return;
      }
    }
  }

  /** The content changed size: keep what the app is looking at where it was. */
  function keepPlace(node: HTMLElement): void {
    if (props.horizontal || props.anchor === undefined) {
      return;
    }
    if (props.anchor === "end" && pinned) {
      node.scrollTop = node.scrollHeight;
    } else if (anchored !== null && anchored.child.isConnected) {
      const moved = anchored.child.getBoundingClientRect().top - node.getBoundingClientRect().top - anchored.offset;
      if (Math.abs(moved) >= 0.5) {
        node.scrollTop += moved;
      }
    }
    remember(node);
    reportSoon(node);
  }

  /** Untracked, so that what the app's `onscroll` reads does not make the scroller set itself up again. */
  const attach = (node: HTMLElement): (() => void) => untrack(() => setup(node));

  function setup(node: HTMLElement): () => void {
    pinned = props.anchor === "end";
    if (pinned && !props.horizontal) {
      node.scrollTop = node.scrollHeight;
    }
    remember(node);
    report(node);

    const onscroll = (): void => {
      pinned = props.anchor === "end" && read(node).maxY - node.scrollTop <= 1;
      remember(node);
      report(node);
    };
    node.addEventListener("scroll", onscroll, { passive: true });

    // The content grows and shrinks as children do, and as they come and go.
    const resizes = new ResizeObserver(() => keepPlace(node));
    resizes.observe(node);
    const watched = new Set<Element>();
    const watchChildren = (): void => {
      for (const child of watched) {
        if (child.parentElement !== node) {
          resizes.unobserve(child);
          watched.delete(child);
        }
      }
      for (const child of node.children) {
        if (!watched.has(child)) {
          resizes.observe(child);
          watched.add(child);
        }
      }
    };
    watchChildren();
    const changes = new MutationObserver(() => {
      watchChildren();
      keepPlace(node);
    });
    changes.observe(node, { childList: true });

    return () => {
      node.removeEventListener("scroll", onscroll);
      resizes.disconnect();
      changes.disconnect();
      if (reportFrame !== null) {
        cancelAnimationFrame(reportFrame);
        reportFrame = null;
      }
      if (reportTask !== null) {
        clearTimeout(reportTask);
        reportTask = null;
      }
    };
  }

  /**
   * Where it is scrolled to and how far it can go, as `{ x, y, maxX, maxY }`. It is current as of the last
   * frame, and at once after `scrollTo` or `scrollBy`. Read in a template or an effect, it updates the way state does.
   */
  export function position(): ScrollPosition {
    return { ...current };
  }

  /** Scrolls so that `y` (or `x` when horizontal) is at the top (or left) edge. */
  export function scrollTo(offset: number): void {
    const node = element;
    if (node === undefined) {
      return;
    }
    const here = read(node);
    if (props.horizontal) {
      node.scrollLeft = Math.min(Math.max(offset, 0), here.maxX);
    } else {
      node.scrollTop = Math.min(Math.max(offset, 0), here.maxY);
      pinned = props.anchor === "end" && offset >= here.maxY - 1;
    }
    remember(node);
    report(node);
  }

  /** Scrolls down (or right, when horizontal) by `delta` pixels, or up and left when it is negative. It stops at the ends. */
  export function scrollBy(delta: number): void {
    const here = untrack(() => current);
    scrollTo(offsetAfter(here, props.horizontal ?? false, delta));
  }
</script>

<div
  bind:this={element}
  class="g-scroll"
  data-horizontal={props.horizontal ? "" : undefined}
  style={css}
  {@attach attach}
  {@attach (node) => (props.onlayout ? watchLayout(node, (layout) => props.onlayout?.(layout)) : undefined)}
>{@render props.children?.()}</div>
