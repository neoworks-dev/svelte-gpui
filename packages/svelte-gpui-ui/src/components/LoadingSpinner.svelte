<!--
  A small circle that keeps turning while something loads and nothing is known about how long it takes.
  For work with a known end, use `ProgressIndicator` with a `value`.
-->
<script lang="ts">
  import type { Color, Style } from "@neoworks/svelte-gpui/ui";
  import ProgressIndicator from "./ProgressIndicator.svelte";

  interface Props {
    /** Diameter in pixels. */
    size?: number;
    /** Stroke width in viewBox units out of 48. */
    thickness?: number;
    /** What is loading. For assistive tools, which the host does not have yet, so it is not drawn. */
    label?: string;
    /** The colour of the spinner, standing in for the web's surrounding text colour. The text colour when left out. */
    color?: Color;
    /** Layout only, such as margins. */
    style?: Style;
  }

  let { size = 24, thickness = 5, label = "Loading", color, style }: Props = $props();
</script>

<ProgressIndicator variant="circular" tone="inherit" {label} {size} {thickness} {color} {style} />
