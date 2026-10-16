<!-- A hundred thousand rows, of which only the ones near the window exist. -->
<script lang="ts">
  import { Button, Input, List, Text, View } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import { caption, colors, field, fieldFocus, mono } from "../theme";

  interface Row {
    id: number;
    name: string;
    kind: string;
    size: number;
  }

  const kinds = ["svelte", "rust", "ts", "json", "md", "toml"];
  const words = ["render", "layout", "paint", "tree", "bridge", "event", "style", "frame", "view", "text", "image", "focus"];

  const rows: Row[] = Array.from({ length: 100_000 }, (_, id) => ({
    id,
    name: `${words[id % words.length]}-${words[(id * 7) % words.length]}-${id}`,
    kind: kinds[(id * 13) % kinds.length],
    size: (id * 7919) % 90_000,
  }));

  let query = $state("");
  let selected = $state<number | null>(null);
  const matching = $derived(query === "" ? rows : rows.filter((row) => row.name.includes(query)));

  const kindColor: Record<string, string> = {
    svelte: colors.amber,
    rust: colors.red,
    ts: colors.blue,
    json: colors.green,
    md: colors.dim,
    toml: colors.pink,
  };

  let list: ReturnType<typeof List> | undefined = $state();

  /** 100000 as "100,000": QuickJS has no Intl, so toLocaleString does not group digits. */
  const grouped = (count: number): string => String(count).replace(/\B(?=(\d{3})+$)/g, ",");
</script>

<Card title="List" note="Rows have a fixed height, so the list knows where each one sits and builds only what is near the window.">
  <View style={{ direction: "row", gap: 8, align: "center" }}>
    <Input bind:value={query} placeholder="Filter 100,000 rows" style={{ ...field, flex: 1 }} focus={fieldFocus} />
    <Button
      style={{ paddingX: 10, paddingY: 6, radius: 6, background: colors.raised }}
      hover={{ background: colors.hover }}
      onpress={() => list?.scrollToIndex(Math.floor(matching.length / 2))}
    >
      <Text>Jump to the middle</Text>
    </Button>
  </View>
  <Text style={caption}>
    {grouped(matching.length)} rows{selected !== null ? `, row ${selected} selected` : ""}
  </Text>
  <List
    bind:this={list}
    items={matching}
    rowHeight={30}
    key={(row) => row.id}
    style={{ height: 420, radius: 8, background: colors.window, borderWidth: 1, borderColor: colors.border }}
  >
    {#snippet row(item, index)}
      <Button
        style={{
          flex: 1,
          gap: 10,
          paddingX: 12,
          background: item.id === selected ? colors.accentSoft : index % 2 === 0 ? "transparent" : "#ffffff05",
          cursor: "default",
        }}
        hover={{ background: item.id === selected ? colors.accentSoft : colors.raised }}
        onpress={() => (selected = item.id)}
      >
        <Text style={{ ...mono, color: colors.faint, width: 56 }}>{item.id}</Text>
        <Text style={{ flex: 1, truncate: true, color: item.id === selected ? colors.accentText : colors.text }}>{item.name}</Text>
        <View style={{ paddingX: 6, radius: 4, background: colors.raised }}>
          <Text style={{ ...mono, fontSize: 11, color: kindColor[item.kind] }}>{item.kind}</Text>
        </View>
        <Text style={{ ...mono, color: colors.dim, width: 72, textAlign: "right" }}>{(item.size / 1024).toFixed(1)} KB</Text>
      </Button>
    {/snippet}
  </List>
</Card>
