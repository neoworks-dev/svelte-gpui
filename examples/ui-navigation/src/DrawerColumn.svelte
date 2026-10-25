<!-- The navigation drawer, docked and modal. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { Button, currentTheme, NavigationDrawer, typeStyle, type NavSection } from "@neoworks/svelte-gpui-ui";
  import Frame from "./Frame.svelte";
  import { drawerItems, pageItems } from "./icons";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  const sections: NavSection[] = [
    { items: drawerItems },
    { heading: "Documentation", items: pageItems },
  ];

  const modalSections: NavSection[] = [
    { items: [{ label: "Overview", value: "/overview" }, { label: "Reports", value: "/reports" }] },
  ];

  let dockedCurrent = $state("/favourites");
  let modalCurrent = $state("/overview");
  let modalOpen = $state(false);
</script>

<View style={{ gap: 22 }}>
  <Section title="Docked drawer">
    <Frame height={520} style={{ direction: "row" }}>
      <NavigationDrawer {sections} current={dockedCurrent} title="NeoWorks" onselect={(value) => (dockedCurrent = value)} />
    </Frame>
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>current: {dockedCurrent}</Text>
  </Section>

  <Section title="Modal drawer">
    <Frame height={520}>
      <View style={{ padding: 16, gap: 12, align: "start" }}>
        <Button onpress={() => (modalOpen = true)}>Open the drawer</Button>
        <Text style={{ ...typeStyle("body"), color: theme.colors.muted }}>current: {modalCurrent}</Text>
        <View style={{ width: "full", height: 120, radius: theme.radii.card, background: theme.colors.lake }} />
        <View style={{ width: "full", height: 120, radius: theme.radii.card, background: theme.colors.larch }} />
      </View>
      <NavigationDrawer
        sections={modalSections}
        current={modalCurrent}
        title="Menu"
        modal
        bind:open={modalOpen}
        onselect={(value) => {
          modalCurrent = value;
          modalOpen = false;
        }}
      />
    </Frame>
  </Section>
</View>
