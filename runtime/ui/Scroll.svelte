<!--
  A view whose content scrolls: vertically, or sideways when `horizontal` is set.
  Give it a bounded size, such as `flex: 1` inside a column, or it grows to fit.

  With `anchor`, a vertical scroller keeps what is in view while the host lays out, without a round
  trip to the app. Only the scroller's own children are kept in place, so put what should hold still
  directly in it.

  - `"content"`: when what is above the window changes size (a row taller than guessed, rows added
    above), the child at the top of the window stays where it is on screen, as `overflow-anchor`
    does in a browser.
  - `"end"`: the same, and the scroller opens scrolled to its end, on the first frame. While it is at
    the end, content that grows or shrinks (a row changing height, rows added, text wrapping again
    after a resize) keeps the end in view. Scrolling away from the end lets go of it, and scrolling
    back, or to an offset at or past `maxY`, pins it again.
-->
<script lang="ts">
  import { untrack, type Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import { offsetAfter } from "./scroll";
  import { itself, listener, type Layout, type ScrollPosition } from "./events";

  interface Props {
    style?: Style;
    horizontal?: boolean;
    /** What the host keeps in view as the content changes: the window's content, or that and the end. Vertical only. */
    anchor?: "content" | "end";
    onscroll?: (position: ScrollPosition) => void;
    onlayout?: (layout: Layout) => void;
    children?: Snippet;
  }

  let props: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    horizontal: props.horizontal ? "" : undefined,
    anchor: props.anchor,
    // Always listened to, so that `position()` is known before anything has asked for it.
    onscroll: (position: ScrollPosition) => {
      remember(position);
      props.onscroll?.(position);
    },
    onlayout: listener(props.onlayout, itself<Layout>),
  });

  let element: any;
  /** What the host last reported, adjusted by the moves the app has asked for since. */
  let current = $state.raw<ScrollPosition>({ x: 0, y: 0, maxX: 0, maxY: 0 });

  function remember({ x, y, maxX, maxY }: ScrollPosition): void {
    current = { x, y, maxX, maxY };
  }

  /**
   * Where it is scrolled to and how far it can go, as `{ x, y, maxX, maxY }`. The host reports this whenever the
   * offset or the content's size changes, so it is current as of the last frame, and at once after `scrollTo` or
   * `scrollBy`. Read in a template or an effect, it updates the way state does.
   */
  export function position(): ScrollPosition {
    return { ...current };
  }

  /** Scrolls so that `y` (or `x` when horizontal) is at the top (or left) edge. */
  export function scrollTo(offset: number): void {
    // Untracked, so that an effect that scrolls does not rerun each time it has scrolled.
    const here = untrack(() => current);
    if (props.horizontal) {
      remember({ ...here, x: Math.min(Math.max(offset, 0), here.maxX) });
      element?.scrollTo(offset, 0);
    } else {
      remember({ ...here, y: Math.min(Math.max(offset, 0), here.maxY) });
      element?.scrollTo(0, offset);
    }
  }

  /** Scrolls down (or right, when horizontal) by `delta` pixels, or up and left when it is negative. It stops at the ends. */
  export function scrollBy(delta: number): void {
    const here = untrack(() => current);
    scrollTo(offsetAfter(here, props.horizontal ?? false, delta));
  }
</script>

<g-scroll bind:this={element} {...attributes}>{@render props.children?.()}</g-scroll>
