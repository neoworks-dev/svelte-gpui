<!--
  A small label that appears next to its content after the pointer has rested on it. Give `title`,
  `content` or `actions` to make it a rich tooltip: a card that stays open while the pointer is on it.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Overlay, Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import { visibleShadows } from "./look";
  import { parsePlacement, type FloatingPlacement } from "./placement";

  interface Props {
    /** The tooltip message. Keep it to a few words; the trigger already has its own label. */
    text?: string;
    /** Makes the tooltip a rich one with a heading above the text. */
    title?: string;
    /** Preferred side. It moves to the opposite side when there is no room. */
    placement?: FloatingPlacement;
    /** Milliseconds the pointer rests before it shows. */
    delay?: number;
    disabled?: boolean;
    /** Rich tooltip body that replaces `text`. */
    content?: Snippet;
    /** Buttons at the bottom of a rich tooltip. */
    actions?: Snippet;
    /** The wrapper around the trigger, which is what the pointer rests on. Layout only. */
    anchorStyle?: Style;
    /** The trigger the tooltip is attached to. */
    children: Snippet;
  }

  let { text = "", title, placement = "top", delay = 300, disabled = false, content, actions, anchorStyle, children }: Props = $props();

  const hideDelayMilliseconds = 120;
  const tooltipLayer = 3;
  const richWidth = 320;

  const theme = $derived(currentTheme());
  const side = $derived(parsePlacement(placement).side);
  const align = $derived(parsePlacement(placement).align);
  const isRich = $derived(title !== undefined || content !== undefined || actions !== undefined);
  const hasMessage = $derived(text !== "" || title !== undefined || content !== undefined);

  let open = $state(false);
  let timer: number | undefined;

  function show(): void {
    if (disabled || !hasMessage) {
      return;
    }
    clearTimeout(timer);
    timer = setTimeout(() => (open = true), delay) as unknown as number;
  }

  function hide(): void {
    clearTimeout(timer);
    timer = setTimeout(() => (open = false), hideDelayMilliseconds) as unknown as number;
  }

  function keepOpen(): void {
    clearTimeout(timer);
  }

  function handleAnchorHover(hovered: boolean): void {
    if (hovered) {
      show();
      return;
    }
    hide();
  }

  function handleTooltipHover(hovered: boolean): void {
    if (hovered) {
      keepOpen();
      return;
    }
    hide();
  }

  function closeOnEscape(event: KeyboardEvent): void {
    if (open && event.key === "Escape") {
      clearTimeout(timer);
      open = false;
    }
  }

  function surfaceLook(): Style {
    if (isRich) {
      return {
        background: theme.colors.surface,
        color: theme.colors.text,
        radius: theme.radii.card,
        padding: 16,
        width: richWidth,
        shadow: visibleShadows(theme.shadows.overlay),
      };
    }
    return {
      ...typeStyle("caption"),
      background: theme.colors.fill,
      color: theme.colors.onFill,
      radius: theme.radii.control,
      paddingX: 12,
      paddingY: 6,
      maxWidth: 320,
    };
  }

  $effect(() => () => clearTimeout(timer));
</script>

<svelte:window onkeydown={closeOnEscape} />

<View style={{ alignSelf: "start", ...anchorStyle }} onhover={handleAnchorHover}>
  {@render children()}
  <Overlay {side} {align} offset={8} layer={tooltipLayer} passthrough={!isRich} onhover={handleTooltipHover} style={surfaceLook()}>
    {#if open}
      {#if isRich}
        {#if title}
          <Text style={{ ...typeStyle("body"), fontWeight: 500 }}>{title}</Text>
        {/if}
        {#if content}
          <View style={{ ...typeStyle("bodySmall"), color: theme.colors.muted }}>{@render content()}</View>
        {:else if text}
          <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.muted }}>{text}</Text>
        {/if}
        {#if actions}
          <View style={{ direction: "row", gap: 8, marginTop: 12 }}>{@render actions()}</View>
        {/if}
      {:else}
        <Text>{text}</Text>
      {/if}
    {/if}
  </Overlay>
</View>
