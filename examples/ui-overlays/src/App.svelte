<!--
  Every NeoWorks overlay component, each opened from a button. The button at the top switches between light and dark.
-->
<script lang="ts">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, themeMode, toggleThemeMode, typeStyle } from "@neoworks/svelte-gpui-ui";
  import LayersSection from "./LayersSection.svelte";
  import MenuSection from "./MenuSection.svelte";
  import SelectSection from "./SelectSection.svelte";
  import StandardSheets from "./StandardSheets.svelte";
  import TooltipSection from "./TooltipSection.svelte";

  const theme = $derived(currentTheme());

  function toggleLabel(): string {
    if (themeMode() === "light") {
      return "Switch to dark";
    }
    return "Switch to light";
  }
</script>

<View style={{ size: "full", background: theme.colors.canvas, padding: 28, gap: 24 }}>
  <View style={{ direction: "row", align: "center", justify: "between" }}>
    <Text style={{ ...typeStyle("title"), color: theme.colors.text }}>NeoWorks overlays, {themeMode()}</Text>
    <Button
      onpress={toggleThemeMode}
      style={{ paddingX: 14, paddingY: 8, radius: theme.radii.control, background: theme.colors.fill, cursor: "pointer" }}
      hover={{ background: theme.colors.fillHover }}
    >
      <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.onFill }}>{toggleLabel()}</Text>
    </Button>
  </View>

  <View style={{ direction: "row", gap: 40, align: "start" }}>
    <MenuSection />
    <TooltipSection />
    <LayersSection />
  </View>

  <View style={{ direction: "row", gap: 40, flex: 1, minHeight: 0 }}>
    <SelectSection />
    <StandardSheets />
  </View>
</View>
