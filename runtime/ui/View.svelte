<!--
  A box that lays out its children: a column unless `style.direction` says "row".
  `hover`, `active` and `focus` are styles laid over `style` while the pointer is
  over it, while it is pressed, and while it has focus.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import {
    itself,
    listener,
    type FileDragEvent,
    type KeyEvent,
    type Layout,
    type PointerEvent,
    type WheelEvent,
  } from "./events";

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
    /** The buttons `ondown` takes. A press of another button passes to what is around the view. The default is all three. */
    buttons?: ("left" | "middle" | "right")[];
    onmove?: (event: PointerEvent) => void;
    onup?: (event: PointerEvent) => void;
    onwheel?: (event: WheelEvent) => void;
    onlayout?: (layout: Layout) => void;
    /** Like `onlayout`, but only when its width or height changed, not when it merely moved. */
    onresize?: (layout: Layout) => void;
    onkey?: (event: KeyEvent) => void;
    onfocus?: () => void;
    onblur?: () => void;
    /** Files dragged in from another program entered, left, moved over this, or were dropped on it. */
    ondragenter?: (event: FileDragEvent) => void;
    ondragleave?: (event: FileDragEvent) => void;
    ondragover?: (event: FileDragEvent) => void;
    ondrop?: (event: FileDragEvent) => void;
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
    buttons: props.buttons?.join(","),
    onmove: listener(props.onmove, itself<PointerEvent>),
    onup: listener(props.onup, itself<PointerEvent>),
    onwheel: listener(props.onwheel, itself<WheelEvent>),
    onlayout: listener(props.onlayout, itself<Layout>),
    onresize: listener(props.onresize, itself<Layout>),
    onkey: listener(props.onkey, itself<KeyEvent>),
    onfocus: listener(props.onfocus, () => undefined),
    onblur: listener(props.onblur, () => undefined),
    ondragenter: listener(props.ondragenter, itself<FileDragEvent>),
    ondragleave: listener(props.ondragleave, itself<FileDragEvent>),
    ondragover: listener(props.ondragover, itself<FileDragEvent>),
    ondrop: listener(props.ondrop, itself<FileDragEvent>),
  });

  let element: any;

  /** Gives it keyboard focus. It must be `focusable`. */
  export function focus(): void {
    element?.focus();
  }
</script>

<g-view bind:this={element} {...attributes}>{@render props.children?.()}</g-view>
