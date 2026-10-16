<!--
  A region with a menu that opens at the pointer when it is pressed with the right button. The
  menu flips and shifts to stay inside the window, closes on an outside press or Escape, and
  gives focus back when it closes.

    <ContextMenu items={[{ label: "Rename", onselect: rename }, { label: "Delete", danger: true, onselect: remove }]}>
      <Text>Right-press me</Text>
    </ContextMenu>

  `items` can be a function of the pressed position, for a menu that depends on what is under
  the pointer. `onopen` is called as it opens, with the position in the region.

  To open it from code, for an app that handles the right press itself, bind it and call `openAt`.
  Without children it draws nothing in the layout, so it can sit anywhere:

    <ContextMenu bind:this={menu} />
    <View ondown={(event) => event.button === 2 && menu.openAt(event.windowX, event.windowY, rowItems)} />
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import Menu from "./Menu.svelte";
  import Popover from "./Popover.svelte";
  import View from "./View.svelte";
  import type { MenuItem } from "./menu";
  import type { PointerEvent } from "./events";
  import type { Style } from "./style";

  interface Props {
    /** The menu's items. Optional when `openAt` is always given its own. */
    items?: MenuItem[] | ((event: PointerEvent) => MenuItem[]);
    onselect?: (item: MenuItem, index: number) => void;
    onopen?: (event: PointerEvent) => void;
    /** The region's style. */
    style?: Style;
    /** The menu's own style. */
    menuStyle?: Style;
    /** The popover's panel style. */
    panelStyle?: Style;
    children?: Snippet;
  }

  let { items = [], onselect, onopen, style, menuStyle, panelStyle, children: region }: Props = $props();

  let open = $state(false);
  let at: [number, number] = $state([0, 0]);
  let shown: MenuItem[] = $state([]);
  /** Counts the times it has opened, so that a menu opened again while open starts afresh. */
  let opened = $state(0);

  function show(x: number, y: number, list: MenuItem[]): void {
    shown = list;
    at = [x, y];
    opened++;
    open = true;
  }

  function press(event: PointerEvent): void {
    if (event.button !== 2) {
      return;
    }
    show(event.windowX, event.windowY, typeof items === "function" ? items(event) : items);
    onopen?.(event);
  }

  /**
   * Opens the menu at a point in the window, such as a pointer event's `windowX` and `windowY`, with
   * `items` or, without them, the `items` prop. It flips and shifts like the pointer's own menu, and
   * moves there if it is already open. `onopen` is for the menu's own right press and is not called.
   */
  export function openAt(x: number, y: number, list?: MenuItem[]): void {
    if (list === undefined && typeof items === "function") {
      // A function of the pointer, called with a pointer at this point.
      const event = { x, y, windowX: x, windowY: y, button: 2, clicks: 1, ctrl: false, shift: false, alt: false, meta: false };
      list = items(event);
    }
    show(x, y, list ?? (items as MenuItem[]));
  }

  /** Closes the menu, if it is open. */
  export function close(): void {
    open = false;
  }
</script>

{#snippet popover()}
  <Popover bind:open {at} side="bottom" align="start" offset={0} flip shift style={panelStyle}>
    {#snippet children({ close })}
      {#key opened}
        <Menu items={shown} {onselect} onclose={() => close("select")} style={menuStyle} />
      {/key}
    {/snippet}
  </Popover>
{/snippet}

{#if region}
  <View {style} ondown={press}>
    {@render region()}
    {@render popover()}
  </View>
{:else}
  {@render popover()}
{/if}
