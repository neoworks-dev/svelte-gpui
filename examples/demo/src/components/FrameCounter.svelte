<!--
  Frames drawn in the last second, from the host. GPUI only draws when something
  changed, so a still window shows the two frames this counter's own update costs;
  dragging, scrolling or an animation shows the real rate.
-->
<script lang="ts">
  import { frameStats } from "@neoworks/svelte-gpui";
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { colors, mono } from "../theme";

  let fps = $state(0);
  let buildMs = $state(0);

  $effect(() => {
    const timer = setInterval(() => {
      const stats = frameStats();
      fps = stats.fps;
      buildMs = stats.buildMs;
    }, 500);
    return () => clearInterval(timer);
  });

  const tone = $derived(fps >= 50 ? colors.green : fps >= 25 ? colors.amber : colors.dim);
</script>

<View style={{ direction: "row", align: "center", gap: 8, paddingX: 10, paddingY: 6, radius: 6, background: colors.surface, borderWidth: 1, borderColor: colors.border }}>
  <View style={{ size: 6, radius: 3, background: tone }} />
  <Text style={{ ...mono, color: colors.text }}>{fps} fps</Text>
  <View style={{ flex: 1 }} />
  <Text style={{ ...mono, fontSize: 11, color: colors.faint }}>{buildMs.toFixed(2)} ms</Text>
</View>
