<!--
  A dropdown. `surface` and `ghost` are compact pills for toolbars and rows, `outlined` and `filled` are
  full-height form fields with a label inside. The list opens on press, or with Down, Up, Enter or Space
  while the select has focus. Up and Down move the highlight, Enter or Space choose, and typing jumps to the
  next label that starts with it. With `filter` the list has a search box that takes the typing instead.
-->
<script lang="ts">
  import { Icon, Input, Popover, Scroll, Text, View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import caretDown from "@phosphor-icons/core/regular/caret-down.svg";
  import caretUp from "@phosphor-icons/core/regular/caret-up.svg";
  import magnifyingGlass from "@phosphor-icons/core/regular/magnifying-glass.svg";
  import { checkBold } from "../glyphs";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import {
    firstEnabledIndex,
    isTypeaheadKey,
    movedIndex,
    typeaheadIndex,
    typedTextAfter,
  } from "./listNavigation";
  import {
    glassPanelLook,
    optionRowCheckSize,
    optionRowHeight,
    optionRowIconSize,
    optionRowLook,
  } from "./overlay-look";
  import {
    isFieldVariant,
    selectFieldLabelLook,
    selectIconSizes,
    selectListMaximumHeight,
    selectPanelMinimumWidth,
    selectTriggerLook,
    type SelectSize,
    type SelectVariant,
  } from "./select-look";
  import {
    scrollToReveal,
    selectedValuesOf,
    toggledValues,
    triggerTextOf,
    visibleOptionsOf,
    type SelectFilter,
    type SelectOption,
    type SelectValue,
  } from "./selectModel";
  import type { Style } from "@neoworks/svelte-gpui/ui";

  interface Props {
    /** A single value in single mode, a list of values when `multiple`. */
    value?: SelectValue;
    /** Receives the new value: a string in single mode, a list when `multiple`. */
    onChange?: (value: SelectValue) => void;
    options: SelectOption[];
    /** Names the select. With `outlined` and `filled` it sits inside the field like a text field label. */
    label?: string;
    /** Shown while nothing is chosen. */
    placeholder?: string;
    /** Help under the field, in the danger colour when `error` is on. Only shown by `outlined` and `filled`. */
    supportingText?: string;
    error?: boolean;
    disabled?: boolean;
    /** Allows choosing several options; the list stays open on each pick. */
    multiple?: boolean;
    /** Adds a search box to the list. Decides whether an option matches the typed query. */
    filter?: SelectFilter;
    searchPlaceholder?: string;
    /** Height of the compact variants. */
    size?: SelectSize;
    variant?: SelectVariant;
    /** Shows only the chosen option's icon on the trigger, for narrow toolbars. */
    iconOnly?: boolean;
    /** Layout of the whole select, such as its width. */
    style?: Style;
  }

  let {
    value = $bindable(""),
    onChange,
    options,
    label,
    placeholder = "Select",
    supportingText,
    error = false,
    disabled = false,
    multiple = false,
    filter,
    searchPlaceholder = "Search",
    size = "md",
    variant = "surface",
    iconOnly = false,
    style,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const isField = $derived(isFieldVariant(variant));
  const iconSize = $derived(selectIconSizes[size]);

  let open = $state(false);
  let query = $state("");
  let activeIndex = $state(-1);
  let searchInput: ReturnType<typeof Input> | undefined = $state();
  let scroller: ReturnType<typeof Scroll> | undefined = $state();
  let viewportHeight = $state(0);
  let typedText = "";
  let typedAt = 0;

  const selectedValues = $derived(selectedValuesOf(value));
  const selectedOptions = $derived(options.filter((option) => selectedValues.includes(option.value)));
  const hasSelection = $derived(selectedOptions.length > 0);
  const triggerText = $derived(triggerTextOf(selectedOptions, placeholder));
  const visibleOptions = $derived(visibleOptionsOf(options, filter, query));
  const disabledStates = $derived(visibleOptions.map((option) => option.disabled === true));
  const rowHeights = $derived(visibleOptions.map((option) => optionRowHeight(option.description !== undefined)));
  const floatedLabel = $derived(open || hasSelection);
  const triggerIcon = $derived.by(() => {
    if (multiple || selectedOptions.length === 0) {
      return undefined;
    }
    return selectedOptions[0].icon;
  });
  const caretIcon = $derived.by(() => {
    if (open) {
      return caretUp;
    }
    return caretDown;
  });
  const triggerHover = $derived.by(() => {
    if (disabled) {
      return {};
    }
    return triggerLook.hover;
  });
  const hidesText = $derived(iconOnly && triggerIcon !== undefined);
  const triggerLook = $derived(selectTriggerLook(theme, { variant, size, open, error, disabled }));

  function isSelected(option: SelectOption): boolean {
    return selectedValues.includes(option.value);
  }

  function openList(): void {
    if (disabled) {
      return;
    }
    const firstSelected = visibleOptions.findIndex(isSelected);
    activeIndex = Math.max(firstSelected, firstEnabledIndex(disabledStates), 0);
    open = true;
  }

  function toggleList(): void {
    if (open) {
      open = false;
      return;
    }
    openList();
  }

  function emit(next: SelectValue): void {
    value = next;
    onChange?.(next);
  }

  function pick(option: SelectOption): void {
    if (option.disabled) {
      return;
    }
    if (!multiple) {
      emit(option.value);
      open = false;
      return;
    }
    emit(toggledValues(options, selectedValues, option.value));
  }

  function typeahead(character: string): void {
    const now = Date.now();
    typedText = typedTextAfter(typedText, typedAt, character, now);
    typedAt = now;
    const labels = visibleOptions.map((option) => option.label);
    const target = typeaheadIndex(typedText, activeIndex - 1, labels, disabledStates);
    if (target !== undefined) {
      activeIndex = target;
    }
  }

  function handleOpenKey(event: KeyEvent): void {
    const target = movedIndex(event.key, activeIndex, disabledStates);
    if (target !== undefined) {
      event.preventDefault();
      activeIndex = target;
      return;
    }
    if (event.key === "Enter" || (event.key === " " && filter === undefined)) {
      event.preventDefault();
      pickActive();
      return;
    }
    if (event.key === "Tab") {
      open = false;
      return;
    }
    if (filter === undefined && isTypeaheadKey(event.key, event.ctrl, event.alt, event.meta)) {
      typeahead(event.key);
    }
  }

  function pickActive(): void {
    if (activeIndex >= 0 && activeIndex < visibleOptions.length) {
      pick(visibleOptions[activeIndex]);
    }
  }

  function handleTriggerKey(event: KeyEvent): void {
    if (disabled) {
      return;
    }
    if (open) {
      handleOpenKey(event);
      return;
    }
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(event.key)) {
      event.preventDefault();
      openList();
    }
  }

  function highlight(index: number, hovered: boolean): void {
    if (hovered && !disabledStates[index]) {
      activeIndex = index;
    }
  }

  function handleQueryInput(): void {
    activeIndex = firstEnabledIndex(disabledStates);
  }

  function rowStyle(option: SelectOption, index: number): Style {
    const row = optionRowLook(theme, { active: index === activeIndex, danger: false, disabled: option.disabled === true });
    if (isSelected(option)) {
      row.fontWeight = 500;
    }
    return row;
  }

  function valueColor(): string {
    if (hasSelection) {
      return theme.colors.text;
    }
    return theme.colors.faint;
  }

  function valueStyle(): Style {
    const base: Style = { flex: 1, minWidth: 0, truncate: true, color: valueColor() };
    if (!isField) {
      return base;
    }
    if (floatedLabel) {
      return { ...base, paddingTop: 22, paddingBottom: 6 };
    }
    return { ...base, paddingTop: 22, paddingBottom: 6, color: "transparent" };
  }

  function supportingColor(): string {
    if (error) {
      return theme.colors.danger;
    }
    return theme.colors.muted;
  }

  $effect(() => {
    if (!open) {
      query = "";
    }
  });

  $effect(() => {
    if (open && filter !== undefined) {
      searchInput?.focus();
    }
  });

  $effect(() => {
    if (!open || activeIndex < 0 || scroller === undefined || viewportHeight === 0) {
      return;
    }
    const target = scrollToReveal(rowHeights, activeIndex, scroller.position().y, viewportHeight);
    if (target !== undefined) {
      scroller.scrollTo(target);
    }
  });
</script>

<View style={{ minWidth: 0, ...style }}>
  <Popover
    bind:open
    side="bottom"
    align="start"
    offset={6}
    matchWidth
    focusPanel={false}
    anchorStyle={{ alignSelf: "stretch" }}
    style={glassPanelLook(theme)}
  >
    {#snippet trigger()}
      <View
        focusable
        style={triggerLook.style}
        hover={triggerHover}
        focus={triggerLook.focus}
        onpress={toggleList}
        onkey={handleTriggerKey}
      >
        <View style={{ direction: "row", align: "center", gap: 8, flex: 1, minWidth: 0 }}>
          {#if triggerIcon}
            <Icon svg={triggerIcon} size={iconSize} style={{ shrink: 0 }} />
          {/if}
          {#if !hidesText}
            <Text style={valueStyle()}>{triggerText}</Text>
          {/if}
        </View>
        <Icon svg={caretIcon} size={iconSize} color={theme.colors.muted} style={{ shrink: 0 }} />
        {#if isField && label}
          <Text style={selectFieldLabelLook(theme, { floated: floatedLabel, error, open })}>{label}</Text>
        {/if}
      </View>
    {/snippet}
    {#snippet children()}
      <View style={{ minWidth: selectPanelMinimumWidth }}>
        {#if filter}
          <View
            style={{
              direction: "row",
              align: "center",
              gap: 8,
              paddingX: 16,
              paddingY: 10,
              borderBottomWidth: 1,
              borderColor: theme.colors.line,
            }}
          >
            <Icon svg={magnifyingGlass} size={16} color={theme.colors.faint} style={{ shrink: 0 }} />
            <Input
              bind:this={searchInput}
              bind:value={query}
              placeholder={searchPlaceholder}
              style={{ ...typeStyle("body"), flex: 1, minWidth: 0, background: "transparent", color: theme.colors.text }}
              oninput={handleQueryInput}
              onkey={handleOpenKey}
            />
          </View>
        {/if}
        <Scroll
          bind:this={scroller}
          style={{ maxHeight: selectListMaximumHeight }}
          onlayout={(layout) => (viewportHeight = layout.height)}
        >
          <View style={{ padding: 6 }}>
            {#each visibleOptions as option, index (option.value)}
              <View
                keepFocus
                style={rowStyle(option, index)}
                onhover={(hovered) => highlight(index, hovered)}
                onpress={() => pick(option)}
              >
                {#if option.icon}
                  <Icon svg={option.icon} size={optionRowIconSize} color={theme.colors.muted} style={{ shrink: 0 }} />
                {/if}
                <View style={{ flex: 1, minWidth: 0 }}>
                  <Text style={{ ...typeStyle("body"), truncate: true }}>{option.label}</Text>
                  {#if option.description}
                    <Text style={{ ...typeStyle("bodySmall"), fontWeight: 400, color: theme.colors.muted }}>{option.description}</Text>
                  {/if}
                </View>
                {#if isSelected(option)}
                  <Icon svg={checkBold} size={optionRowCheckSize} style={{ shrink: 0 }} />
                {/if}
              </View>
            {/each}
            {#if visibleOptions.length === 0}
              <Text style={{ ...typeStyle("body"), color: theme.colors.muted, paddingX: 12, paddingY: 8 }}>No results</Text>
            {/if}
          </View>
        </Scroll>
      </View>
    {/snippet}
  </Popover>

  {#if isField && supportingText}
    <Text style={{ ...typeStyle("caption"), color: supportingColor(), marginTop: 6, paddingX: 16 }}>{supportingText}</Text>
  {/if}
</View>
