<!--
  A box that lays out its children: a column unless `style.direction` says "row".
  `hover`, `active` and `focus` are styles laid over `style` while the pointer is
  over it, while it is pressed, and while it has focus.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import { itself, listener, type KeyEvent, type Layout, type PointerEvent, type WheelEvent } from "./events";

  interface Props {
    style?: Style;
    hover?: Style;
    active?: Style;
    focus?: Style;
    /** Takes keyboard focus when pressed. */
    focusable?: boolean;
    onpress?: (event: PointerEvent) => void;
    onhover?: (hovered: boolean) => void;
    /** A button went down here; `onmove` and `onup` follow wherever the pointer goes until release. */
    ondown?: (event: PointerEvent) => void;
    onmove?: (event: PointerEvent) => void;
    onup?: (event: PointerEvent) => void;
    onwheel?: (event: WheelEvent) => void;
    onlayout?: (layout: Layout) => void;
    onkey?: (event: KeyEvent) => void;
    onfocus?: () => void;
    onblur?: () => void;
    children?: Snippet;
  }

  let props: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    hover: encode(props.hover),
    active: encode(props.active),
    focus: encode(props.focus),
    focusable: props.focusable ? "" : undefined,
    onpress: listener(props.onpress, itself<PointerEvent>),
    onhover: listener(props.onhover, (event: { hovered: boolean }) => event.hovered),
    ondown: listener(props.ondown, itself<PointerEvent>),
    onmove: listener(props.onmove, itself<PointerEvent>),
    onup: listener(props.onup, itself<PointerEvent>),
    onwheel: listener(props.onwheel, itself<WheelEvent>),
    onlayout: listener(props.onlayout, itself<Layout>),
    onkey: listener(props.onkey, itself<KeyEvent>),
    onfocus: listener(props.onfocus, () => undefined),
    onblur: listener(props.onblur, () => undefined),
  });
</script>

<g-view {...attributes}>{@render props.children?.()}</g-view>
