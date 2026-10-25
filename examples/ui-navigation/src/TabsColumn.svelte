<!-- Tabs in both variants, and the navigation rail. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, Fab, IconButton, NavigationRail, Tabs, typeStyle, type TabItem } from "@neoworks/svelte-gpui-ui";
  import houseIcon from "@phosphor-icons/core/regular/house.svg";
  import houseFill from "@phosphor-icons/core/fill/house-fill.svg";
  import imagesIcon from "@phosphor-icons/core/regular/images.svg";
  import imagesFill from "@phosphor-icons/core/fill/images-fill.svg";
  import userIcon from "@phosphor-icons/core/regular/user.svg";
  import userFill from "@phosphor-icons/core/fill/user-fill.svg";
  import Frame from "./Frame.svelte";
  import { list, plus, railItems } from "./icons";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  const sectionTabs: TabItem[] = [
    { id: "home", label: "Home", icon: houseIcon, selectedIcon: houseFill },
    { id: "photos", label: "Photos", icon: imagesIcon, selectedIcon: imagesFill, count: 24 },
    { id: "people", label: "People", icon: userIcon, selectedIcon: userFill },
  ];
  const textTabs: TabItem[] = [
    { id: "all", label: "All" },
    { id: "unread", label: "Unread", count: 8 },
    { id: "archived", label: "Archived", disabled: true },
    { id: "spam", label: "Spam" },
  ];

  let sectionValue = $state("photos");
  let textValue = $state("all");
  let secondaryValue = $state("unread");
  let wideValue = $state("home");
  let railCurrent = $state("/library/albums");
</script>

<View style={{ gap: 22 }}>
  <Section title="Primary tabs with icons">
    <Tabs tabs={sectionTabs} bind:value={sectionValue} />
  </Section>

  <Section title="Primary tabs, text only">
    <Tabs tabs={textTabs} bind:value={textValue} />
  </Section>

  <Section title="Secondary tabs">
    <Tabs tabs={textTabs} bind:value={secondaryValue} variant="secondary" />
  </Section>

  <Section title="Full width, with a panel">
    <Tabs tabs={sectionTabs} bind:value={wideValue} fullWidth>
      {#snippet panel(id)}
        <Text style={{ ...typeStyle("body"), color: theme.colors.text, padding: 16 }}>Content of the {id} tab.</Text>
      {/snippet}
    </Tabs>
  </Section>

  <Section title="Navigation rail">
    <Frame height={560} style={{ direction: "row" }}>
      <NavigationRail items={railItems} current={railCurrent} onselect={(value) => (railCurrent = value)}>
        {#snippet header()}
          <IconButton icon={list} label="Menu" />
          <Fab icon={plus} label="Compose" size="sm" variant="surface" />
        {/snippet}
        {#snippet footer()}
          <IconButton icon={userIcon} label="Account" variant="tonal" />
        {/snippet}
      </NavigationRail>
      <View style={{ flex: 1, padding: 16 }}>
        <Text style={{ ...typeStyle("body"), color: theme.colors.muted }}>current: {railCurrent}</Text>
      </View>
    </Frame>
  </Section>
</View>
