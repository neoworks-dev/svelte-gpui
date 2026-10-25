<!--
  A panel attached to the left or right edge of the window, for details or a form that belongs to the page. A
  modal one dims the page and blocks it. A standard one takes its own column next to the page content.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Scroll, Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import closeIcon from "@phosphor-icons/core/regular/x.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import IconButton from "./IconButton.svelte";
  import ModalLayer from "./ModalLayer.svelte";
  import { surfacePanelLook } from "./overlay-look";

  type Side = "start" | "end";

  interface Props {
    open?: boolean;
    /** The name of the panel, such as "Event details". */
    title: string;
    /** Which edge of the window the sheet attaches to. */
    side?: Side;
    /** A modal sheet dims the page and blocks it. A standard sheet takes its own column next to the page content. */
    modal?: boolean;
    /** Whether Escape and a press outside close a modal sheet. */
    dismissible?: boolean;
    onclose?: () => void;
    children: Snippet;
    /** Buttons pinned to the bottom of the sheet. */
    actions?: Snippet;
  }

  let { open = $bindable(false), title, side = "end", modal = true, dismissible = true, onclose, children: sheetContent, actions }: Props = $props();

  const theme = $derived(currentTheme());
  const maximumModalWidth = 384;
  const standardWidth = 320;

  function modalRadius(): Style {
    const corner = theme.radii.media;
    if (side === "end") {
      return { radius: [corner, 0, 0, corner] };
    }
    return { radius: [0, corner, corner, 0] };
  }

  function standardEdge(): Style {
    if (side === "end") {
      return { borderLeftWidth: 1, borderColor: theme.colors.line };
    }
    return { borderRightWidth: 1, borderColor: theme.colors.line };
  }

  function entrance() {
    if (side === "end") {
      return "from-end";
    }
    return "from-start";
  }

  function sheetStyle(): Style {
    if (modal) {
      return { ...surfacePanelLook(theme), ...modalRadius(), size: "full" };
    }
    return { background: theme.colors.surface, color: theme.colors.text, size: "full", ...standardEdge() };
  }

  function closeInPage(): void {
    open = false;
    onclose?.();
  }
</script>

{#snippet sheet(requestClose: () => void)}
  <View style={sheetStyle()}>
    <View style={{ direction: "row", align: "center", gap: 8, paddingX: 24, paddingTop: 16, paddingBottom: 8 }}>
      <Text style={{ ...typeStyle("title"), flex: 1, minWidth: 0, truncate: true }}>{title}</Text>
      <IconButton icon={closeIcon} label={`Close ${title}`} onpress={requestClose} />
    </View>
    <Scroll style={{ flex: 1, minHeight: 0 }}>
      <View style={{ paddingX: 24, paddingY: 12 }}>{@render sheetContent()}</View>
    </Scroll>
    {#if actions}
      <View
        style={{
          direction: "row",
          wrap: true,
          justify: "end",
          gap: 8,
          paddingX: 24,
          paddingY: 16,
          borderTopWidth: 1,
          borderColor: theme.colors.line,
        }}
      >
        {@render actions()}
      </View>
    {/if}
  </View>
{/snippet}

{#if modal}
  <ModalLayer
    bind:open
    placement={side}
    entrance={entrance()}
    {dismissible}
    slotStyle={{ height: "full", width: "full", maxWidth: maximumModalWidth }}
    onclose={() => onclose?.()}
  >
    {#snippet children(close)}
      {@render sheet(() => close("api"))}
    {/snippet}
  </ModalLayer>
{:else if open}
  <View style={{ height: "full", width: standardWidth, maxWidth: "75%", minWidth: 0 }}>
    {@render sheet(closeInPage)}
  </View>
{/if}
