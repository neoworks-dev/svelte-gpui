<!--
  A search bar. While it has focus, `suggestions` such as recent searches or live results hang under
  it in a frosted panel. Arrow keys move through them, Enter chooses the highlighted one or submits the
  text, and Escape clears the text.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Input, Overlay, Scroll, Text, View, type KeyEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import magnifyingGlass from "@phosphor-icons/core/regular/magnifying-glass.svg";
  import closeIcon from "@phosphor-icons/core/regular/x.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import type { SearchSuggestion } from "./types";
  import { withAlpha } from "./colorMix";
  import { nextFocusIndex } from "./rovingFocus";
  import IconButton from "./IconButton.svelte";

  interface Props {
    value?: string;
    placeholder?: string;
    suggestions?: SearchSuggestion[];
    /** A small title above the suggestions, such as "Recent searches". */
    heading?: string;
    /** Called with the text when Enter is pressed without a highlighted suggestion. */
    onsubmit?: (value: string) => void;
    /** Called when a suggestion is chosen. The field is filled with its value first. */
    onselect?: (suggestion: SearchSuggestion) => void;
    /** Replaces the magnifier at the start, for example with a menu button. */
    leading?: Snippet;
    /** Content at the end, such as an avatar or a voice input button. */
    trailing?: Snippet;
    /** `sm` fits a header row next to icon buttons. */
    size?: "md" | "sm";
    /** Layout of the whole bar, such as its width. */
    style?: Style;
  }

  let {
    value = $bindable(""),
    placeholder = "Search",
    suggestions = [],
    heading,
    onsubmit,
    onselect,
    leading,
    trailing,
    size = "md",
    style,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const maximumListHeight = 320;

  let input: ReturnType<typeof Input> | undefined = $state();
  let focused = $state(false);
  let listOpen = $state(false);
  let activeIndex = $state(-1);

  const reservesIconSlot = $derived(suggestions.some((suggestion) => suggestion.icon !== undefined));
  const showList = $derived(listOpen && focused && suggestions.length > 0);

  function iconSize(): number {
    if (size === "sm") {
      return 16;
    }
    return 20;
  }

  function barHeight(): number {
    if (size === "sm") {
      return 36;
    }
    return 48;
  }

  function inputType(): Style {
    if (size === "sm") {
      return typeStyle("bodySmall");
    }
    return typeStyle("body");
  }

  function barBorder(): string {
    if (focused) {
      return theme.colors.lineStrong;
    }
    return "transparent";
  }

  function barHover(): Style {
    if (focused) {
      return {};
    }
    return { background: theme.colors.hover };
  }

  function textOf(suggestion: SearchSuggestion): string {
    if (suggestion.label) {
      return suggestion.label;
    }
    return suggestion.value;
  }

  function rowBackground(index: number): string {
    if (index === activeIndex) {
      return theme.colors.hover;
    }
    return "transparent";
  }

  function glassPanel(): Style {
    return {
      background: withAlpha(theme.colors.canvas, 0.96),
      radius: theme.radii.card,
      shadow: theme.shadows.overlay,
      overflow: "hidden",
    };
  }

  function choose(suggestion: SearchSuggestion): void {
    value = suggestion.value;
    listOpen = false;
    activeIndex = -1;
    onselect?.(suggestion);
    input?.focus();
  }

  function clear(): void {
    value = "";
    input?.focus();
  }

  function highlightOnHover(index: number, isHovered: boolean): void {
    if (isHovered) {
      activeIndex = index;
    }
  }

  function handleInput(): void {
    listOpen = true;
    activeIndex = -1;
  }

  function moveActive(key: string): void {
    const disabledStates = suggestions.map(() => false);
    let from = activeIndex;
    if (activeIndex < 0 && key === "ArrowUp") {
      from = suggestions.length;
    }
    const target = nextFocusIndex(key, from, disabledStates, "vertical");
    if (target !== undefined) {
      activeIndex = target;
    }
  }

  function submit(): void {
    if (showList && activeIndex >= 0) {
      choose(suggestions[activeIndex]);
      return;
    }
    listOpen = false;
    onsubmit?.(value);
  }

  function handleKey(event: KeyEvent): void {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      listOpen = true;
      moveActive(event.key);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      submit();
      return;
    }
    if (event.key === "Escape" && value !== "") {
      clear();
    }
  }
</script>

<View style={{ width: "full", ...style }}>
  <View
    style={{
      direction: "row",
      align: "center",
      gap: 4,
      width: "full",
      height: barHeight(),
      paddingLeft: 16,
      paddingRight: 8,
      radius: 999,
      borderWidth: 1,
      borderColor: barBorder(),
      background: theme.colors.sunken,
      transition: 200,
    }}
    hover={barHover()}
    onpress={() => input?.focus()}
  >
    {#if leading}
      {@render leading()}
    {:else}
      <Icon svg={magnifyingGlass} size={iconSize()} color={theme.colors.muted} style={{ shrink: 0 }} />
    {/if}
    <Input
      bind:this={input}
      bind:value
      {placeholder}
      style={{
        ...inputType(),
        flex: 1,
        minWidth: 0,
        alignSelf: "center",
        paddingX: 8,
        background: "transparent",
        color: theme.colors.text,
      }}
      oninput={handleInput}
      onkey={handleKey}
      onfocus={() => {
        focused = true;
        listOpen = true;
      }}
      onblur={() => (focused = false)}
    />
    {#if value !== ""}
      <IconButton icon={closeIcon} label="Clear" size="sm" onpress={clear} style={{ shrink: 0 }} />
    {/if}
    {#if trailing}
      {@render trailing()}
    {/if}
  </View>

  {#if showList}
    <Overlay side="bottom" align="start" offset={8} matchWidth keepFocus style={glassPanel()}>
      {#if heading}
        <Text style={{ ...typeStyle("label"), uppercase: true, paddingX: 20, paddingTop: 12, paddingBottom: 4, color: theme.colors.muted }}>
          {heading}
        </Text>
      {/if}
      <Scroll style={{ maxHeight: maximumListHeight, padding: 6 }}>
        {#each suggestions as suggestion, index (suggestion.value)}
          <View
            style={{
              direction: "row",
              align: "center",
              gap: 12,
              paddingX: 12,
              paddingY: 8,
              radius: theme.radii.control,
              background: rowBackground(index),
              cursor: "pointer",
            }}
            onhover={(isHovered) => highlightOnHover(index, isHovered)}
            onpress={() => choose(suggestion)}
          >
            {#if suggestion.icon}
              <Icon svg={suggestion.icon} size={18} color={theme.colors.muted} style={{ shrink: 0 }} />
            {:else if reservesIconSlot}
              <View style={{ size: 18, shrink: 0 }} />
            {/if}
            <View style={{ flex: 1, minWidth: 0 }}>
              <Text style={{ ...typeStyle("body"), truncate: true, color: theme.colors.text }}>{textOf(suggestion)}</Text>
              {#if suggestion.supportingText}
                <Text style={{ ...typeStyle("bodySmall"), truncate: true, color: theme.colors.muted }}>{suggestion.supportingText}</Text>
              {/if}
            </View>
          </View>
        {/each}
      </Scroll>
    </Overlay>
  {/if}
</View>
