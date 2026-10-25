<!--
  The one main action of a screen, floating above the content. `extended` shows the label beside the icon;
  collapse it while the page scrolls down to keep the content free.
-->
<script lang="ts">
  import { Button, Icon, Text, type PointerEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { fabIconSizes, fabLabelLook, fabLook, type FabSize, type FabVariant } from "./fab-look";

  interface Props {
    /** SVG markup. */
    icon: string;
    /** Shown next to the icon when `extended`. */
    label: string;
    /** `primary` for the main action of the screen, `surface` when it floats over busy content, `accent` for something good such as a new upload. */
    variant?: FabVariant;
    size?: FabSize;
    /** Shows the label beside the icon. */
    extended?: boolean;
    /** Layout only, such as the fixed position in a corner. */
    style?: Style;
    onpress?: (event: PointerEvent) => void;
  }

  let { icon, label, variant = "primary", size = "md", extended = false, style, onpress }: Props = $props();

  const look = $derived(fabLook(currentTheme(), { variant, size, extended }));
</script>

<Button style={{ ...look.style, ...style }} hover={look.hover} active={look.active} focus={look.focus} {onpress}>
  <Icon svg={icon} size={fabIconSizes[size]} />
  {#if extended}
    <Text style={fabLabelLook()}>{label}</Text>
  {/if}
</Button>
