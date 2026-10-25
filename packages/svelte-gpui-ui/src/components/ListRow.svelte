<!--
  A single outlined row, for places where a list would be too much, such as a link card in a settings page.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Text, View, type KeyEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { caretRight } from "../glyphs";
  import { currentTheme } from "../theme.svelte";
  import {
    listRowChevronSize,
    listRowFrameLook,
    listRowLook,
    listRowSubtitleLook,
    listRowTitleLook,
  } from "./list-row-look";

  interface Props {
    /** The main line. */
    title: string;
    /** A second line with detail. */
    subtitle?: string;
    /** Makes the whole row a button. */
    onpress?: () => void;
    /** A caret that says the row opens something. */
    chevron?: boolean;
    leading?: Snippet;
    trailing?: Snippet;
    /** Layout only, such as margins. */
    style?: Style;
  }

  let { title, subtitle, onpress, chevron = false, leading, trailing, style }: Props = $props();

  const theme = $derived(currentTheme());
  const interactive = $derived(onpress !== undefined);
  const look = $derived(listRowLook(theme, interactive));
  let focused = $state(false);

  function pressOnEnterOrSpace(event: KeyEvent): void {
    if (!interactive) {
      return;
    }
    if (event.key !== "Enter" && event.key !== " ") {
      return;
    }
    event.preventDefault();
    onpress?.();
  }
</script>

<View style={{ ...listRowFrameLook(theme, focused), ...style }}>
  <View
    style={look.style}
    hover={look.hover}
    active={look.active}
    focusable={interactive}
    {onpress}
    onkey={pressOnEnterOrSpace}
    onfocus={() => (focused = true)}
    onblur={() => (focused = false)}
  >
    {#if leading}
      <View style={{ direction: "row", align: "center", shrink: 0 }}>{@render leading()}</View>
    {/if}
    <View style={{ flex: 1, minWidth: 0 }}>
      <Text style={listRowTitleLook()}>{title}</Text>
      {#if subtitle}
        <Text style={listRowSubtitleLook(theme)}>{subtitle}</Text>
      {/if}
    </View>
    {#if trailing}
      <View style={{ direction: "row", align: "center", shrink: 0 }}>{@render trailing()}</View>
    {/if}
    {#if chevron}
      <Icon svg={caretRight} size={listRowChevronSize} color={theme.colors.faint} />
    {/if}
  </View>
</View>
