<!-- A titled group of examples. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, typeStyle } from "@neoworks/svelte-gpui-ui";

  let { title, children }: { title: string; children: Snippet } = $props();

  const theme = $derived(currentTheme());
</script>

<View style={{ gap: 10 }}>
  <Text style={{ ...typeStyle("label"), color: theme.colors.muted, uppercase: true }}>{title}</Text>
  {@render children()}
</View>
