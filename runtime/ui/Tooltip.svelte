<!--
  A small label that appears next to its content after the pointer has rested on it for `delay`
  milliseconds, and goes when the pointer leaves. After one tooltip has been showing, the next appears
  at once. It never takes the pointer, so it cannot flicker under it.

    <Tooltip text="Save the document" side="top">
      <Button onpress={save}><Icon svg={floppy} /></Button>
    </Tooltip>

  Pass a `content` snippet instead of `text` for anything more than a line.
-->
<script lang="ts" module>
  /** When a tooltip went away last, so moving between neighbours does not wait again. */
  let tooltipsHiddenAt = 0;
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import Overlay from "./Overlay.svelte";
  import Text from "./Text.svelte";
  import View from "./View.svelte";
  import { tip } from "./overlay-look";
  import type { Style } from "./style";

  interface Props {
    text?: string;
    content?: Snippet;
    /** Milliseconds the pointer rests before it shows. The default is 500. */
    delay?: number;
    side?: "top" | "bottom" | "left" | "right";
    align?: "start" | "center" | "end";
    /** The gap between the content and the tooltip. The default is 6. */
    offset?: number;
    /** Never shows. */
    disabled?: boolean;
    /** The tooltip's style. */
    style?: Style;
    /** The wrapper around the content, which is what the pointer rests on. */
    anchorStyle?: Style;
    children?: Snippet;
  }

  let { text, content, delay = 500, side = "bottom", align = "center", offset = 6, disabled = false, style, anchorStyle, children }: Props = $props();

  let shown = $state(false);
  let timer: number | undefined;

  function hover(hovered: boolean): void {
    clearTimeout(timer);
    if (!hovered) {
      hide();
      return;
    }
    const soon = Date.now() - tooltipsHiddenAt < 300;
    timer = setTimeout(() => (shown = !disabled), soon ? 0 : delay) as unknown as number;
  }

  function hide(): void {
    if (shown) {
      tooltipsHiddenAt = Date.now();
    }
    shown = false;
  }

  $effect(() => () => clearTimeout(timer));
</script>

<View style={{ alignSelf: "start", ...anchorStyle }} onhover={hover}>
  {@render children?.()}
  <Overlay {side} {align} {offset} passthrough layer={1} style={{ ...tip, ...style }}>
    {#if shown}
      {#if content}{@render content()}{:else}<Text>{text}</Text>{/if}
    {/if}
  </Overlay>
</View>
