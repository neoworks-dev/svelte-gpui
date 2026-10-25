<!-- One tab of a Tabs list: icon, label and count, with the line under it while it is active. -->
<script lang="ts">
  import { Icon, Text, View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { createFocusSource } from "./focusSource";
  import { iconForState } from "./navigation";
  import {
    tabCountLook,
    tabIconSize,
    tabIndicatorLook,
    tabLook,
    tabWashLook,
    type TabsOptions,
  } from "./tabs-look";
  import type { TabItem } from "./types";

  interface Props {
    tab: TabItem;
    active: boolean;
    options: TabsOptions;
    onselect: () => void;
    onkey: (event: KeyEvent) => void;
  }

  let { tab, active, options, onselect, onkey }: Props = $props();

  let element: ReturnType<typeof View> | undefined = $state();
  let hovered = $state(false);
  let pressed = $state(false);
  let focused = $state(false);
  const focusSource = createFocusSource();

  const theme = $derived(currentTheme());
  const disabled = $derived(tab.disabled === true);
  const interactionState = $derived({ active, hovered, pressed, focused, hasIcon: tab.icon !== undefined, disabled });
  const icon = $derived(iconForState(tab, active));

  export function focus(): void {
    element?.focus();
  }

  function handleDown(): void {
    focusSource.pointerDown();
    pressed = true;
  }

  function handleUp(): void {
    pressed = false;
  }

  function handlePress(): void {
    if (!disabled) {
      onselect();
    }
  }

  function handleBlur(): void {
    focused = false;
    focusSource.blurred();
  }

  function handleFocus(): void {
    focused = focusSource.focused();
  }
</script>

<View
  bind:this={element}
  focusable={!disabled}
  style={tabLook(theme, options, interactionState)}
  onpress={handlePress}
  onhover={(isHovered) => (hovered = isHovered && !disabled)}
  ondown={handleDown}
  onup={handleUp}
  onkey={onkey}
  onfocus={handleFocus}
  onblur={handleBlur}
>
  <View style={tabWashLook(theme, interactionState)} />
  {#if icon}
    <Icon svg={icon} size={tabIconSize} />
  {/if}
  <View style={{ direction: "row", align: "center", gap: 8 }}>
    <Text>{tab.label}</Text>
    {#if tab.count !== undefined}
      <Text style={tabCountLook(theme)}>{tab.count}</Text>
    {/if}
  </View>
  <View style={tabIndicatorLook(theme, options, active)} />
</View>
