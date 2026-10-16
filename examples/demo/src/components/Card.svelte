<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { caption, card, cardTitle } from "../theme";

  interface Props {
    title: string;
    note?: string;
    style?: Style;
    children: Snippet;
  }

  let { title, note, style, children }: Props = $props();
</script>

<View style={{ ...card, ...style }}>
  <View style={{ gap: 2 }}>
    <Text style={cardTitle}>{title}</Text>
    {#if note}<Text style={caption}>{note}</Text>{/if}
  </View>
  {@render children()}
</View>
