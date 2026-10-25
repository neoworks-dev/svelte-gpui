<!--
  The bottom bar of a phone-sized layout: three to five destinations of equal importance in a floating pill.
  `current` names the active one; sub-pages of an item keep it active.
-->
<script lang="ts">
  import { View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import NavDestination from "./NavDestination.svelte";
  import { isCurrentItem, type NavItem } from "./navigation";
  import { navBarLook } from "./navigation-look";

  interface Props {
    /** Three to five destinations of equal importance. */
    items: NavItem[];
    /** The `value` of the active destination. Sub-pages of an item keep it active. */
    current: string;
    onselect?: (value: string) => void;
    /** Layout only, such as the margins. */
    style?: Style;
  }

  let { items, current, onselect, style }: Props = $props();
</script>

<View style={{ ...navBarLook(currentTheme()), ...style }}>
  {#each items as item (item.value)}
    <NavDestination {item} active={isCurrentItem(current, item.value)} shape="bar" {onselect} />
  {/each}
</View>
