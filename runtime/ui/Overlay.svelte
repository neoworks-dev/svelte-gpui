<!--
  Content drawn above everything else, placed against its parent's box. It takes no room
  where it sits, is not cut off by an ancestor that clips, and is drawn after the rest of
  the window, so nothing covers it. It is placed after layout, against the box the host has
  just measured: on `side` of the parent, lined up by `align`, `offset` pixels away, moved to
  the other side when there is too little room, and shifted back inside the window.

  `at` anchors it to a point in the window instead, such as where the pointer was. It draws nothing
  while it has no children. A `View` with extra manners; use `Popover` for the usual open and
  close behaviour.
-->
<script lang="ts">
  import { encode } from "./style";
  import { itself, listener, type KeyEvent, type Layout, type OutsideEvent, type PointerEvent } from "./events";
  import type { OverlayProps } from "./props";

  let props: OverlayProps = $props();

  const attributes = $derived({
    look: encode(props.style),
    hover: encode(props.hover),
    active: encode(props.active),
    focus: encode(props.focus),
    side: props.side,
    align: props.align,
    offset: props.offset === undefined ? undefined : String(props.offset),
    "cross-offset": props.crossOffset === undefined ? undefined : String(props.crossOffset),
    at: props.at === undefined ? undefined : `${props.at[0]},${props.at[1]}`,
    flip: props.flip === undefined ? undefined : String(props.flip),
    shift: props.shift === undefined ? undefined : String(props.shift),
    margin: props.margin === undefined ? undefined : String(props.margin),
    "match-width": props.matchWidth ? "" : undefined,
    passthrough: props.passthrough ? "" : undefined,
    layer: props.layer === undefined ? undefined : String(props.layer),
    focusable: props.focusable ? "" : undefined,
    keepfocus: props.keepFocus ? "" : undefined,
    onoutside: listener(props.onoutside, itself<OutsideEvent>),
    onpress: listener(props.onpress, itself<PointerEvent>),
    onhover: listener(props.onhover, (event: { hovered: boolean }) => event.hovered),
    onlayout: listener(props.onlayout, itself<Layout>),
    onkey: listener(props.onkey, itself<KeyEvent>),
    onfocus: listener(props.onfocus, () => undefined),
    onblur: listener(props.onblur, () => undefined),
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

<g-overlay bind:this={element} {...attributes}>{@render props.children?.()}</g-overlay>
