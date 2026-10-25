<!-- Top app bars, the bottom nav bar and floating action buttons. -->
<script lang="ts">
  import { Scroll, Text, View, type ScrollPosition } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, Fab, IconButton, NavBar, TopAppBar, typeStyle } from "@neoworks/svelte-gpui-ui";
  import Frame from "./Frame.svelte";
  import { arrowLeft, dots, list, magnifyingGlass, navItems, pencil, plus, star } from "./icons";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());
  const blockColors = $derived([theme.colors.accent, theme.colors.lake, theme.colors.larch, theme.colors.rock, theme.colors.pine]);

  let scrolled = $state(false);
  let current = $state("/library/albums");
  let extended = $state(true);

  function followScroll(position: ScrollPosition): void {
    scrolled = position.y > 0;
  }
</script>

<View style={{ gap: 22 }}>
  <Section title="Top app bar">
    <Frame>
      <TopAppBar title="Small" variant="small">
        {#snippet leading()}<IconButton icon={list} label="Menu" />{/snippet}
        {#snippet trailing()}<IconButton icon={magnifyingGlass} label="Search" /><IconButton icon={dots} label="More" />{/snippet}
      </TopAppBar>
    </Frame>
    <Frame>
      <TopAppBar title="Centered" variant="centered">
        {#snippet leading()}<IconButton icon={arrowLeft} label="Back" />{/snippet}
        {#snippet trailing()}<IconButton icon={star} label="Star" />{/snippet}
      </TopAppBar>
    </Frame>
    <Frame>
      <TopAppBar title="Medium" variant="medium">
        {#snippet leading()}<IconButton icon={arrowLeft} label="Back" />{/snippet}
        {#snippet trailing()}<IconButton icon={dots} label="More" />{/snippet}
      </TopAppBar>
    </Frame>
    <Frame>
      <TopAppBar title="Large title" variant="large">
        {#snippet leading()}<IconButton icon={arrowLeft} label="Back" />{/snippet}
        {#snippet trailing()}<IconButton icon={dots} label="More" />{/snippet}
      </TopAppBar>
    </Frame>
    <Frame>
      <TopAppBar title="Large, scrolled" variant="large" scrolled>
        {#snippet leading()}<IconButton icon={arrowLeft} label="Back" />{/snippet}
        {#snippet trailing()}<IconButton icon={dots} label="More" />{/snippet}
      </TopAppBar>
    </Frame>
  </Section>

  <Section title="Top app bar over a scrolling page">
    <Frame height={300}>
      <Scroll style={{ flex: 1 }} onscroll={followScroll}>
        <View style={{ paddingTop: 64, paddingBottom: 16, paddingX: 16, gap: 12 }}>
          {#each blockColors as color, index (index)}
            <View style={{ height: 96, radius: theme.radii.card, background: color, padding: 12 }}>
              <Text style={{ ...typeStyle("body"), color: theme.colors.onAccent }}>Block {index + 1}</Text>
            </View>
          {/each}
        </View>
      </Scroll>
      <TopAppBar
        title="Large title"
        variant="large"
        {scrolled}
        style={{ position: "absolute", top: 0, left: 0, right: 0 }}
      >
        {#snippet leading()}<IconButton icon={arrowLeft} label="Back" />{/snippet}
      </TopAppBar>
    </Frame>
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>scrolled: {scrolled}</Text>
  </Section>

  <Section title="Nav bar">
    <Frame height={96} style={{ justify: "center" }}>
      <NavBar items={navItems} {current} onselect={(value) => (current = value)} />
    </Frame>
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>current: {current}</Text>
  </Section>

  <Section title="Floating action button">
    <View style={{ direction: "row", align: "center", gap: 16, wrap: true, padding: 8 }}>
      <Fab icon={plus} label="Compose" size="sm" />
      <Fab icon={plus} label="Compose" />
      <Fab icon={plus} label="Compose" size="lg" />
      <Fab icon={pencil} label="Compose" variant="surface" />
      <Fab icon={plus} label="Upload" variant="accent" />
    </View>
    <View style={{ direction: "row", align: "center", gap: 16, padding: 8 }}>
      <Fab icon={pencil} label="Compose" extended={extended} onpress={() => (extended = !extended)} />
      <Fab icon={plus} label="Upload" variant="accent" extended />
      <Fab icon={star} label="Starred" variant="surface" extended />
    </View>
  </Section>
</View>
