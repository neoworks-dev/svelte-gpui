<script lang="ts">
  import { Button, List, Text, View } from "@neoworks/svelte-gpui/ui";

  let { target }: { target: string } = $props();
  let count = $state(0);
  const items = Array.from({ length: 50 }, (_, index) => `row ${index}`);
</script>

<View style={{ flex: 1, padding: 8, gap: 6 }}>
  <Text>Running on {target}</Text>
  <Button onpress={() => count++}><Text>Pressed {count}</Text></Button>
  <List {items} rowHeight={20} style={{ flex: 1 }}>
    {#snippet row(item)}<Text>{item}</Text>{/snippet}
  </List>
</View>

<style>
  :global(body) {
    margin: 0;
  }
</style>
