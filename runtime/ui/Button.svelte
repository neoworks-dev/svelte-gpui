<!--
  Something to press: by the pointer, or with Enter or Space while it has focus.
  Its children sit in a row.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { encode, type Style } from "./style";
  import { itself, listener, type KeyEvent, type PointerEvent } from "./events";

  interface Props {
    style?: Style;
    hover?: Style;
    active?: Style;
    focus?: Style;
    /** Laid over `style` while the button is disabled. */
    disabledStyle?: Style;
    disabled?: boolean;
    onpress?: (event: PointerEvent) => void;
    onhover?: (hovered: boolean) => void;
    onkey?: (event: KeyEvent) => void;
    onfocus?: () => void;
    onblur?: () => void;
    children?: Snippet;
  }

  let props: Props = $props();

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
