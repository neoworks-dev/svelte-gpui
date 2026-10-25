<!--
  One look drawn four times, at rest and with its hover, pressed and focus styles laid on, side by side. The host
  only applies hover and pressed styles to a real pointer, so this shows what each state looks like on a still screen.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, typeStyle, type StateStyles } from "@neoworks/svelte-gpui-ui";

  let { caption, look, children }: { caption: string; look: StateStyles; children: Snippet } = $props();

  const theme = $derived(currentTheme());
  const states = $derived([
    { name: "rest", style: look.style },
    { name: "hover", style: { ...look.style, ...look.hover } },
    { name: "pressed", style: { ...look.style, ...look.hover, ...look.active } },
    { name: "focus", style: { ...look.style, ...look.focus } },
  ]);
</script>

<View style={{ direction: "row", align: "center", gap: 10 }}>
  <Text style={{ ...typeStyle("caption"), color: theme.colors.faint, width: 64 }}>{caption}</Text>
  {#each states as state (state.name)}
    <View style={{ ...state.style, alignSelf: "center" }}>{@render children()}</View>
  {/each}
</View>
