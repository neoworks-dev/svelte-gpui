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
  import { untrack, type Snippet } from "svelte";
  import Overlay from "./Overlay.svelte";
  import View from "./View.svelte";
  import { focusedElement, isTopmost, pushOverlay, restoreFocus, type CloseReason, type PopoverApi } from "./overlay-stack";
  import { panel as panelLook } from "./overlay-look";
  import type { KeyEvent, OutsideEvent } from "./events";
  import type { Style } from "./style";
  import type { SElement } from "../dom/nodes";

  interface Props {
    open?: boolean;
    trigger?: Snippet<[PopoverApi]>;
    children?: Snippet<[PopoverApi]>;
    side?: "top" | "bottom" | "left" | "right";
    align?: "start" | "center" | "end";
    /** The gap between the trigger and the popover. The default is 6. */
    offset?: number;
    crossOffset?: number;
    flip?: boolean;
    shift?: boolean;
    margin?: number;
    /** At least as wide as the trigger. */
    matchWidth?: boolean;
    /** A point in the window to place it against, instead of the trigger. */
    at?: [number, number];
    /** The panel's look. The default is a dark card; pass `{}` for none. */
    style?: Style;
    /** The wrapper around the trigger, whose box the popover is placed against. */
    anchorStyle?: Style;
    closeOnOutside?: boolean;
    closeOnEscape?: boolean;
    /** Moves focus to the panel when it opens. Turn it off when the content takes focus itself. */
    focusPanel?: boolean;
    /** Gives focus back to what had it when it closes by Escape or a choice. The default is on. */
    restoreFocus?: boolean;
    layer?: number;
    onclose?: (reason: CloseReason) => void;
    onkey?: (event: KeyEvent) => void;
  }

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
  }: Props = $props();

  let overlay: ReturnType<typeof Overlay> | undefined = $state();
  let before: SElement | null = null;
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
