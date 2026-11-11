<!--
  An SVG icon, tinted with `color` or else the text colour around it. Pass the markup
  (`import gear from "./gear.svg"` gives it, with the build's text loader), or the URL of a file.
  The drawing is used as a mask over the colour, as the host does, so any SVG is tinted, whatever
  fills and strokes it names.
-->
<script lang="ts">
  import type { IconProps } from "../../ui/props";
  import { cssText, toCss } from "./css";

  let props: IconProps = $props();

  const source = $derived(
    props.svg !== undefined
      ? `url("data:image/svg+xml;utf8,${encodeURIComponent(props.svg)}")`
      : props.src !== undefined
        ? `url(${JSON.stringify(props.src)})`
        : "none",
  );

  const css = $derived(
    cssText({
      ...toCss({ size: props.size, color: props.color, ...props.style }),
      "background-color": "currentColor",
      "-webkit-mask-image": source,
      "mask-image": source,
      "-webkit-mask-repeat": "no-repeat",
      "mask-repeat": "no-repeat",
      "-webkit-mask-position": "center",
      "mask-position": "center",
      "-webkit-mask-size": "contain",
      "mask-size": "contain",
    }),
  );
</script>

<span class="g-icon" style={css}></span>
