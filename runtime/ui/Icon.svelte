<!--
  An SVG icon, tinted with `color` or else the text colour around it. Pass the
  markup (`import gear from "./gear.svg"` gives it, with the build's text loader),
  or the path of a file.
-->
<script lang="ts">
  import { encode } from "./style";
  import type { IconProps } from "./props";

  let props: IconProps = $props();

  const attributes = $derived({
    svg: props.svg,
    src: props.src,
    look: encode({ size: props.size, color: props.color, ...props.style }),
  });
</script>

<g-icon {...attributes}></g-icon>
