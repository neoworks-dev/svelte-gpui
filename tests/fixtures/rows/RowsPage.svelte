<!--
  A long list whose rows are big enough to be drawn from a cache while the list scrolls, for checking by
  eye and by screenshot (scripts/shot.sh) that a cached row still hovers, presses, changes and grows:

    - hovering a row lights it, and the Count button lights more;
    - Count adds one to the row's counter, which must show at once while the row is scrolled;
    - Open adds a line under the row, so the row grows and the rows under it move down;
    - every fifth row holds a scroller of its own, which scrolls sideways and shows its bar.

  Build it with `bun tests/fixtures/rows/build.mjs`, run it with `scripts/shot.sh tests/fixtures/rows/dist/rows.js out.png`.
-->
<script lang="ts">
  import { Button, List, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";

  const items = Array.from({ length: 600 }, (_, id) => id);
  let counts = $state<Record<number, number>>({});
  let open = $state<Record<number, boolean>>({});
  const columns = ["artist", "album", "added", "length", "plays", "rating"];
</script>

<View style={{ height: 640, background: "#101014" }}>
  <List {items} key={(id: number) => id} estimatedRowHeight={48} style={{ flex: 1 }}>
    {#snippet row(id: number)}
      <View style={{ paddingX: 12 }}>
        <View
          style={{ direction: "row", align: "center", gap: 12, paddingY: 8, paddingX: 8, radius: 6, background: id % 2 ? "#18181c" : "#141418" }}
          hover={{ background: "#2d2a4a" }}
        >
          <Text style={{ width: 40, color: "#71717a" }}>{id}</Text>
          <View style={{ width: 36, height: 36, radius: 4, background: `hsl(${(id * 37) % 360}, 60%, 45%)` }} />
          <View style={{ flex: 1 }}>
            <Text style={{ fontWeight: "bold" }}>Song number {id}</Text>
            <Text style={{ color: "#a1a1aa" }}>Artist {id % 17}</Text>
          </View>
          {#each columns as column (column)}
            <View style={{ width: 54 }}>
              <Text style={{ color: "#71717a", fontSize: 11 }}>{column}</Text>
              <Text style={{ fontSize: 12 }}>{(id * column.length) % 97}</Text>
            </View>
          {/each}
          <Button
            style={{ paddingX: 10, paddingY: 4, radius: 4, background: "#3f3f46" }}
            hover={{ background: "#7c3aed" }}
            onpress={() => (counts[id] = (counts[id] ?? 0) + 1)}
          >
            <Text>Count {counts[id] ?? 0}</Text>
          </Button>
          <Button
            style={{ paddingX: 10, paddingY: 4, radius: 4, background: "#3f3f46" }}
            hover={{ background: "#7c3aed" }}
            onpress={() => (open[id] = !open[id])}
          >
            <Text>{open[id] ? "Close" : "Open"}</Text>
          </Button>
        </View>
        {#if id % 5 === 0}
          <!-- A scroller inside a row, which keeps its bar and its wheel where the row moved to. -->
          <Scroll horizontal style={{ height: 34 }}>
            <View style={{ direction: "row", gap: 6, paddingY: 4, paddingX: 8, width: 2600 }}>
              {#each Array.from({ length: 30 }, (_, chip) => chip) as chip (chip)}
                <View style={{ paddingX: 10, paddingY: 4, radius: 10, background: "#27272a", shrink: 0 }}>
                  <Text style={{ fontSize: 12, nowrap: true }}>tag {id}.{chip}</Text>
                </View>
              {/each}
            </View>
          </Scroll>
        {/if}
        {#if open[id]}
          <View style={{ paddingX: 60, paddingY: 10, background: "#1e1b3a", radius: 6 }}>
            <Text>The extra line of row {id}, which makes the row taller.</Text>
          </View>
        {/if}
      </View>
    {/snippet}
  </List>
</View>
