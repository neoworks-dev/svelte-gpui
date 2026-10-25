<!-- The non-modal sheets, which sit in the page instead of over it. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { BottomSheet, Button, SideSheet, currentTheme, typeStyle } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());
  let sideOpen = $state(false);
  let bottomOpen = $state(false);
</script>

<View style={{ flex: 1, minWidth: 0, gap: 10 }}>
  <Section title="Standard sheets">
    <View style={{ direction: "row", gap: 8 }}>
      <Button onpress={() => (sideOpen = !sideOpen)}>Toggle standard side sheet</Button>
      <Button onpress={() => (bottomOpen = !bottomOpen)}>Toggle standard bottom sheet</Button>
    </View>
  </Section>
  <View
    style={{
      direction: "row",
      flex: 1,
      minHeight: 0,
      radius: theme.radii.card,
      borderWidth: 1,
      borderColor: theme.colors.line,
      overflow: "hidden",
    }}
  >
    <View style={{ flex: 1, minWidth: 0, padding: 20 }}>
      <Text style={{ ...typeStyle("body"), color: theme.colors.muted }}>The page content. A standard sheet leaves it usable.</Text>
    </View>
    <SideSheet bind:open={sideOpen} title="Details" modal={false}>
      <Text style={{ ...typeStyle("body"), color: theme.colors.muted }}>Takes its own column.</Text>
    </SideSheet>
    <BottomSheet bind:open={bottomOpen} title="Now playing" modal={false}>
      <Text style={{ ...typeStyle("body"), color: theme.colors.muted }}>Sits at the bottom of the box.</Text>
    </BottomSheet>
  </View>
</View>
