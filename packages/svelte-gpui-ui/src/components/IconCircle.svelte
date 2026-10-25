<!-- An icon on a soft tinted tile, to mark a row or a card by what it is about. -->
<script lang="ts">
  import { Icon, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { iconCircleIconSize, iconCircleLook, type IconCircleSize } from "./status-look";
  import type { StatusTone } from "./status-tone";

  interface Props {
    /** SVG markup of the icon, best in a bold weight. */
    icon: string;
    tone?: StatusTone;
    size?: IconCircleSize;
    /** Layout only, such as margins. */
    style?: Style;
  }

  let { icon, tone = "neutral", size = "md", style }: Props = $props();

  const theme = $derived(currentTheme());
</script>

<View style={{ ...iconCircleLook(theme, tone, size), ...style }}>
  <Icon svg={icon} size={iconCircleIconSize(size)} />
</View>
