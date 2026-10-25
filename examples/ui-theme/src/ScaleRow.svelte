<!-- One colour scale: its twelve steps side by side. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, typeStyle, type ScaleName, type ScaleStep } from "@neoworks/svelte-gpui-ui";

  let { name }: { name: ScaleName } = $props();

  const theme = $derived(currentTheme());
  const steps: ScaleStep[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
</script>

<View style={{ direction: "row", align: "center", gap: 0 }}>
  <Text style={{ ...typeStyle("caption"), color: theme.colors.muted, width: 56 }}>{name}</Text>
  {#each steps as step (step)}
    <View style={{ flex: 1, height: 30, background: theme.scales[name][step] }} />
  {/each}
</View>
