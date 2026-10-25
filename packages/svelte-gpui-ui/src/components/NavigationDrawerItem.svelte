<!-- One destination row of a NavigationDrawer: an optional icon, the label and an optional count. -->
<script lang="ts">
  import { Button, Icon, Text } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { iconForState, type DrawerItem } from "./navigation";
  import { drawerBadgeLook, drawerIconSize, drawerItemLook } from "./navigation-look";

  interface Props {
    item: DrawerItem;
    active: boolean;
    docked: boolean;
    onselect?: (value: string) => void;
  }

  let { item, active, docked, onselect }: Props = $props();

  const theme = $derived(currentTheme());
  const look = $derived(drawerItemLook(theme, { docked, active }));
  const icon = $derived(iconForState(item, active));
</script>

<Button style={look.style} hover={look.hover} active={look.active} focus={look.focus} onpress={() => onselect?.(item.value)}>
  {#if icon}
    <Icon svg={icon} size={drawerIconSize(docked)} />
  {/if}
  <Text style={{ flex: 1, minWidth: 0, truncate: true }}>{item.label}</Text>
  {#if item.badge}
    <Text style={drawerBadgeLook(theme)}>{item.badge}</Text>
  {/if}
</Button>
