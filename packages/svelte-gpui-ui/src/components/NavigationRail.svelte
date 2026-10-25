<!--
  A narrow column of three to seven destinations for tablets and desktops. `header` sits above them, such as a
  menu button or a Fab, and `footer` is pinned to the bottom, such as the account button.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import NavDestination from "./NavDestination.svelte";
  import { isCurrentItem, type NavItem } from "./navigation";
  import { navigationRailColumn, navigationRailItems, navigationRailLook } from "./navigation-look";

  interface Props {
    /** Three to seven destinations. */
    items: NavItem[];
    /** The `value` of the active destination. Sub-pages of an item keep it active. */
    current: string;
    onselect?: (value: string) => void;
    /** Content above the destinations. */
    header?: Snippet;
    /** Content pinned to the bottom. */
    footer?: Snippet;
    /** Layout only. */
    style?: Style;
  }

  let { items, current, onselect, header, footer, style }: Props = $props();
</script>

<View style={{ ...navigationRailLook(currentTheme()), ...style }}>
  {#if header}
    <View style={{ ...navigationRailColumn, paddingBottom: 8 }}>{@render header()}</View>
  {/if}
  <View style={navigationRailItems}>
    {#each items as item (item.value)}
      <NavDestination {item} active={isCurrentItem(current, item.value)} shape="rail" {onselect} />
    {/each}
  </View>
  {#if footer}
    <View style={navigationRailColumn}>{@render footer()}</View>
  {/if}
</View>
