<!--
  A menu of actions that opens under a trigger. `trigger` receives `onpress` to give a button. Up and Down
  move the highlight, Home and End jump, Enter or Space choose, and typing jumps to the next label that
  starts with it. Escape and an outside press close it. The pointer highlights what it is over and chooses
  on press.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Popover, Text, View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import { checkBold } from "../glyphs";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import {
    firstEnabledIndex,
    isTypeaheadKey,
    lastEnabledIndex,
    movedIndex,
    typeaheadIndex,
    typedTextAfter,
  } from "./listNavigation";
  import { menuActions, menuRows, type MenuEntry, type MenuItem, type MenuTriggerProps } from "./menuModel";
  import {
    glassPanelLook,
    menuHeadingLook,
    menuPanelMinimumWidth,
    menuPanelPadding,
    menuShortcutLook,
    optionRowCheckSize,
    optionRowIconSize,
    optionRowLook,
  } from "./overlay-look";
  import { parsePlacement, type FloatingPlacement } from "./placement";

  interface Props {
    open?: boolean;
    items: MenuEntry[];
    /** The element that opens the menu. Give the `onpress` it receives to a button. */
    trigger: Snippet<[MenuTriggerProps]>;
    placement?: FloatingPlacement;
  }

  let { open = $bindable(false), items, trigger: renderTrigger, placement = "bottom-start" }: Props = $props();

  const theme = $derived(currentTheme());
  const side = $derived(parsePlacement(placement).side);
  const align = $derived(parsePlacement(placement).align);
  const rows = $derived(menuRows(items));
  const actions = $derived(menuActions(items));
  const disabledStates = $derived(actions.map((item) => item.disabled === true));
  const hasCheckableItems = $derived(actions.some((item) => item.selected !== undefined));

  let activeIndex = $state(-1);
  let typedText = "";
  let typedAt = 0;

  function openAt(position: "first" | "last"): void {
    if (position === "first") {
      activeIndex = firstEnabledIndex(disabledStates);
    } else {
      activeIndex = lastEnabledIndex(disabledStates);
    }
    open = true;
  }

  function toggle(): void {
    if (open) {
      open = false;
      return;
    }
    openAt("first");
  }

  function handleTriggerKey(event: KeyEvent): void {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      openAt("first");
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      openAt("last");
    }
  }

  function choose(actionIndex: number): void {
    const item = actions[actionIndex];
    if (item === undefined || item.disabled) {
      return;
    }
    open = false;
    item.onselect?.();
  }

  function highlight(actionIndex: number, hovered: boolean): void {
    if (hovered && !disabledStates[actionIndex]) {
      activeIndex = actionIndex;
    }
  }

  function typeahead(character: string): void {
    const now = Date.now();
    typedText = typedTextAfter(typedText, typedAt, character, now);
    typedAt = now;
    const labels = actions.map((item) => item.label);
    const target = typeaheadIndex(typedText, activeIndex - 1, labels, disabledStates);
    if (target !== undefined) {
      activeIndex = target;
    }
  }

  function handlePanelKey(event: KeyEvent): void {
    if (event.key === "Tab") {
      open = false;
      return;
    }
    const target = movedIndex(event.key, activeIndex, disabledStates);
    if (target !== undefined) {
      event.preventDefault();
      activeIndex = target;
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      choose(activeIndex);
      return;
    }
    if (isTypeaheadKey(event.key, event.ctrl, event.alt, event.meta)) {
      typeahead(event.key);
    }
  }

  function rowLook(item: MenuItem, actionIndex: number) {
    return optionRowLook(theme, {
      active: actionIndex === activeIndex && item.disabled !== true,
      danger: item.danger === true,
      disabled: item.disabled === true,
    });
  }
</script>

<Popover
  bind:open
  {side}
  {align}
  offset={6}
  style={{ ...glassPanelLook(theme), padding: menuPanelPadding, minWidth: menuPanelMinimumWidth }}
  onkey={handlePanelKey}
>
  {#snippet trigger()}
    <View style={{ direction: "row" }} onkey={handleTriggerKey}>
      {@render renderTrigger({ onpress: toggle, expanded: open })}
    </View>
  {/snippet}
  {#snippet children()}
    {#each rows as row}
      {#if row.entry.type === "divider"}
        <View style={{ height: 1, marginY: 6, background: theme.colors.line, shrink: 0 }} />
      {:else if row.entry.type === "heading"}
        <Text style={menuHeadingLook(theme)}>{row.entry.label}</Text>
      {:else}
        {@const item = row.entry}
        <View
          style={rowLook(item, row.actionIndex)}
          onhover={(hovered) => highlight(row.actionIndex, hovered)}
          onpress={() => choose(row.actionIndex)}
        >
          {#if hasCheckableItems}
            <View style={{ size: optionRowCheckSize, shrink: 0, align: "center", justify: "center" }}>
              {#if item.selected}
                <Icon svg={checkBold} size={optionRowCheckSize} />
              {/if}
            </View>
          {/if}
          {#if item.icon}
            <Icon svg={item.icon} size={optionRowIconSize} style={{ shrink: 0 }} />
          {/if}
          <Text style={{ ...typeStyle("body"), flex: 1, nowrap: true }}>{item.label}</Text>
          {#if item.shortcut}
            <Text style={menuShortcutLook(theme)}>{item.shortcut}</Text>
          {/if}
        </View>
      {/if}
    {/each}
  {/snippet}
</Popover>
