<!--
  Where an app element would be drawn. The views an app's Rust crate registers are GPUI views, which
  only the native window draws, so in a browser this shows a notice in the element's box, and
  calling one of its methods fails with the same.
-->
<script lang="ts">
  import { cssText, toCss } from "./css";
  import { onMount } from "svelte";
  import type { Style } from "../../ui/style";

  interface Props {
    /** The name the element is registered under in Rust. */
    __element: string;
    style?: Style;
    [prop: string]: unknown;
  }

  let { __element: name, style }: Props = $props();

  const notice = $derived(`The native element "${name}" is drawn by the GPUI host and not available in a browser.`);

  onMount(() => console.warn(notice));

  export function focus(): void {}

  export function blur(): void {}

  export function __call(method: string): Promise<unknown> {
    return Promise.reject(new Error(`cannot call ${method}: ${notice}`));
  }
</script>

<div
  class="g-view"
  style={cssText({ ...toCss(style), "align-items": "center", "justify-content": "center" })}
  title={notice}
>
  <span class="g-text" style="color:#a1a1aa;font-size:12px">{name}: native only</span>
</div>
