<!--
  One destination of a NavBar or NavigationRail: an icon in an indicator pill with the label under it.
  Hovering, pressing and keyboard focus show on the pill, as they do on the web.
-->
<script lang="ts">
  import { Icon, Text, View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { createFocusSource } from "./focusSource";
  import Badge from "./Badge.svelte";
  import { iconForState, isActivationKey, type NavItem } from "./navigation";
  import {
    destinationIconSize,
    destinationIndicatorLook,
    destinationLabelLook,
    destinationLook,
    type DestinationShape,
  } from "./navigation-look";

  interface Props {
    item: NavItem;
    active: boolean;
    shape: DestinationShape;
    onselect?: (value: string) => void;
  }

  let { item, active, shape, onselect }: Props = $props();

  let hovered = $state(false);
  let pressed = $state(false);
  let focused = $state(false);
  const focusSource = createFocusSource();

  const theme = $derived(currentTheme());
  const interactionState = $derived({ active, hovered, pressed, focused });

  function select(): void {
    onselect?.(item.value);
  }

  function handleKey(event: KeyEvent): void {
    if (!isActivationKey(event.key)) {
      return;
    }
    event.preventDefault();
    select();
  }

  function handleDown(): void {
    focusSource.pointerDown();
    pressed = true;
  }

  function handleUp(): void {
    pressed = false;
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
  focusable
  style={destinationLook(theme, shape, interactionState)}
  onpress={select}
  onhover={(isHovered) => (hovered = isHovered)}
  ondown={handleDown}
  onup={handleUp}
  onkey={handleKey}
  onfocus={handleFocus}
  onblur={handleBlur}
>
  <View style={destinationIndicatorLook(theme, shape, interactionState)}>
    <Badge count={item.badge} tone="danger">
      <Icon svg={iconForState(item, active)} size={destinationIconSize} />
    </Badge>
  </View>
  <Text style={destinationLabelLook()}>{item.label}</Text>
</View>
