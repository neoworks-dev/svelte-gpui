<!--
  A count or a dot. Given children, it sits on their top right corner, as on an icon button; without them it stands alone.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { wrapperAlignment } from "./look";
  import {
    badgeCountText,
    badgeLook,
    badgeTextLook,
    defaultBadgeMaximum,
    isBadgeVisible,
    type BadgeTone,
  } from "./badge-look";

  interface Props {
    /** The number to show. Zero or nothing hides the badge, unless `dot` is set. */
    count?: number;
    /** Counts above this show as "99+". */
    max?: number;
    /** A small dot without a number, for something new that needs no counting. */
    dot?: boolean;
    /** `danger` for things that need attention, `neutral` and `accent` for quieter counts. */
    tone?: BadgeTone;
    /** What the badge is attached to, usually an icon or an icon button. */
    children?: Snippet;
    /** Layout only, such as margins. */
    style?: Style;
  }

  let { count, max = defaultBadgeMaximum, dot = false, tone = "danger", children, style }: Props = $props();

  const visible = $derived(isBadgeVisible(count, dot));
  const attached = $derived(children !== undefined);
  const look = $derived(badgeLook(currentTheme(), { tone, dot, attached }));
  const boxStyle = $derived.by(() => {
    if (attached) {
      return look;
    }
    return { ...look, ...style };
  });
</script>

{#snippet badge()}
  {#if visible}
    <View style={boxStyle}>
      {#if !dot}
        <Text style={badgeTextLook()}>{badgeCountText(count, max)}</Text>
      {/if}
    </View>
  {/if}
{/snippet}

{#if children}
  <View style={{ position: "relative", direction: "row", ...wrapperAlignment(style), ...style }}>
    {@render children()}
    {@render badge()}
  </View>
{:else}
  {@render badge()}
{/if}
