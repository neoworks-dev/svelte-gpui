<!--
  A short message at the bottom of the window about something that just happened, with at most one action
  such as "Undo". It closes itself after `duration`, and stays open while the pointer is over it.
-->
<script lang="ts">
  import { fly } from "svelte/transition";
  import { window } from "@neoworks/svelte-gpui";
  import { Button, Icon, Overlay, Presence, Text, View } from "@neoworks/svelte-gpui/ui";
  import closeIcon from "@phosphor-icons/core/regular/x.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import { withAlpha } from "./colorMix";
  import { visibleShadows } from "./look";

  interface Props {
    open?: boolean;
    /** One short sentence about what just happened, such as "Moved to the bin". */
    message: string;
    /** A word or two that lets people react, such as "Undo". */
    actionLabel?: string;
    onaction?: () => void;
    /** Adds a close button. Useful when the snackbar stays until dismissed. */
    dismissible?: boolean;
    /** Milliseconds before it closes by itself. 0 keeps it until dismissed. */
    duration?: number;
    onclose?: () => void;
  }

  let { open = $bindable(false), message, actionLabel, onaction, dismissible = false, duration = 5000, onclose }: Props = $props();

  const theme = $derived(currentTheme());
  const bottomGap = 24;
  const snackbarLayer = 2;
  const liftMilliseconds = 300;

  let hovered = $state(false);

  function close(): void {
    open = false;
    onclose?.();
  }

  function runAction(): void {
    onaction?.();
    close();
  }

  $effect(() => {
    if (!open || hovered || duration <= 0) {
      return;
    }
    const timer = setTimeout(close, duration);
    return () => clearTimeout(timer);
  });
</script>

<Overlay
  at={[window.size.width / 2, window.size.height - bottomGap]}
  side="top"
  align="center"
  offset={0}
  flip={false}
  shift={false}
  layer={snackbarLayer}
  onhover={(isHovered) => (hovered = isHovered)}
>
  {#if open}
    <Presence in={[fly, { y: 16, duration: liftMilliseconds }]}>
      <View
        style={{
          ...typeStyle("body"),
          direction: "row",
          align: "center",
          gap: 8,
          maxWidth: 384,
          paddingY: 8,
          paddingRight: 8,
          paddingLeft: 20,
          radius: theme.radii.control,
          background: theme.colors.fill,
          color: theme.colors.onFill,
          shadow: visibleShadows(theme.shadows.overlay),
        }}
      >
        <Text style={{ flex: 1, minWidth: 0, paddingY: 6 }}>{message}</Text>
        {#if actionLabel}
          <Button
            style={{ paddingX: 12, paddingY: 6, radius: 999, fontWeight: 500, cursor: "pointer", shrink: 0, transition: 300 }}
            hover={{ background: withAlpha(theme.colors.onFill, 0.12) }}
            onpress={runAction}
          >
            <Text>{actionLabel}</Text>
          </Button>
        {/if}
        {#if dismissible}
          <Button
            style={{ size: 32, radius: 16, align: "center", justify: "center", cursor: "pointer", shrink: 0, transition: 300 }}
            hover={{ background: withAlpha(theme.colors.onFill, 0.12) }}
            onpress={close}
          >
            <Icon svg={closeIcon} size={16} color={theme.colors.onFill} />
          </Button>
        {/if}
      </View>
    </Presence>
  {/if}
</Overlay>
