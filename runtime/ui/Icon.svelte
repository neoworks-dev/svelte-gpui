<!--
  An SVG icon, tinted with `color` or else the text colour around it. Pass the
  markup (`import gear from "./gear.svg"` gives it, with the build's text loader),
  or the path of a file.
-->
<script lang="ts">
  import { encode, type Color, type Style } from "./style";

  interface Props {
    svg?: string;
    src?: string;
    /** Width and height in pixels; the font size around it otherwise. */
    size?: number;
    color?: Color;
    style?: Style;
  }

  let props: Props = $props();

  const attributes = $derived({
    svg: props.svg,
    src: props.src,
    look: encode({ size: props.size, color: props.color, ...props.style }),
  });
</script>

<g-icon {...attributes}></g-icon>
