<!-- A bordered box that clips what it holds, standing in for a screen. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "@neoworks/svelte-gpui-ui";

  let { height, style, children }: { height?: number; style?: Style; children: Snippet } = $props();

  const theme = $derived(currentTheme());
</script>

<View
  style={{
    position: "relative",
    height,
    overflow: "hidden",
    radius: theme.radii.card,
    borderWidth: 1,
    borderColor: theme.colors.line,
    background: theme.colors.canvas,
    ...style,
  }}
>
  {@render children()}
</View>
