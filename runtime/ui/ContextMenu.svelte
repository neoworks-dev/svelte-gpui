<!--
  A region with a menu that opens at the pointer when it is pressed with the right button. The
  menu flips and shifts to stay inside the window, closes on an outside press or Escape, and
  gives focus back when it closes.

    <ContextMenu items={[{ label: "Rename", onselect: rename }, { label: "Delete", danger: true, onselect: remove }]}>
      <Text>Right-press me</Text>
    </ContextMenu>

  `items` can be a function of the pressed position, for a menu that depends on what is under
  the pointer. `onopen` is called as it opens, with the position in the region.
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
    items: MenuItem[] | ((event: PointerEvent) => MenuItem[]);
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

  let { items, onselect, onopen, style, menuStyle, panelStyle, children: region }: Props = $props();

  let open = $state(false);
  let at: [number, number] = $state([0, 0]);
  let shown: MenuItem[] = $state([]);

  function press(event: PointerEvent): void {
    if (event.button !== 2) {
      return;
    }
    shown = typeof items === "function" ? items(event) : items;
    at = [event.windowX, event.windowY];
    open = true;
    onopen?.(event);
  }
</script>

<View {style} ondown={press}>
  {@render region?.()}
  <Popover bind:open {at} side="bottom" align="start" offset={0} flip shift style={panelStyle}>
    {#snippet children({ close })}
      <Menu items={shown} {onselect} onclose={() => close("select")} style={menuStyle} />
    {/snippet}
  </Popover>
</View>
