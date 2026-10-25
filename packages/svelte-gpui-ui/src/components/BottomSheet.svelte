<!--
  A sheet that rises from the bottom of the window. A modal one dims the page and blocks it. A standard one sits
  at the bottom of its parent and leaves the page usable. Dragging the handle down, or pressing it, closes it.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Scroll, Text, View, type PointerEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import ModalLayer from "./ModalLayer.svelte";
  import { surfacePanelLook } from "./overlay-look";
  import { dragOffset, dragOutcome } from "./sheetDrag";

  interface Props {
    open?: boolean;
    /** Names the sheet. Shown at the top unless `hideTitle` is on. */
    title: string;
    /** Leaves the title out, for sheets whose content already explains itself. */
    hideTitle?: boolean;
    /** A modal sheet dims the page and blocks it. A standard sheet sits at the bottom of its positioned parent. */
    modal?: boolean;
    /** Shows the grab bar. */
    dragHandle?: boolean;
    /** Whether Escape and a press outside close a modal sheet. */
    dismissible?: boolean;
    onclose?: () => void;
    children: Snippet;
  }

  let { open = $bindable(false), title, hideTitle = false, modal = true, dragHandle = true, dismissible = true, onclose, children: sheetContent }: Props = $props();

  const theme = $derived(currentTheme());
  const maximumWidth = 576;

  let dragStartY = 0;
  let largestOffset = 0;
  let dragging = $state(false);
  let currentOffset = $state(0);

  function startDrag(event: PointerEvent): void {
    dragStartY = event.windowY;
    largestOffset = 0;
    currentOffset = 0;
    dragging = true;
  }

  function moveDrag(event: PointerEvent): void {
    if (!dragging) {
      return;
    }
    currentOffset = dragOffset(dragStartY, event.windowY);
    largestOffset = Math.max(largestOffset, currentOffset);
  }

  function endDrag(requestClose: () => void): void {
    if (!dragging) {
      return;
    }
    dragging = false;
    const outcome = dragOutcome(largestOffset, currentOffset);
    currentOffset = 0;
    if (outcome !== "settle") {
      requestClose();
    }
  }

  function sheetStyle(): Style {
    return {
      ...surfacePanelLook(theme),
      radius: [theme.radii.media, theme.radii.media, 0, 0],
      width: "full",
      maxWidth: maximumWidth,
      maxHeight: "full",
      translate: [0, currentOffset],
    };
  }

  function titlePaddingTop(): number {
    if (dragHandle) {
      return 0;
    }
    return 12;
  }

  function closeInPage(): void {
    open = false;
    onclose?.();
  }
</script>

{#snippet sheet(requestClose: () => void)}
  <View style={sheetStyle()}>
    {#if dragHandle}
      <View
        style={{ align: "center", paddingY: 12, cursor: "grab", shrink: 0 }}
        ondown={startDrag}
        onmove={moveDrag}
        onup={() => endDrag(requestClose)}
      >
        <View style={{ width: 32, height: 4, radius: 2, background: theme.colors.lineStrong }} />
      </View>
    {/if}
    {#if !hideTitle}
      <Text style={{ ...typeStyle("title"), paddingX: 24, paddingTop: titlePaddingTop() }}>{title}</Text>
    {/if}
    <Scroll style={{ shrink: 1, minHeight: 0 }}>
      <View style={{ paddingX: 24, paddingTop: 12, paddingBottom: 24 }}>{@render sheetContent()}</View>
    </Scroll>
  </View>
{/snippet}

{#if modal}
  <ModalLayer
    bind:open
    placement="bottom"
    entrance="rise"
    {dismissible}
    slotStyle={{ width: "full", maxWidth: maximumWidth, maxHeight: "full" }}
    onclose={() => onclose?.()}
  >
    {#snippet children(close)}
      {@render sheet(() => close("api"))}
    {/snippet}
  </ModalLayer>
{:else if open}
  <View style={{ position: "absolute", left: 0, right: 0, bottom: 0, align: "center" }}>
    {@render sheet(closeInPage)}
  </View>
{/if}
