<!--
  Scrollers to try the wheel on, each with a line that says where it is:

  - a `Scroll` of eighty rows with a smaller `Scroll` inside it, which keeps the wheel until it is at its end;
  - a horizontal `Scroll`, and another vertical one;
  - a `List` of a hundred thousand rows of one height, taller than the host lays out at once;
  - a `List` of three thousand rows of different heights, which are measured as they are drawn.

  Drive it with `wheel:X,Y:DY` steps; see docs/testing.md. Each move of the outer, inner and measured scrollers
  is logged with its time, which shows how a notch arrives.
-->
<script lang="ts">
  import { List, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
  import type { ScrollPosition } from "@neoworks/svelte-gpui/ui";

  const rows = Array.from({ length: 80 }, (_, index) => index);
  const inner = Array.from({ length: 30 }, (_, index) => index);
  const tiles = Array.from({ length: 40 }, (_, index) => index);
  const fixed = Array.from({ length: 100_000 }, (_, index) => index);
  const varied = Array.from({ length: 3000 }, (_, index) => index);

  /** One line in the host's log per move, with the time, for measuring how a wheel notch arrives. */
  function logMove(name: string, position: ScrollPosition): void {
    console.log(`move ${name} ${performance.now().toFixed(1)} x=${position.x.toFixed(1)} y=${position.y.toFixed(1)}`);
  }

  let scrollY = $state(0);
  let horizontalX = $state(0);
  let fixedY = $state(0);
  let variedY = $state(0);
  let innerY = $state(0);

  const panel = { flex: 1, minWidth: 0, minHeight: 0, gap: 6 };
  const frame = { flex: 1, minHeight: 0, background: "#1c1c21", radius: 6 };
  const caption = { fontSize: 12, color: "#9a9aa4" };

  /** How many lines a row of the measured list has: one to five, the same each time. */
  function lines(index: number): number {
    return 1 + ((index * 7 + 3) % 5);
  }
</script>

<View style={{ flex: 1, padding: 12, gap: 12, background: "#111114" }}>
  <View style={{ flex: 1, minHeight: 0, direction: "row", gap: 12 }}>
    <View style={panel}>
      <Text style={caption}>Scroll, with a Scroll inside: y {Math.round(scrollY)}, inner {Math.round(innerY)}</Text>
      <Scroll
        style={frame}
        onscroll={(position: ScrollPosition) => {
          scrollY = position.y;
          logMove("outer", position);
        }}
      >
        {#each rows as row}
          <Text style={{ padding: 6 }}>Row {row}</Text>
          {#if row === 4}
            <Scroll
              style={{ height: 140, shrink: 0, background: "#26262c", margin: 6, radius: 4 }}
              onscroll={(position: ScrollPosition) => {
                innerY = position.y;
                logMove("inner", position);
              }}
            >
              {#each inner as item}
                <Text style={{ padding: 4, color: "#c4b5fd" }}>Inner {item}</Text>
              {/each}
            </Scroll>
          {/if}
        {/each}
      </Scroll>
    </View>
    <View style={panel}>
      <Text style={caption}>Horizontal Scroll: x {Math.round(horizontalX)}</Text>
      <Scroll
        horizontal
        style={{ ...frame, flex: undefined, height: 120, direction: "row", gap: 8, padding: 8 }}
        onscroll={(position: ScrollPosition) => (horizontalX = position.x)}
      >
        {#each tiles as tile}
          <View style={{ width: 90, shrink: 0, background: "#3b3b46", radius: 6, align: "center", justify: "center" }}>
            <Text>Tile {tile}</Text>
          </View>
        {/each}
      </Scroll>
      <Text style={caption}>Another Scroll, to wheel in</Text>
      <Scroll style={frame}>
        {#each rows as row}
          <Text style={{ padding: 6 }}>Row {row}</Text>
        {/each}
      </Scroll>
    </View>
  </View>
  <View style={{ flex: 1, minHeight: 0, direction: "row", gap: 12 }}>
    <View style={panel}>
      <Text style={caption}>List of 100,000 rows of 28 px: y {Math.round(fixedY)}</Text>
      <List items={fixed} rowHeight={28} style={frame} onscroll={(position: ScrollPosition) => (fixedY = position.y)}>
        {#snippet row(item: number)}
          <Text style={{ paddingX: 8, height: 28, align: "center", justify: "center" }}>Item {item}</Text>
        {/snippet}
      </List>
    </View>
    <View style={panel}>
      <Text style={caption}>List of 3,000 rows of different heights: y {Math.round(variedY)}</Text>
      <List
        items={varied}
        style={frame}
        onscroll={(position: ScrollPosition) => {
          variedY = position.y;
          logMove("varied", position);
        }}
      >
        {#snippet row(item: number)}
          <View style={{ padding: 8, borderColor: "#2c2c33", borderWidth: 1 }}>
            {#each Array.from({ length: lines(item) }) as _, line}
              <Text>Message {item}, line {line + 1}</Text>
            {/each}
          </View>
        {/snippet}
      </List>
    </View>
  </View>
</View>
