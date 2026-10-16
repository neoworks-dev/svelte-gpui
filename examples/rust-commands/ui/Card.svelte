<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { card, colors } from "./theme";

  interface Props {
    title: string;
    note: string;
    children: Snippet;
  }

  let { title, note, children }: Props = $props();
</script>

<View style={card}>
  <View style={{ gap: 2 }}>
    <Text style={{ fontSize: 13, fontWeight: "semibold" }}>{title}</Text>
    <Text style={{ fontSize: 12, color: colors.faint }}>{note}</Text>
  </View>
  {@render children()}
</View>
