<!--
  A container for a group of content. `outlined` groups content on the page, `filled` sets a quiet area apart,
  `elevated` sits above it. With `onpress` the whole card is a button, so put no other buttons inside it.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { window } from "@neoworks/svelte-gpui";
  import { Text, View, type KeyEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import {
    cardLook,
    cardPadding,
    headlineLook,
    subheadLook,
    supportingTextLook,
    type CardPadding,
    type CardVariant,
  } from "./card-look";

  interface Props {
    variant?: CardVariant;
    padding?: CardPadding;
    /** Makes the whole card a button. */
    onpress?: () => void;
    /** Layout only, such as width or margins. */
    style?: Style;
    headline?: string;
    /** A short line under the headline, such as a date or a place. */
    subhead?: string;
    /** A sentence or two that explains the headline. */
    supportingText?: string;
    /** Full-bleed image or illustration at the top, outside the padding. */
    media?: Snippet;
    /** Buttons at the bottom right. Leave out `onpress` when using them. */
    actions?: Snippet;
    /** Anything that does not fit headline, subhead and supporting text, shown after them. */
    children?: Snippet;
  }

  let {
    variant = "outlined",
    padding = "md",
    onpress,
    style,
    headline,
    subhead,
    supportingText,
    media,
    actions,
    children,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const interactive = $derived(onpress !== undefined);
  const look = $derived(cardLook(theme, { variant, interactive }));
  const textOptions = $derived({ hasHeadline: headline !== undefined, hasSubhead: subhead !== undefined });

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

<View
  style={{ ...look.style, ...style }}
  hover={look.hover}
  active={look.active}
  focus={look.focus}
  focusable={interactive}
  {onpress}
  onkey={pressOnEnterOrSpace}
>
  {#if media}
    <View style={{ overflow: "hidden" }}>{@render media()}</View>
  {/if}
  <View style={{ flex: 1, padding: cardPadding(padding) }}>
    {#if headline}
      <Text style={headlineLook(theme, window.size.width)}>{headline}</Text>
    {/if}
    {#if subhead}
      <Text style={subheadLook(theme, textOptions)}>{subhead}</Text>
    {/if}
    {#if supportingText}
      <Text style={supportingTextLook(theme, textOptions)}>{supportingText}</Text>
    {/if}
    {@render children?.()}
    {#if actions}
      <View style={{ direction: "row", wrap: true, justify: "end", gap: 8, paddingTop: 24, marginTop: "auto" }}>
        {@render actions()}
      </View>
    {/if}
  </View>
</View>
