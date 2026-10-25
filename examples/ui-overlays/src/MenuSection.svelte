<!-- Menus: one with groups, shortcuts and checks, one on an icon button opening toward the end. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import copyIcon from "@phosphor-icons/core/regular/copy.svg";
  import dotsIcon from "@phosphor-icons/core/regular/dots-three.svg";
  import pencilIcon from "@phosphor-icons/core/regular/pencil-simple.svg";
  import trashIcon from "@phosphor-icons/core/regular/trash.svg";
  import { Button, IconButton, Menu, currentTheme, typeStyle, type MenuEntry } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());
  let actionsOpen = $state(false);
  let moreOpen = $state(false);
  let lastAction = $state("nothing yet");
  let showGrid = $state(true);
  let showRulers = $state(false);

  const actionItems: MenuEntry[] = $derived([
    { type: "heading", label: "Edit" },
    { label: "Rename", icon: pencilIcon, shortcut: "F2", onselect: () => (lastAction = "Rename") },
    { label: "Copy", icon: copyIcon, shortcut: "Ctrl+C", onselect: () => (lastAction = "Copy") },
    { type: "divider" },
    { type: "heading", label: "View" },
    { label: "Show grid", selected: showGrid, onselect: () => (showGrid = !showGrid) },
    { label: "Show rulers", selected: showRulers, onselect: () => (showRulers = !showRulers) },
    { type: "divider" },
    { label: "Archive", disabled: true },
    { label: "Delete album", icon: trashIcon, danger: true, onselect: () => (lastAction = "Delete album") },
  ]);

  const moreItems: MenuEntry[] = [
    { label: "Share", onselect: () => (lastAction = "Share") },
    { label: "Move to", onselect: () => (lastAction = "Move to") },
    { label: "Details", shortcut: "I", onselect: () => (lastAction = "Details") },
  ];
</script>

<Section title="Menu">
  <View style={{ gap: 12, align: "start" }}>
    <Menu bind:open={actionsOpen} items={actionItems}>
      {#snippet trigger({ onpress })}
        <Button {onpress}>Actions</Button>
      {/snippet}
    </Menu>
    <Menu bind:open={moreOpen} items={moreItems} placement="bottom-end">
      {#snippet trigger({ onpress })}
        <IconButton icon={dotsIcon} label="More" {onpress} />
      {/snippet}
    </Menu>
    <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.muted }}>Last action: {lastAction}</Text>
  </View>
</Section>
