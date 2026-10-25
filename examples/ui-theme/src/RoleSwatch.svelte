<!-- A colour role: a chip of the colour beside its name and value. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, typeStyle, type ColorRole } from "@neoworks/svelte-gpui-ui";

  let { role }: { role: ColorRole } = $props();

  const theme = $derived(currentTheme());
</script>

<View style={{ direction: "row", align: "center", gap: 10, width: "33.3%", paddingBottom: 6 }}>
  <View
    style={{
      size: 30,
      radius: theme.radii.control,
      background: theme.colors[role],
      borderWidth: 1,
      borderColor: theme.colors.line,
    }}
  />
  <View style={{ gap: 1 }}>
    <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.text }}>{role}</Text>
    <Text style={{ fontFamily: "mono", fontSize: 11, color: theme.colors.faint }}>{theme.colors[role]}</Text>
  </View>
</View>
