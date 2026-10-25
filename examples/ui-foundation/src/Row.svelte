<!-- Examples side by side, with an optional caption on the left. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, typeStyle } from "@neoworks/svelte-gpui-ui";

  let { caption, children }: { caption?: string; children: Snippet } = $props();

  const theme = $derived(currentTheme());
</script>

<View style={{ direction: "row", align: "center", gap: 10, wrap: true }}>
  {#if caption}
    <Text style={{ ...typeStyle("caption"), color: theme.colors.faint, width: 64 }}>{caption}</Text>
  {/if}
  {@render children()}
</View>
