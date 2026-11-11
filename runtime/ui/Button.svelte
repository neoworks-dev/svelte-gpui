<!--
  Something to press: by the pointer, or with Enter or Space while it has focus.
  Its children sit in a row.
-->
<script lang="ts">
  import { encode } from "./style";
  import { itself, listener, type KeyEvent, type PointerEvent } from "./events";
  import type { ButtonProps } from "./props";

  let props: ButtonProps = $props();

  const attributes = $derived({
    look: encode(props.disabled ? { cursor: "default", ...props.style, ...props.disabledStyle } : props.style),
    hover: encode(props.hover),
    active: encode(props.active),
    focus: encode(props.focus),
    disabled: props.disabled ? "" : undefined,
    onpress: props.disabled ? undefined : listener(props.onpress, itself<PointerEvent>),
    onhover: listener(props.onhover, (event: { hovered: boolean }) => event.hovered),
    onkey: listener(props.onkey, itself<KeyEvent>),
    onfocus: listener(props.onfocus, () => undefined),
    onblur: listener(props.onblur, () => undefined),
  });

  let element: any;

  export function focus(): void {
    element?.focus();
  }

  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  export function blur(): void {
    element?.blur();
  }
</script>

<g-button bind:this={element} {...attributes}>{@render props.children?.()}</g-button>
