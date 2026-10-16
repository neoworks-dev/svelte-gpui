<!-- A hundred thousand rows, of which only the ones near the window exist. -->
<script lang="ts">
  import { files, type FileEntry } from "@neoworks/svelte-gpui";
  import { Button, Icon, Input, List, Text, Tree, View } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import Segmented from "../components/Segmented.svelte";
  import folderIcon from "@phosphor-icons/core/regular/folder.svg";
  import fileIcon from "@phosphor-icons/core/regular/file.svg";
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

  // Rows that differ in height: a chat of a hundred thousand messages of random length.
  interface Message {
    id: number;
    author: string;
    text: string;
  }

  const people = ["Ada", "Grace", "Linus", "Margaret", "Edsger"];
  const phrases = [
    "the frame is built once", "layout comes out of flexbox", "scrolling keeps the rows in view",
    "a row is measured as it is drawn", "that sounds right", "I will look at it tomorrow", "can we ship it",
    "the host draws what the app builds", "it works on my machine", "good morning", "no DOM, no CSS", "see the attached",
  ];

  /** A repeatable number from 0 up to but not including `limit`, for an id. */
  const roll = (id: number, salt: number, limit: number): number => {
    let value = (Math.abs(id) * 2654435761 + salt * 40503) >>> 0;
    value = (value ^ (value >>> 15)) >>> 0;
    value = Math.imul(value, 2246822519) >>> 0;
    value = (value ^ (value >>> 13)) >>> 0;
    return value % limit;
  };

  function message(id: number): Message {
    const count = 1 + (roll(id, 1, 10) < 6 ? roll(id, 2, 4) : roll(id, 3, 40));
    const text = Array.from({ length: count }, (_, at) => phrases[roll(id, 10 + at, phrases.length)]).join(", ");
    return { id, author: people[roll(id, 4, people.length)], text: text[0].toUpperCase() + text.slice(1) + "." };
  }

  // The list holds message numbers; each message is made when its row is drawn.
  let messages = $state.raw<number[]>(Array.from({ length: 100_000 }, (_, id) => id));
  let oldest = 0;
  let newest = 99_999;
  let chat: ReturnType<typeof List<number>> | undefined = $state();

  const addMessage = (): void => {
    messages = [...messages, ++newest];
  };
  const loadOlder = (): void => {
    const older = Array.from({ length: 20 }, () => --oldest);
    messages = [...older, ...messages];
  };

  // A tree: a folder structure that exists only as far as it is opened.
  interface Node {
    path: string;
    name: string;
    folder: boolean;
    size?: number;
  }

  const verbs = ["render", "layout", "paint", "bridge", "event", "style", "frame", "view"];

  /** The children of a generated folder, made when it is asked for. */
  function generated(parent: Node): Node[] | undefined {
    if (!parent.folder) {
      return undefined;
    }
    const depth = parent.path.split("/").length;
    const count = 3 + roll(parent.path.length * 31 + depth, 5, 9);
    return Array.from({ length: count }, (_, at) => {
      const folder = depth < 7 && roll(parent.path.length + at, 6, 3) === 0;
      const base = `${verbs[roll(at + depth, 7, verbs.length)]}-${at}`;
      const name = folder ? base : `${base}.${["rs", "ts", "svelte", "md"][roll(at, 8, 4)]}`;
      return { path: `${parent.path}/${name}`, name, folder, size: folder ? undefined : roll(at + depth, 9, 90_000) };
    });
  }

  const generatedRoots: Node[] = Array.from({ length: 6 }, (_, at) => ({
    path: `/${["src", "docs", "host", "runtime", "examples", "scripts"][at]}`,
    name: ["src", "docs", "host", "runtime", "examples", "scripts"][at],
    folder: true,
  }));

  let source = $state<"generated" | "disk">("generated");
  let treeSelected = $state<string | number | undefined>(undefined);
  let expanded = $state(new Set<string | number>());
  let activated = $state("");
  let diskRoots = $state<Node[]>([]);
  let loaded = $state<Record<string, Node[]>>({});
  let diskNote = $state("");

  const asNode = (entry: FileEntry): Node => ({
    path: entry.path,
    name: entry.name,
    folder: entry.kind === "directory",
    size: entry.size,
  });

  async function loadDirectory(path: string): Promise<Node[]> {
    return (await files.list(path)).map(asNode);
  }

  async function showDisk(path?: string): Promise<void> {
    diskNote = "";
    try {
      const root = path ?? `${(await files.paths()).data}/svelte-gpui-demo`;
      await files.mkdir(root, { recursive: true });
      loaded = {};
      expanded = new Set();
      diskRoots = await loadDirectory(root);
      diskNote = root;
    } catch (error) {
      diskNote = (error as Error).message;
    }
  }

  async function pickFolder(): Promise<void> {
    const picked = await files.pickFiles({ directories: true });
    if (picked?.[0]) {
      source = "disk";
      await showDisk(picked[0]);
    }
  }

  async function expandDisk(node: Node): Promise<void> {
    if (loaded[node.path] === undefined) {
      loaded = { ...loaded, [node.path]: await loadDirectory(node.path) };
    }
  }

  $effect(() => {
    if (source === "disk" && diskRoots.length === 0 && diskNote === "") {
      void showDisk();
    }
  });

  const treeRoots = $derived(source === "generated" ? generatedRoots : diskRoots);
  const childrenOf = (node: Node): Node[] | undefined => (source === "generated" ? generated(node) : loaded[node.path]);

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

<View style={{ gap: 16, marginTop: 16 }}>
  <Card title="Rows of different heights" note="No rowHeight: each row is measured as it is drawn, and the ones not drawn yet count as the average. When a row above the window turns out taller than guessed, the list scrolls by the difference, so nothing in view moves. A hundred thousand messages.">
    <View style={{ direction: "row", gap: 8, wrap: true }}>
      {#each [{ label: "Top", go: () => chat?.scrollToIndex(0) }, { label: "Jump to message 50,000", go: () => chat?.scrollToIndex(50_000) }, { label: "Jump to the end", go: () => chat?.scrollToIndex(messages.length - 1, { align: "end" }) }] as entry (entry.label)}
        <Button style={{ paddingX: 10, paddingY: 6, radius: 6, background: colors.raised }} hover={{ background: colors.hover }} onpress={entry.go}>
          <Text>{entry.label}</Text>
        </Button>
      {/each}
      <Button style={{ paddingX: 10, paddingY: 6, radius: 6, background: colors.raised }} hover={{ background: colors.hover }} onpress={addMessage}>
        <Text>Add a message</Text>
      </Button>
      <Button style={{ paddingX: 10, paddingY: 6, radius: 6, background: colors.raised }} hover={{ background: colors.hover }} onpress={loadOlder}>
        <Text>Load 20 older</Text>
      </Button>
    </View>
    <Text style={caption}>{grouped(messages.length)} messages. Scrolled to the end, the list stays at the end as messages are added.</Text>
    <List
      bind:this={chat}
      items={messages}
      key={(id) => id}
      estimatedRowHeight={56}
      stickToEnd
      style={{ height: 380, radius: 8, background: colors.window, borderWidth: 1, borderColor: colors.border }}
    >
      {#snippet row(id)}
        {@const item = message(id)}
        <View style={{ direction: "row", gap: 10, paddingX: 12, paddingY: 8 }}>
          <View style={{ size: 28, radius: 14, shrink: 0, align: "center", justify: "center", background: colors.raised }}>
            <Text style={{ fontWeight: "semibold", color: colors.accentText }}>{item.author[0]}</Text>
          </View>
          <View style={{ flex: 1, minWidth: 0, gap: 2 }}>
            <View style={{ direction: "row", gap: 8, align: "baseline" }}>
              <Text style={{ fontWeight: "semibold" }}>{item.author}</Text>
              <Text style={{ ...mono, fontSize: 11, color: colors.faint }}>#{item.id}</Text>
            </View>
            <Text style={{ color: colors.dim }}>{item.text}</Text>
          </View>
        </View>
      {/snippet}
    </List>
  </Card>

  <Card title="Tree" note="A List underneath, so only the rows of open folders in view exist. Click to select, click the arrow or press Right to open, Enter to activate. Up, Down, Left, Home and End work while it has focus.">
    <View style={{ direction: "row", gap: 12, align: "center" }}>
      <Segmented options={["generated", "disk"]} bind:value={source} />
      <Button style={{ paddingX: 10, paddingY: 4, radius: 6, background: colors.raised }} hover={{ background: colors.hover }} onpress={pickFolder}>
        <Text style={{ fontSize: 12 }}>Open a folder…</Text>
      </Button>
      <Text style={{ ...caption, flex: 1, minWidth: 0, truncate: true }}>
        {source === "generated" ? "Folders are made as they open: millions of nodes, none of them built until asked for." : diskNote}
      </Text>
    </View>
    <Tree
      items={treeRoots}
      key={(node) => node.path}
      {childrenOf}
      isBranch={(node) => node.folder}
      bind:expanded
      bind:selected={treeSelected}
      onexpand={source === "disk" ? expandDisk : undefined}
      onactivate={(node) => (activated = node.folder ? `Toggled ${node.name}` : `Opened ${node.name}`)}
      style={{ height: 320, radius: 8, background: colors.window, borderWidth: 1, borderColor: colors.border }}
    >
      {#snippet row(node, info)}
        <View style={{ direction: "row", align: "center", gap: 8 }}>
          <Icon svg={node.folder ? folderIcon : fileIcon} size={15} color={node.folder ? colors.amber : colors.dim} />
          <Text style={{ flex: 1, minWidth: 0, truncate: true, color: info.selected ? colors.text : colors.dim }}>{node.name}</Text>
          {#if !node.folder && node.size !== undefined}
            <Text style={{ ...mono, fontSize: 11, color: colors.faint }}>{node.size < 1024 ? `${node.size} B` : `${(node.size / 1024).toFixed(1)} KB`}</Text>
          {/if}
        </View>
      {/snippet}
    </Tree>
    <Text style={caption}>{treeSelected === undefined ? "Nothing selected" : `Selected ${treeSelected}`}{activated ? `. ${activated}.` : ""}</Text>
  </Card>
</View>
