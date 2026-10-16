<!--
  A tree of rows that expand and collapse: a file browser, an outline, a nested list of anything.
  Only the rows of expanded branches exist, and of those only the ones near the window are built,
  because it is a `List` underneath. A tree with a million nodes costs what the ones in view do,
  when `childrenOf` makes a branch's children only when it is asked.

    <Tree items={roots} key={(node) => node.path} childrenOf={(node) => node.children} bind:selected onactivate={open}>
      {#snippet row(node, info)}<Text>{node.name}</Text>{/snippet}
    </Tree>

  Click a row to select it, the arrow to expand it, a row twice to activate it. With the tree focused, Down
  and Up move the selection, Right expands a branch (and then goes into it), Left collapses it (and
  then goes to its parent), Home and End jump, and Enter activates the row, which also toggles a branch.
-->
<script lang="ts" generics="T">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";
  import List from "./List.svelte";
  import View from "./View.svelte";
  import { caretDown, caretRight } from "./icons";
  import type { TreeRow } from "./tree";
  import type { KeyEvent, PointerEvent } from "./events";
  import type { Style } from "./style";

  interface Props {
    /** The top level. */
    items: T[];
    /** Identifies a node, for what is expanded and selected. It must be unique and stay the same for the same node. */
    key: (node: T) => string | number;
    /** A node's children. Return nothing for a leaf. It is called for expanded branches only. */
    childrenOf?: (node: T) => T[] | undefined | null;
    /** Whether a node can be expanded. The default is that it has children, which a lazily loaded branch does not yet. */
    isBranch?: (node: T) => boolean;
    row: Snippet<[T, TreeRow]>;
    /** The keys of the nodes that are expanded. Replace it to change it; it is never changed in place. */
    expanded?: Set<string | number>;
    /** The key of the selected node. */
    selected?: string | number | undefined;
    /** The height of every row, in pixels, or `"auto"` to measure rows that differ. The default is 28. */
    rowHeight?: number | "auto";
    /** The indent of each level, in pixels. The default is 16. */
    indent?: number;
    overscan?: number;
    /** A node was selected, by the pointer or the keys. */
    onselect?: (node: T) => void;
    /** Enter, or a second press, on a node. */
    onactivate?: (node: T) => void;
    /** A branch was expanded: load its children here. */
    onexpand?: (node: T) => void;
    oncollapse?: (node: T) => void;
    /** The tree's style. Give it a bounded height. */
    style?: Style;
    /** Laid over every row's style. */
    rowStyle?: Style;
    /** Laid over a selected row's style. */
    selectedStyle?: Style;
  }

  let {
    items,
    key,
    childrenOf = () => undefined,
    isBranch,
    row: renderRow,
    expanded = $bindable(new Set()),
    selected = $bindable(),
    rowHeight = 28,
    indent = 16,
    overscan,
    onselect,
    onactivate,
    onexpand,
    oncollapse,
    style,
    rowStyle,
    selectedStyle,
  }: Props = $props();

  interface Entry {
    node: T;
    key: string | number;
    depth: number;
    branch: boolean;
    /** The index of its parent in `flat`, or -1. */
    parent: number;
  }

  const branchOf = (node: T): boolean => (isBranch ? isBranch(node) : (childrenOf(node)?.length ?? 0) > 0);

  /** Every row now showing, in order: the tree with its collapsed branches left out. */
  const flat = $derived.by(() => {
    const out: Entry[] = [];
    const walk = (nodes: T[], depth: number, parent: number): void => {
      for (const node of nodes) {
        const id = key(node);
        const branch = branchOf(node);
        const index = out.length;
        out.push({ node, key: id, depth, branch, parent });
        if (branch && expanded.has(id)) {
          walk(childrenOf(node) ?? [], depth + 1, index);
        }
      }
    };
    walk(items, 0, -1);
    return out;
  });

  const current = $derived(selected === undefined ? -1 : flat.findIndex((entry) => entry.key === selected));

  let list: ReturnType<typeof List> | undefined = $state();
  let focused = $state(false);

  function expand(entry: Entry, open: boolean): void {
    if (expanded.has(entry.key) === open) {
      return;
    }
    const next = new Set(expanded);
    if (open) {
      next.add(entry.key);
    } else {
      next.delete(entry.key);
    }
    expanded = next;
    (open ? onexpand : oncollapse)?.(entry.node);
  }

  function select(index: number): void {
    const entry = flat[index];
    if (entry === undefined) {
      return;
    }
    const changed = selected !== entry.key;
    selected = entry.key;
    list?.scrollToIndex(index, { align: "nearest" });
    if (changed) {
      onselect?.(entry.node);
    }
  }

  function activate(index: number): void {
    const entry = flat[index];
    if (entry === undefined) {
      return;
    }
    if (entry.branch) {
      expand(entry, !expanded.has(entry.key));
    }
    onactivate?.(entry.node);
  }

  function handleKey(event: KeyEvent): void {
    const entry = flat[current];
    switch (event.key) {
      case "ArrowDown":
        select(current < 0 ? 0 : Math.min(current + 1, flat.length - 1));
        break;
      case "ArrowUp":
        select(current < 0 ? 0 : Math.max(current - 1, 0));
        break;
      case "Home":
        select(0);
        break;
      case "End":
        select(flat.length - 1);
        break;
      case "ArrowRight":
        if (entry?.branch && !expanded.has(entry.key)) {
          expand(entry, true);
        } else if (entry?.branch && flat[current + 1]?.depth === entry.depth + 1) {
          select(current + 1);
        }
        break;
      case "ArrowLeft":
        if (entry?.branch && expanded.has(entry.key)) {
          expand(entry, false);
        } else if (entry !== undefined && entry.parent >= 0) {
          select(entry.parent);
        }
        break;
      case "Enter":
        if (current >= 0) {
          activate(current);
        }
        break;
      default:
        return;
    }
    event.stopPropagation();
  }

  /** Selects the row of a node that is showing, and scrolls to it. */
  export function reveal(id: string | number): void {
    const index = flat.findIndex((entry) => entry.key === id);
    if (index >= 0) {
      select(index);
    }
  }

  const rowLook = (chosen: boolean): Style => ({
    direction: "row",
    align: "center",
    gap: 4,
    paddingRight: 8,
    cursor: "default",
    background: chosen ? (focused ? "#8b5cf633" : "#ffffff14") : "transparent",
    ...(rowHeight === "auto" ? { minHeight: 28 } : { height: rowHeight }),
    ...rowStyle,
    ...(chosen ? selectedStyle : undefined),
  });
</script>

<View focusable style={{ minHeight: 0, ...style }} onkey={handleKey} onfocus={() => (focused = true)} onblur={() => (focused = false)}>
  <List
    bind:this={list}
    items={flat}
    rowHeight={rowHeight === "auto" ? undefined : rowHeight}
    estimatedRowHeight={28}
    key={(entry) => entry.key}
    {overscan}
    style={{ flex: 1 }}
  >
    {#snippet row(entry: Entry, index: number)}
      {@const chosen = entry.key === selected}
      {@const open = expanded.has(entry.key)}
      <View
        style={{ ...rowLook(chosen), paddingLeft: 4 + entry.depth * indent }}
        onpress={(event: PointerEvent) => {
          select(index);
          if (event.clicks === 2) {
            activate(index);
          }
        }}
      >
        <View
          style={{ size: 16, shrink: 0, align: "center", justify: "center", radius: 4 }}
          hover={entry.branch ? { background: "#ffffff14" } : undefined}
          onpress={entry.branch ? () => expand(entry, !open) : undefined}
        >
          {#if entry.branch}<Icon svg={open ? caretDown : caretRight} size={12} />{/if}
        </View>
        <View style={{ flex: 1, minWidth: 0 }}>
          {@render renderRow(entry.node, {
            depth: entry.depth,
            index,
            branch: entry.branch,
            expanded: open,
            selected: chosen,
            toggle: () => expand(entry, !open),
            select: () => select(index),
          })}
        </View>
      </View>
    {/snippet}
  </List>
</View>
