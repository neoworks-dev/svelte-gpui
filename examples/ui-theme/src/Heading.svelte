<!-- A section's title, in the theme's label type. -->
<script lang="ts">
  import { Text } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, typeStyle } from "@neoworks/svelte-gpui-ui";

  let { title }: { title: string } = $props();

  const theme = $derived(currentTheme());
</script>

<Text style={{ ...typeStyle("label"), color: theme.colors.muted, uppercase: true }}>{title}</Text>
