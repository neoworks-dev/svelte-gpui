<!-- Three lists, for tests/list.test.ts: a chat of rows of unknown height, a plain one, and one of fixed rows. -->
<script lang="ts">
  import { List, Text, View } from "@neoworks/svelte-gpui/ui";

  let chat = $state.raw(Array.from({ length: 5000 }, (_, id) => id));
  let list: { scrollToIndex(index: number, options: { align: "end" }): void } | undefined = $state();

  // What the test drives, since nothing can click here.
  (globalThis as any).__page = {
    append: () => (chat = [...chat, chat.length]),
    toEnd: () => list?.scrollToIndex(chat.length - 1, { align: "end" }),
  };
</script>

<View style={{ height: 600 }}>
  <List bind:this={list} items={chat} key={(id: number) => id} stickToEnd style={{ flex: 1 }}>
    {#snippet row(id: number)}<Text>chat {id}</Text>{/snippet}
  </List>
</View>
<View style={{ height: 600 }}>
  <List items={chat} key={(id: number) => id} style={{ flex: 1 }}>
    {#snippet row(id: number)}<Text>plain {id}</Text>{/snippet}
  </List>
</View>
<View style={{ height: 600 }}>
  <List items={chat} rowHeight={20} style={{ flex: 1 }}>
    {#snippet row(id: number)}<Text>fixed {id}</Text>{/snippet}
  </List>
</View>
