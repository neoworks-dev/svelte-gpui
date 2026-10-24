<!--
  Things that must draw by themselves, in a page too big for one cached view, so that it is split into
  the regions below (the log on the far right holds most of the nodes). Every change here arrives a
  moment after the press that asked for it, because a scripted step redraws every view right after it
  runs, which would hide a view that is not redrawn.

  - "Add rows" appends rows to a `Tree`, each showing an `Image` of an SVG file, as Grove's explorer does.
  - "Add photos" appends rows whose picture is a large PNG, which the host scales on a thread and wakes the
    window when that is done.
  - "Type for me" sets the multiline `Input` inside a `Scroll`, as a restored draft would.
  - "Drop log" removes the log, so the page now fits in one cached view and the regions stop being views.
    The rows and the text above are only drawn if the host drops what it knew about those regions.
-->
<script lang="ts">
  import { Button, Image, Input, Scroll, Text, Tree, View } from "@neoworks/svelte-gpui/ui";
  import type { Layout } from "@neoworks/svelte-gpui/ui";

  interface Row {
    id: number;
    icon: string;
    name: string;
  }

  const FIELD_MAX_HEIGHT = 160;

  /** The icons the rows alternate between, as SVG files. */
  const icons = ["assets/folder.svg", "assets/file.svg"];

  /** Row `id`, with the next of the icons, or a photograph, which the host has to scale down on a thread. */
  function row(id: number, photo = false): Row {
    return { id, icon: photo ? "assets/photo.png" : icons[id % icons.length], name: `entry-${id}.ts` };
  }

  let rows = $state<Row[]>(Array.from({ length: 3 }, (_, id) => row(id)));
  let text = $state("");
  let showLog = $state(true);
  let scroller: ReturnType<typeof Scroll> | undefined = $state();

  /**
   * Appends four rows a moment later, as the answer to a request does, so that nothing the pointer or the
   * test steps do lands with them: a step redraws every view after it runs, which hides the problem.
   */
  function addRowsLater(photo = false): void {
    setTimeout(() => addRows(photo), 300);
  }

  /** Sets the field's text a moment later, as a restored draft or a dictation would, with nothing else changing on the page. */
  function typeLater(): void {
    setTimeout(() => (text = "hello there"), 300);
  }

  /** Appends four rows. */
  function addRows(photo = false): void {
    const first = rows.length;
    rows = [...rows, ...Array.from({ length: 4 }, (_, at) => row(first + at, photo))];
  }

  /** Follows the end of the text while it grows past its cap, as the chat composer does. */
  function grown(box: Layout): void {
    if (box.height > FIELD_MAX_HEIGHT) {
      scroller?.scrollTo(box.height);
    }
  }
</script>

<View style={{ flex: 1, minWidth: 0, minHeight: 0, padding: 12, gap: 12, background: "#16181d" }}>
  <View style={{ direction: "row", gap: 8, align: "center" }}>
    <Button onpress={() => addRowsLater()} style={{ padding: 8, background: "#2d3340", radius: 6 }}>
      <Text style={{ color: "#fff" }}>Add rows</Text>
    </Button>
    <Button onpress={() => addRowsLater(true)} style={{ padding: 8, background: "#2d3340", radius: 6 }}>
      <Text style={{ color: "#fff" }}>Add photos</Text>
    </Button>
    <Button onpress={typeLater} style={{ padding: 8, background: "#2d3340", radius: 6 }}>
      <Text style={{ color: "#fff" }}>Type for me</Text>
    </Button>
    <Button onpress={() => (showLog = false)} style={{ padding: 8, background: "#2d3340", radius: 6 }}>
      <Text style={{ color: "#fff" }}>Drop log</Text>
    </Button>
  </View>
  <View style={{ direction: "row", flex: 1, minWidth: 0, minHeight: 0, gap: 12 }}>
    <View style={{ flex: 1, minWidth: 0, minHeight: 0, padding: 8, background: "#1e2128", radius: 8 }}>
      {#each Array.from({ length: 8 }, (_, at) => at) as at (at)}
        <View style={{ direction: "row", gap: 8 }}>
          <Text style={{ color: "#9aa3b2" }}>folder {at}</Text>
          <Text style={{ color: "#5b6270" }}>header</Text>
        </View>
      {/each}
      <Tree items={rows} key={(entry: Row) => entry.id} rowHeight={24} style={{ flex: 1 }}>
        {#snippet row(entry: Row)}
          <View style={{ direction: "row", gap: 8, align: "center", flex: 1, height: "full" }}>
            <Image src={entry.icon} fit="contain" style={{ width: 14, height: 14, shrink: 0 }} />
            <Text style={{ color: "#e6e9ef" }}>{entry.name}</Text>
          </View>
        {/snippet}
      </Tree>
    </View>
    <View style={{ flex: 1, minWidth: 0, minHeight: 0, gap: 4, padding: 8, background: "#1e2128", radius: 8 }}>
      {#each Array.from({ length: 12 }, (_, at) => at) as at (at)}
        <View style={{ direction: "row", gap: 8 }}>
          <Text style={{ color: "#9aa3b2" }}>line {at}</Text>
          <Text style={{ color: "#5b6270" }}>filler</Text>
        </View>
      {/each}
      <View style={{ padding: 8, background: "#2d3340", radius: 8 }}>
        <Scroll bind:this={scroller} style={{ maxHeight: FIELD_MAX_HEIGHT }}>
          <View onlayout={grown}>
            <Input bind:value={text} multiline rows={2} placeholder="Type here" style={{ color: "#fff", lineHeight: 20 }} />
          </View>
        </Scroll>
      </View>
    </View>
    {#if showLog}
      <Scroll style={{ width: 120, background: "#1e2128", radius: 8 }}>
        {#each Array.from({ length: 1500 }, (_, at) => at) as at (at)}
          <Text style={{ color: "#5b6270" }}>log {at}</Text>
        {/each}
      </Scroll>
    {/if}
  </View>
</View>
