<!--
  One row of a List: an icon or other leading content, a headline with optional overline and supporting text,
  and trailing text, content or a caret. A row with `onpress` is a button.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Text, View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import { caretRight } from "../glyphs";
  import { currentTheme } from "../theme.svelte";
  import { createFocusSource } from "./focusSource";
  import { isActivationKey } from "./navigation";
  import {
    listItemChevronSize,
    listItemFocusRingLook,
    listItemHeadlineLook,
    listItemIconCircleLook,
    listItemIconSize,
    listItemLook,
    listItemOverlineLook,
    listItemSupportingTextLook,
    listItemTrailingTextLook,
    type ListLines,
  } from "./list-look";
  import { useList } from "./listContext";

  interface Props {
    /** The main line, such as a name or a title. */
    headline: string;
    /** A small line above the headline, such as a category or a date. */
    overline?: string;
    /** Detail under the headline. */
    supportingText?: string;
    /** How many lines the supporting text may use before it is cut off with an ellipsis. */
    lines?: ListLines;
    /** SVG markup of a leading icon in a soft circle. For avatars and images use `leading` instead. */
    icon?: string;
    /** Custom leading content such as an avatar, a thumbnail or a checkbox. */
    leading?: Snippet;
    /** Custom trailing content such as a badge, a switch or a button. Do not combine with `onpress`. */
    trailing?: Snippet;
    /** Short text at the end, such as a size or a time. */
    trailingText?: string;
    /** A caret that says the row opens something. */
    chevron?: boolean;
    /** Makes the whole row a button. */
    onpress?: () => void;
    /** Marks the current row, for example the open conversation. */
    selected?: boolean;
    disabled?: boolean;
  }

  let {
    headline,
    overline,
    supportingText,
    lines = 2,
    icon,
    leading,
    trailing,
    trailingText,
    chevron = false,
    onpress,
    selected = false,
    disabled = false,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const list = useList();
  const interactive = $derived(onpress !== undefined && !disabled);
  const look = $derived(
    listItemLook(theme, { interactive, selected, disabled, dividers: list !== undefined && list.dividers }),
  );
  let focused = $state(false);
  const focusSource = createFocusSource();

  function pressOnActivationKey(event: KeyEvent): void {
    if (!isActivationKey(event.key)) {
      return;
    }
    event.preventDefault();
    onpress?.();
  }

  function handleFocus(): void {
    focused = focusSource.focused();
  }

  function handleBlur(): void {
    focused = false;
    focusSource.blurred();
  }

  /** A row that cannot be pressed listens for nothing, so it takes no handler from the host. */
  function whenInteractive<Handler>(handler: Handler): Handler | undefined {
    if (interactive) {
      return handler;
    }
    return undefined;
  }
</script>

<View
  style={look.style}
  hover={look.hover}
  active={look.active}
  focusable={interactive}
  onpress={whenInteractive(onpress)}
  ondown={whenInteractive(focusSource.pointerDown)}
  onkey={whenInteractive(pressOnActivationKey)}
  onfocus={whenInteractive(handleFocus)}
  onblur={whenInteractive(handleBlur)}
>
  {#if leading}
    <View style={{ direction: "row", align: "center", shrink: 0 }}>{@render leading()}</View>
  {:else if icon}
    <View style={listItemIconCircleLook(theme)}>
      <Icon svg={icon} size={listItemIconSize} />
    </View>
  {/if}
  <View style={{ flex: 1, minWidth: 0 }}>
    {#if overline}
      <Text style={listItemOverlineLook(theme)}>{overline}</Text>
    {/if}
    <Text style={listItemHeadlineLook()}>{headline}</Text>
    {#if supportingText}
      <Text style={listItemSupportingTextLook(theme, lines)}>{supportingText}</Text>
    {/if}
  </View>
  {#if trailingText}
    <Text style={listItemTrailingTextLook(theme)}>{trailingText}</Text>
  {/if}
  {#if trailing}
    <View style={{ direction: "row", align: "center", shrink: 0 }}>{@render trailing()}</View>
  {/if}
  {#if chevron}
    <Icon svg={caretRight} size={listItemChevronSize} color={theme.colors.faint} style={{ shrink: 0 }} />
  {/if}
  {#if focused}
    <View style={listItemFocusRingLook(theme)} />
  {/if}
</View>
