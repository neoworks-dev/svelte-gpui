<!--
  Five lists, for tests/list.test.ts: a chat of rows of unknown height, a plain one, one of fixed rows,
  a chat of tall rows, as long messages are, and one whose rows count how often they are worked out.
-->
<script lang="ts">
  import { List, Text, View } from "@neoworks/svelte-gpui/ui";

  let chat = $state.raw(Array.from({ length: 5000 }, (_, id) => id));
  const tall = Array.from({ length: 60 }, (_, id) => id);
  const counted = Array.from({ length: 200 }, (_, id) => id);
  let list: { scrollToIndex(index: number, options: { align: "end" }): void } | undefined = $state();

  // What the test drives, since nothing can click here.
  const page = {
    append: () => (chat = [...chat, chat.length]),
    toEnd: () => list?.scrollToIndex(chat.length - 1, { align: "end" }),
    /** How many times a row of the counted list has worked out its text. */
    evaluations: 0,
  };
  (globalThis as any).__page = page;

  function tally(id: number): string {
    page.evaluations++;
    return `counted ${id}`;
  }
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
<View style={{ height: 600 }}>
  <List items={tall} key={(id: number) => id} stickToEnd style={{ flex: 1 }}>
    {#snippet row(id: number)}<Text>tall {id}</Text>{/snippet}
  </List>
</View>
<View style={{ height: 600 }}>
  <List items={counted} key={(id: number) => id} style={{ flex: 1 }}>
    {#snippet row(id: number)}<Text>{tally(id)}</Text>{/snippet}
  </List>
</View>
