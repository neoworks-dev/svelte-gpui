<!--
  An image from a file path, a `data:` URL, or an `http(s)` URL, which the host
  fetches and caches. It loads when it comes near the window and is decoded no
  larger than it is shown. Until then its box keeps the image's proportions.
-->
<script lang="ts">
  import { encode, type Style } from "./style";
  import { listener } from "./events";

  interface Props {
    src: string;
    /** How the image fills its box. */
    fit?: "contain" | "cover" | "fill";
    style?: Style;
    onload?: () => void;
    onerror?: () => void;
  }

  let props: Props = $props();

  const attributes = $derived({
    src: props.src,
    fit: props.fit,
    look: encode(props.style),
    onload: listener(props.onload, () => undefined),
    onerror: listener(props.onerror, () => undefined),
  });
</script>

<g-image {...attributes}></g-image>
