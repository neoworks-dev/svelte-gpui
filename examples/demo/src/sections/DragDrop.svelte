<!--
  Dragging. Files from other programs are dropped on a `View` with `ondragenter`, `ondragleave`
  and `ondrop`, and what is dropped can then be read, though the demo's file limit would
  refuse it otherwise. Things inside the app are moved with `Draggable` and `DropTarget`: a
  board of cards that reorder and move between columns.
-->
<script lang="ts">
  import { files, FileError } from "@neoworks/svelte-gpui";
  import { Draggable, DropTarget, Icon, Text, View, type DragInfo, type FileDragEvent } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import { caption, colors, mono } from "../theme";

  import tray from "@phosphor-icons/core/regular/tray-arrow-down.svg";
  import grip from "@phosphor-icons/core/regular/dots-six-vertical.svg";
  import file from "@phosphor-icons/core/regular/file.svg";
  import folder from "@phosphor-icons/core/regular/folder.svg";

  interface Dropped {
    path: string;
    kind: string;
    size: number;
    start: string;
  }

  let hovering = $state(0);
  let dropped = $state<Dropped[]>([]);
  let problem = $state("");

  const nameOf = (path: string): string => path.slice(path.lastIndexOf("/") + 1);

  async function inspect(path: string): Promise<Dropped> {
    const stat = await files.stat(path);
    let start = "";
    if (stat.kind === "file" && stat.size < 200_000) {
      try {
        start = (await files.readText(path)).slice(0, 120).replace(/\s+/g, " ");
      } catch (error) {
        start = error instanceof FileError && error.code === "InvalidData" ? "(not text)" : "";
      }
    }
    return { path, kind: stat.kind, size: stat.size, start };
  }

  async function drop(event: FileDragEvent): Promise<void> {
    hovering = 0;
    problem = "";
    try {
      dropped = await Promise.all(event.paths.map(inspect));
    } catch (error) {
      problem = (error as Error).message;
    }
  }

  // A board: three columns of cards, which can be reordered and moved between columns.
  interface Task {
    id: number;
    title: string;
  }
  type Column = "todo" | "doing" | "done";

  let board = $state<Record<Column, Task[]>>({
    todo: [
      { id: 1, title: "Anchor popovers to a box" },
      { id: 2, title: "Flip near the window edge" },
      { id: 3, title: "Move focus into menus" },
    ],
    doing: [{ id: 4, title: "Drag and drop between lists" }],
    done: [
      { id: 5, title: "Tooltips with a delay" },
      { id: 6, title: "Context menus at the pointer" },
    ],
  });
  const columns: { id: Column; title: string }[] = [
    { id: "todo", title: "To do" },
    { id: "doing", title: "Doing" },
    { id: "done", title: "Done" },
  ];

  // Where a card would go if it were dropped now.
  let marker = $state<{ column: Column; index: number } | null>(null);

  const pointAt = (column: Column, index: number, info: DragInfo<Task>): void => {
    marker = { column, index: info.y < info.height / 2 ? index : index + 1 };
  };

  function move(task: Task): void {
    const target = marker;
    marker = null;
    if (target === null) {
      return;
    }
    const from = (Object.keys(board) as Column[]).find((column) => board[column].some((card) => card.id === task.id));
    if (from === undefined) {
      return;
    }
    const at = board[from].findIndex((card) => card.id === task.id);
    const [card] = board[from].splice(at, 1);
    const index = from === target.column && at < target.index ? target.index - 1 : target.index;
    board[target.column].splice(index, 0, card);
  }

  const size = (bytes: number): string => (bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`);
</script>

<View style={{ gap: 16 }}>
  <Card title="Files from other programs" note="Drag files or folders from a file manager onto the box. The paths are readable once dropped, though the demo only allows its own data directory.">
    <View
      style={{
        height: 120,
        align: "center",
        justify: "center",
        gap: 6,
        radius: 10,
        borderWidth: 2,
        borderColor: hovering > 0 ? colors.accent : colors.strongBorder,
        background: hovering > 0 ? colors.accentSoft : colors.window,
        transition: 120,
      }}
      ondragenter={(event) => (hovering = event.count)}
      ondragleave={() => (hovering = 0)}
      ondrop={drop}
    >
      <Icon svg={tray} size={28} color={hovering > 0 ? colors.accentText : colors.faint} />
      <Text style={{ color: hovering > 0 ? colors.accentText : colors.dim }}>
        {hovering > 0 ? `Release to drop ${hovering} ${hovering === 1 ? "item" : "items"}` : "Drop files here"}
      </Text>
    </View>
    {#if problem}<Text style={{ color: colors.red }}>{problem}</Text>{/if}
    {#each dropped as entry (entry.path)}
      <View style={{ direction: "row", gap: 10, align: "center", paddingY: 4 }}>
        <Icon svg={entry.kind === "directory" ? folder : file} size={16} color={colors.dim} />
        <View style={{ flex: 1, minWidth: 0 }}>
          <Text style={{ truncate: true }}>{nameOf(entry.path)}</Text>
          <Text style={{ ...caption, truncate: true }}>{entry.path}</Text>
          {#if entry.start}<Text style={{ ...mono, color: colors.dim, truncate: true }}>{entry.start}</Text>{/if}
        </View>
        <Text style={{ ...mono, color: colors.faint }}>{entry.kind === "directory" ? "folder" : size(entry.size)}</Text>
      </View>
    {/each}
  </Card>

  <Card title="Moving things around" note="Press a card and drag. A copy follows the pointer, and a line shows where it would land. Release over a column to drop it; Escape cancels.">
    <View style={{ direction: "row", gap: 12 }}>
      {#each columns as column (column.id)}
        <DropTarget
          ondragover={(info: DragInfo<Task>) => (marker = { column: column.id, index: board[column.id].length })}
          ondragleave={() => (marker = marker?.column === column.id ? null : marker)}
          ondrop={(info: DragInfo<Task>) => move(info.data)}
          style={{ flex: 1, minWidth: 0, minHeight: 200, padding: 8, gap: 0, radius: 8, background: colors.window, borderWidth: 1, borderColor: colors.border }}
          over={{ borderColor: colors.accent }}
        >
          <Text style={{ ...caption, paddingX: 4, paddingBottom: 8, fontWeight: "semibold" }}>{column.title}  {board[column.id].length}</Text>
          {#each board[column.id] as task, index (task.id)}
            {#if marker?.column === column.id && marker.index === index}
              <View style={{ height: 3, radius: 2, background: colors.accent, marginY: 2 }} />
            {/if}
            <DropTarget ondragover={(info: DragInfo<Task>) => pointAt(column.id, index, info)} ondrop={(info: DragInfo<Task>) => move(info.data)} style={{ paddingY: 3 }}>
              <Draggable
                data={task}
                onend={() => (marker = null)}
                style={{ direction: "row", align: "center", gap: 8, padding: 10, radius: 6, background: colors.raised, borderWidth: 1, borderColor: colors.strongBorder, cursor: "grab" }}
                hover={{ background: colors.hover }}
              >
                <Icon svg={grip} size={14} color={colors.faint} />
                <Text style={{ flex: 1, minWidth: 0 }}>{task.title}</Text>
              </Draggable>
            </DropTarget>
          {/each}
          {#if marker?.column === column.id && marker.index === board[column.id].length}
            <View style={{ height: 3, radius: 2, background: colors.accent, marginY: 2 }} />
          {/if}
        </DropTarget>
      {/each}
    </View>
  </Card>
</View>
