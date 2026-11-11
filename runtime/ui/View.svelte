<!--
  A box that lays out its children: a column unless `style.direction` says "row".
  `hover`, `active` and `focus` are styles laid over `style` while the pointer is
  over it, while it is pressed, and while it has focus.
-->
<script lang="ts">
  import { encode } from "./style";
  import {
    itself,
    listener,
    type FileDragEvent,
    type KeyEvent,
    type Layout,
    type PointerEvent,
    type WheelEvent,
  } from "./events";
  import type { ViewProps } from "./props";

  let props: ViewProps = $props();

  const attributes = $derived({
    look: encode(props.style),
    hover: encode(props.hover),
    active: encode(props.active),
    focus: encode(props.focus),
    focusable: props.focusable ? "" : undefined,
    keepfocus: props.keepFocus ? "" : undefined,
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

  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  export function blur(): void {
    element?.blur();
  }
</script>

<g-view bind:this={element} {...attributes}>{@render props.children?.()}</g-view>
