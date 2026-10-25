<!--
  The bar at the top of a screen: a menu or back button at the start, the title, and up to three actions at the end.
  `medium` and `large` show a bigger title under the buttons until the page scrolls; set `scrolled` once content has
  moved under the bar and it turns to glass.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import {
    hasHeadlineBlock,
    showsRowTitle,
    topAppBarHeadlineLook,
    topAppBarHeadlineTextLook,
    topAppBarLook,
    topAppBarRowLook,
    topAppBarSlotLook,
    topAppBarTitleLook,
    topAppBarTitleSlotLook,
    type TopAppBarVariant,
  } from "./top-app-bar-look";

  interface Props {
    /** The name of the screen. */
    title: string;
    /** `small` is a single row. `centered` centres the title. `medium` and `large` show a bigger title under the buttons until the page scrolls. */
    variant?: TopAppBarVariant;
    /** Pass true once the page content has scrolled under the bar, so it turns into glass and the big title folds into the row. */
    scrolled?: boolean;
    /** Navigation at the start: a menu or back IconButton. */
    leading?: Snippet;
    /** Up to three actions at the end: IconButtons, or one overflow Menu. */
    trailing?: Snippet;
    /** Layout only, such as pinning it over a scrolling page. */
    style?: Style;
  }

  let { title, variant = "small", scrolled = false, leading, trailing, style }: Props = $props();

  const theme = $derived(currentTheme());
</script>

<View style={{ ...topAppBarLook(theme, scrolled), ...style }}>
  <View style={topAppBarRowLook}>
    {#if leading}
      <View style={topAppBarSlotLook}>{@render leading()}</View>
    {/if}
    <View style={topAppBarTitleSlotLook()}>
      {#if showsRowTitle(variant, scrolled)}
        <Text style={topAppBarTitleLook(variant)}>{title}</Text>
      {/if}
    </View>
    {#if trailing}
      <View style={topAppBarSlotLook}>{@render trailing()}</View>
    {/if}
  </View>
  {#if hasHeadlineBlock(variant)}
    <View style={topAppBarHeadlineLook(scrolled)}>
      <Text style={topAppBarHeadlineTextLook(variant)}>{title}</Text>
    </View>
  {/if}
</View>
