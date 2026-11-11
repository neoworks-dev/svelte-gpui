<!--
  An image from a URL, a `data:` URL, or a path relative to the page, which a browser loads when it
  comes near the window. The box keeps the image's proportions until then.
-->
<script lang="ts">
  import type { ImageProps } from "../../ui/props";
  import { cssText, toCss } from "./css";

  let props: ImageProps = $props();

  const css = $derived(cssText({ ...toCss(props.style), "object-fit": props.fit ?? "contain" }));
</script>

<img
  class="g-image"
  src={props.src}
  alt=""
  draggable="false"
  loading="lazy"
  decoding="async"
  style={css}
  onload={props.onload ? () => props.onload?.() : undefined}
  onerror={props.onerror ? () => props.onerror?.() : undefined}
/>
