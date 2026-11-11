<!--
  Something to press: by the pointer, or with Enter or Space while it has focus. Its children sit in a
  row. The browser's `Button` is a `button` element with its own look taken away.
-->
<script lang="ts">
  import type { ButtonProps } from "../../ui/props";
  import { cssText, layered, toCss } from "./css";
  import { interaction } from "./interact.svelte";

  let props: ButtonProps = $props();

  const ix = interaction(() => (props.disabled ? { ...props, onpress: undefined, active: undefined } : props));
  let element: HTMLElement | undefined = $state();

  const css = $derived(
    cssText(
      toCss(
        layered(props.disabled ? { cursor: "default", ...props.style, ...props.disabledStyle } : props.style, [
          ix.focused && props.focus,
          ix.hovered && props.hover,
          ix.pressed && props.active,
        ]),
      ),
    ),
  );

  export function focus(): void {
    ix.focus(element);
  }

  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  export function blur(): void {
    ix.blur(element);
  }
</script>

<button
  bind:this={element}
  type="button"
  class="g-button"
  style={css}
  aria-disabled={props.disabled ? "true" : undefined}
  tabindex={props.disabled ? -1 : undefined}
  {@attach ix.attach}
>{@render props.children?.()}</button>
