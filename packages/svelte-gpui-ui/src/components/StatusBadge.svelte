<!--
  A status in one or two words, in a soft colour: `success` for healthy or done, `warning` for needs a look,
  `danger` for failed, `info` for in progress. Add an icon or a dot so the meaning does not rest on colour alone.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import {
    statusBadgeDotLook,
    statusBadgeIconSize,
    statusBadgeLook,
    statusBadgeTextLook,
  } from "./status-look";
  import type { StatusTone } from "./status-tone";

  interface Props {
    tone?: StatusTone;
    /** SVG markup before the text, best in a bold weight. */
    icon?: string;
    /** A coloured dot before the text, for statuses in dense tables. */
    dot?: boolean;
    /** Layout only, such as margins. */
    style?: Style;
    /** The status text. */
    children?: Snippet;
  }

  let { tone = "neutral", icon, dot = false, style, children }: Props = $props();

  const theme = $derived(currentTheme());
</script>

<View style={{ ...statusBadgeLook(theme, tone), ...style }}>
  {#if dot}
    <View style={statusBadgeDotLook(theme, tone)} />
  {/if}
  {#if icon}
    <Icon svg={icon} size={statusBadgeIconSize} />
  {/if}
  <Text style={statusBadgeTextLook()}>{@render children?.()}</Text>
</View>
