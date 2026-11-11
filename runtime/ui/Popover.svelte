<!--
  Content that floats next to a trigger: a menu, a date picker, a card of details. It is drawn
  above everything else, placed against the trigger's box on the `side` you choose, flipped to the
  other side and shifted to stay inside the window, and closed by an outside press or Escape.

    <Popover bind:open side="bottom" align="start">
      {#snippet trigger({ toggle })}
        <Button onpress={toggle}><Text>Options</Text></Button>
      {/snippet}
      <Text>Anything goes here</Text>
    </Popover>

  Focus moves into the popover when it opens (to the panel itself, or to whatever inside it takes
  focus first when `focusPanel` is off) and goes back to what had it when it closes by Escape or
  a choice. After an outside press focus stays where that press put it.

  Without a `trigger`, give `at` to place it at a point in the window, such as under the pointer.
  The popover is then placed against its parent's box when `at` is absent, as `Overlay` is.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { Overlay, View } from "#primitives";
  import {
    focusedElement,
    isTopmost,
    pushOverlay,
    restoreFocus,
    type CloseReason,
    type FocusTarget,
    type PopoverApi,
  } from "./overlay-stack";
  import { panel as panelLook } from "./overlay-look";
  import type { OutsideEvent } from "./events";
  import type { PopoverProps } from "./props";

  let {
    open = $bindable(false),
    trigger,
    children,
    side = "bottom",
    align = "start",
    offset = 6,
    crossOffset,
    flip,
    shift,
    margin,
    matchWidth,
    at,
    style = panelLook,
    anchorStyle,
    closeOnOutside = true,
    closeOnEscape = true,
    focusPanel = true,
    restoreFocus: giveFocusBack = true,
    layer,
    onclose,
    onkey,
  }: PopoverProps = $props();

  let overlay: ReturnType<typeof Overlay> | undefined = $state();
  let before: FocusTarget | null = null;
  let reason: CloseReason | undefined;

  function close(why: CloseReason = "api"): void {
    if (!open) {
      return;
    }
    reason = why;
    open = false;
    onclose?.(why);
  }

  const api: PopoverApi = {
    get open() {
      return open;
    },
    show: () => (open = true),
    toggle: () => (open ? close("toggle") : (open = true)),
    close,
  };
  const entry = { close };

  // While open: remember where focus was, move it in, and put it back when this closes.
  $effect(() => {
    if (!open) {
      return;
    }
    const leave = pushOverlay(entry);
    untrack(() => {
      before = focusedElement();
      reason = undefined;
      if (focusPanel) {
        overlay?.focus();
      }
    });
    return () => {
      leave();
      if (giveFocusBack && reason !== "outside") {
        restoreFocus(before);
      }
      before = null;
    };
  });

  function escape(event: KeyboardEvent): void {
    if (open && closeOnEscape && event.key === "Escape" && !event.defaultPrevented && isTopmost(entry)) {
      event.preventDefault();
      close("escape");
    }
  }

  function outside(event: OutsideEvent): void {
    if (closeOnOutside && !event.anchor) {
      close("outside");
    }
  }
</script>

<svelte:window onkeydown={escape} />

{#snippet floating()}
  <Overlay
    bind:this={overlay}
    {side}
    {align}
    {offset}
    {crossOffset}
    {flip}
    {shift}
    {margin}
    {matchWidth}
    {at}
    {layer}
    {style}
    focusable
    onoutside={outside}
    {onkey}
  >
    {#if open}{@render children?.(api)}{/if}
  </Overlay>
{/snippet}

{#if trigger}
  <View style={{ alignSelf: "start", ...anchorStyle }}>
    {@render trigger(api)}
    {@render floating()}
  </View>
{:else}
  {@render floating()}
{/if}
