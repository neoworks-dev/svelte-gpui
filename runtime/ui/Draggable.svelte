<!--
  Something that can be picked up with the pointer and dropped on a `DropTarget`. Hold a button
  down on it and move a few pixels: a copy of it follows the pointer, drawn above everything, and
  the original dims. Release over a target that accepts `data` to drop it there; Escape, or releasing
  anywhere else, cancels.

    <Draggable data={{ id: card.id }} onend={({ dropped }) => dropped || console.log("cancelled")}>
      <Text>{card.title}</Text>
    </Draggable>

  Presses on what is inside still work, because the drag only starts once the pointer has moved.
  Only the left button picks it up. A right or middle press passes through to what is around it,
  such as a `ContextMenu` or a view with `ondown`.
-->
<script lang="ts" generics="T">
  import type { Snippet } from "svelte";
  import Overlay from "./Overlay.svelte";
  import View from "./View.svelte";
  import { beginDrag, dragging, endDrag, moveDrag } from "./dnd.svelte";
  import type { Layout, PointerEvent } from "./events";
  import type { Style } from "./style";

  interface Props {
    /** What a target is given when this is dropped on it. */
    data: T;
    disabled?: boolean;
    /** How far the pointer moves, in pixels, before the drag starts. The default is 4. */
    threshold?: number;
    style?: Style;
    hover?: Style;
    /** Laid over `style` on the original while it is being dragged. The default dims it. */
    draggingStyle?: Style;
    /** What follows the pointer. The default is a copy of the children. */
    preview?: Snippet<[T]>;
    /** Laid over the preview's wrapper, which is as wide as the original. */
    previewStyle?: Style;
    onstart?: (data: T) => void;
    /** The drag ended, dropped on a target or not. */
    onend?: (result: { data: T; dropped: boolean }) => void;
    children?: Snippet;
  }

  let {
    data,
    disabled = false,
    threshold = 4,
    style,
    hover,
    draggingStyle = { opacity: 0.35 },
    preview,
    previewStyle,
    onstart,
    onend,
    children,
  }: Props = $props();

  let pressed: { x: number; y: number; grabX: number; grabY: number } | null = null;
  let held = $state(false);
  let grab = $state({ x: 0, y: 0 });
  let size = $state({ width: 0, height: 0 });

  function down(event: PointerEvent): void {
    if (disabled) {
      return;
    }
    pressed = { x: event.windowX, y: event.windowY, grabX: event.x, grabY: event.y };
  }

  function move(event: PointerEvent): void {
    if (pressed === null) {
      return;
    }
    if (!held) {
      if (Math.hypot(event.windowX - pressed.x, event.windowY - pressed.y) < threshold) {
        return;
      }
      grab = { x: pressed.grabX, y: pressed.grabY };
      held = true;
      beginDrag(data, event.windowX, event.windowY);
      onstart?.(data);
      return;
    }
    moveDrag(event.windowX, event.windowY);
  }

  function finish(drop: boolean): void {
    if (held) {
      held = false;
      const dropped = endDrag(drop);
      onend?.({ data, dropped });
    }
  }

  function up(): void {
    finish(true);
    pressed = null;
  }

  function escape(event: KeyboardEvent): void {
    if (held && event.key === "Escape") {
      event.preventDefault();
      finish(false);
      pressed = null;
    }
  }
</script>

<svelte:window onkeydown={escape} />

<View
  style={{ ...style, ...(held ? draggingStyle : undefined) }}
  {hover}
  ondown={down}
  buttons={["left"]}
  onmove={move}
  onup={up}
  onlayout={(layout: Layout) => (size = layout)}
>
  {@render children?.()}
  {#if held}
    <Overlay
      at={[dragging.x - grab.x, dragging.y - grab.y]}
      side="bottom"
      align="start"
      offset={0}
      flip={false}
      passthrough
      layer={2}
      style={{ ...(preview ? undefined : style), width: size.width, opacity: 0.92, shadow: { y: 8, blur: 20, color: "#000000aa" }, ...previewStyle }}
    >
      {#if preview}{@render preview(data)}{:else}{@render children?.()}{/if}
    </Overlay>
  {/if}
</View>
