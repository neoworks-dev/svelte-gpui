<!--
  An image from a file path, a `data:` URL, or an `http(s)` URL, which the host
  fetches and caches. It loads when it comes near the window and is decoded no
  larger than it is shown. Until then its box keeps the image's proportions.
-->
<script lang="ts">
  import { encode } from "./style";
  import { listener } from "./events";
  import type { ImageProps } from "./props";

  let props: ImageProps = $props();

  const attributes = $derived({
    src: props.src,
    fit: props.fit,
    look: encode(props.style),
    onload: listener(props.onload, () => undefined),
    onerror: listener(props.onerror, () => undefined),
  });
</script>

<g-image {...attributes}></g-image>
