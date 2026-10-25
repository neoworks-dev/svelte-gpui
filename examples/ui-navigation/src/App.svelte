<!--
  Every NeoWorks navigation component in each state. The button switches between light and dark.
-->
<script lang="ts">
  import { Button, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, themeMode, toggleThemeMode, typeStyle } from "@neoworks/svelte-gpui-ui";
  import BarsColumn from "./BarsColumn.svelte";
  import DrawerColumn from "./DrawerColumn.svelte";
  import TabsColumn from "./TabsColumn.svelte";

  const theme = $derived(currentTheme());

  function toggleLabel(): string {
    if (themeMode() === "light") {
      return "Switch to dark";
    }
    return "Switch to light";
  }
</script>

<View style={{ size: "full", background: theme.colors.canvas, padding: 28, gap: 20 }}>
  <View style={{ direction: "row", align: "center", justify: "between" }}>
    <Text style={{ ...typeStyle("title"), color: theme.colors.text }}>NeoWorks navigation, {themeMode()}</Text>
    <Button
      onpress={toggleThemeMode}
      style={{ paddingX: 14, paddingY: 8, radius: theme.radii.control, background: theme.colors.fill, cursor: "pointer" }}
      hover={{ background: theme.colors.fillHover }}
    >
      <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.onFill }}>{toggleLabel()}</Text>
    </Button>
  </View>

  <View style={{ direction: "row", gap: 32, flex: 1, minHeight: 0 }}>
    <Scroll style={{ flex: 1, padding: 4 }}><BarsColumn /></Scroll>
    <Scroll style={{ flex: 1, padding: 4 }}><TabsColumn /></Scroll>
    <Scroll style={{ flex: 1, padding: 4 }}><DrawerColumn /></Scroll>
  </View>
</View>
