<!--
  The window-sized layer behind a dialog or a modal sheet: a scrim that dims the page and blocks it, and the
  panel placed against one of its edges. Escape and a press on the scrim close it when `dismissible`. Focus
  moves into the layer and returns to what had it. Internal to `Dialog`, `BottomSheet` and `SideSheet`.
-->
<script lang="ts">
  import type { ComponentProps, Snippet } from "svelte";
  import { quintOut } from "svelte/easing";
  import { fade, fly } from "svelte/transition";
  import { window } from "@neoworks/svelte-gpui";
  import { Popover, Presence, View, type CloseReason, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { scrimColor } from "./overlay-look";
  import type { ModalEntrance, ModalPlacement } from "./placement";

  interface Props {
    open?: boolean;
    placement?: ModalPlacement;
    entrance?: ModalEntrance;
    /** Whether Escape and a press on the scrim close the layer. */
    dismissible?: boolean;
    /** The size of the slot the panel sits in. The panel fills it. */
    slotStyle?: Style;
    onclose?: (reason: CloseReason) => void;
    /** The panel. `close` closes the layer and tells `onclose`. */
    children: Snippet<[(reason?: CloseReason) => void]>;
  }

  let { open = $bindable(false), placement = "center", entrance = "fade", dismissible = true, slotStyle, onclose, children: panel }: Props = $props();

  const theme = $derived(currentTheme());
  const layerLayer = 1;
  const scrimFadeMilliseconds = 220;
  const riseMilliseconds = 420;
  const slideMilliseconds = 380;

  function layerStyle(): Style {
    const style: Style = { width: window.size.width, height: window.size.height };
    if (placement === "center") {
      return { ...style, align: "center", justify: "center", padding: 16 };
    }
    if (placement === "bottom") {
      return { ...style, align: "center", justify: "end", paddingTop: 48 };
    }
    if (placement === "start") {
      return { ...style, direction: "row", align: "stretch", justify: "start" };
    }
    if (placement === "end") {
      return { ...style, direction: "row", align: "stretch", justify: "end" };
    }
    return style;
  }

  function slotTransition(): ComponentProps<typeof Presence>["in"] {
    if (entrance === "rise") {
      return [fly, { y: window.size.height, opacity: 1, duration: riseMilliseconds, easing: quintOut }];
    }
    if (entrance === "from-start") {
      return [fly, { x: -window.size.width, opacity: 1, duration: slideMilliseconds, easing: quintOut }];
    }
    if (entrance === "from-end") {
      return [fly, { x: window.size.width, opacity: 1, duration: slideMilliseconds, easing: quintOut }];
    }
    return [fade, { duration: scrimFadeMilliseconds }];
  }

  function pressScrim(close: (reason?: CloseReason) => void): void {
    if (dismissible) {
      close("outside");
    }
  }
</script>

<Popover
  bind:open
  at={[0, 0]}
  side="bottom"
  align="start"
  offset={0}
  flip={false}
  shift={false}
  margin={0}
  layer={layerLayer}
  closeOnOutside={false}
  closeOnEscape={dismissible}
  style={layerStyle()}
  {onclose}
>
  {#snippet children(api)}
    <Presence in={[fade, { duration: scrimFadeMilliseconds }]} style={{ position: "absolute", inset: 0 }}>
      <View style={{ size: "full", background: scrimColor(theme) }} onpress={() => pressScrim(api.close)} />
    </Presence>
    <Presence in={slotTransition()} style={slotStyle}>
      {@render panel(api.close)}
    </Presence>
  {/snippet}
</Popover>
