<!--
  A dialog over a dimmed page. `basic` is a small centred card for one decision, `fullscreen` fills the
  window for a longer task such as creating an event. `actions` holds the buttons: at the bottom right of a
  basic dialog, in the top bar of a fullscreen one.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Scroll, Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import closeIcon from "@phosphor-icons/core/regular/x.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import IconButton from "./IconButton.svelte";
  import ModalLayer from "./ModalLayer.svelte";
  import { surfacePanelLook } from "./overlay-look";

  type Variant = "basic" | "fullscreen";

  interface Props {
    open?: boolean;
    /** The question or the name of the task. */
    title: string;
    /** SVG markup of a small icon above the title in a basic dialog, such as a warning. */
    icon?: string;
    variant?: Variant;
    /** Whether Escape and a press outside the card close the dialog. Turn off while a choice must be made. */
    dismissible?: boolean;
    /** `false` shows it in the page instead of over it. */
    modal?: boolean;
    onclose?: () => void;
    /** The supporting text or form. */
    children: Snippet;
    /** The buttons. */
    actions?: Snippet;
  }

  let { open = $bindable(false), title, icon, variant = "basic", dismissible = true, modal = true, onclose, children: supportingContent, actions }: Props = $props();

  const theme = $derived(currentTheme());
  const isFullscreen = $derived(variant === "fullscreen");
  const maximumBasicWidth = 448;

  function placement() {
    if (isFullscreen) {
      return "fill";
    }
    return "center";
  }

  function entrance() {
    if (isFullscreen) {
      return "rise";
    }
    return "fade";
  }

  function slotStyle(): Style {
    if (isFullscreen) {
      return { size: "full" };
    }
    return { width: "full", maxWidth: maximumBasicWidth, maxHeight: "full" };
  }

  function cardStyle(): Style {
    if (isFullscreen) {
      return { ...surfacePanelLook(theme), size: "full" };
    }
    return { ...surfacePanelLook(theme), radius: theme.radii.card, maxHeight: "full", width: "full" };
  }

  function closeInPage(): void {
    open = false;
    onclose?.();
  }

  function handleClose(): void {
    onclose?.();
  }
</script>

{#snippet card(requestClose: () => void)}
  <View style={cardStyle()}>
    {#if isFullscreen}
      <View style={{ direction: "row", align: "center", gap: 8, paddingX: 12, paddingY: 8, borderBottomWidth: 1, borderColor: theme.colors.line }}>
        <IconButton icon={closeIcon} label="Close" onpress={requestClose} />
        <Text style={{ ...typeStyle("title"), flex: 1, minWidth: 0, truncate: true }}>{title}</Text>
        {#if actions}
          <View style={{ direction: "row", align: "center", gap: 8, shrink: 0 }}>{@render actions()}</View>
        {/if}
      </View>
      <Scroll style={{ flex: 1, minHeight: 0 }}>
        <View style={{ ...typeStyle("body"), padding: 24 }}>{@render supportingContent()}</View>
      </Scroll>
    {:else}
      <Scroll style={{ shrink: 1, minHeight: 0 }}>
        <View style={{ padding: 24 }}>
          {#if icon}
            <Icon svg={icon} size={24} color={theme.colors.muted} style={{ marginBottom: 16 }} />
          {/if}
          <Text style={typeStyle("title")}>{title}</Text>
          <View style={{ ...typeStyle("body"), color: theme.colors.muted, marginTop: 12 }}>{@render supportingContent()}</View>
        </View>
      </Scroll>
      {#if actions}
        <View style={{ direction: "row", wrap: true, justify: "end", gap: 8, paddingX: 24, paddingTop: 8, paddingBottom: 24 }}>
          {@render actions()}
        </View>
      {/if}
    {/if}
  </View>
{/snippet}

{#if modal}
  <ModalLayer
    bind:open
    placement={placement()}
    entrance={entrance()}
    {dismissible}
    slotStyle={slotStyle()}
    onclose={handleClose}
  >
    {#snippet children(close)}
      {@render card(() => close("api"))}
    {/snippet}
  </ModalLayer>
{:else if open}
  {@render card(closeInPage)}
{/if}
