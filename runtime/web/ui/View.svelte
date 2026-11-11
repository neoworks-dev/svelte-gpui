<!--
  A box that lays out its children: a column unless `style.direction` says "row". The browser's
  `View`: a `div` with the style as inline CSS, taking the same props as the native one.
-->
<script lang="ts">
  import type { ViewProps } from "../../ui/props";
  import { cssText, layered, toCss } from "./css";
  import { interaction } from "./interact.svelte";

  let props: ViewProps = $props();

  const ix = interaction(() => props);
  let element: HTMLElement | undefined = $state();

  const css = $derived(
    cssText(toCss(layered(props.style, [ix.focused && props.focus, ix.hovered && props.hover, ix.pressed && props.active]))),
  );

  /** Gives it keyboard focus. It must be `focusable`. */
  export function focus(): void {
    ix.focus(element);
  }

  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  export function blur(): void {
    ix.blur(element);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  bind:this={element}
  class="g-view"
  class:g-pointer={ix.grabbing}
  class:g-listening={ix.listening}
  style={css}
  tabindex={props.focusable ? 0 : undefined}
  {@attach ix.attach}
>{@render props.children?.()}</div>
