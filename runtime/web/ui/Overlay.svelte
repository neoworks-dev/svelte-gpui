<!--
  Content drawn above everything else, placed against its parent's box. It takes no room where it
  sits, is not cut off by an ancestor that clips, and is drawn in a layer above the app, so nothing
  covers it; of two overlays, the one that appeared later is on top, and a higher `layer` is above
  a lower one. It is placed against the parent's box when it is laid out: on `side` of the parent,
  lined up by `align`, `offset` pixels away, moved to the other side when there is too little room,
  and shifted back inside the window.

  `at` anchors it to a point in the window instead. It draws nothing while it has no children. In
  the page, its element lives in the layer, and a hidden marker stays where it was written, which
  is how it finds its parent, and how keys and presses inside it reach the elements around it.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { OutsideEvent } from "../../ui/events";
  import type { OverlayProps } from "../../ui/props";
  import { layerElement } from "./base";
  import { cssText, layered, toCss } from "./css";
  import { pointerEventOf } from "./events";
  import { interaction } from "./interact.svelte";
  import { watchLayout } from "./layout";
  import { insideOverlay, nextAppearance, parentOf, registerOverlay, stacking } from "./overlays";
  import { place } from "./placement";

  let props: OverlayProps = $props();

  const ix = interaction(() => props);
  let element: HTMLElement | undefined = $state();

  const css = $derived(
    cssText(toCss(layered(props.style, [ix.focused && props.focus, ix.hovered && props.hover, ix.pressed && props.active]))),
  );

  /** Whether anything is in it, which is when it is drawn. */
  const hasContent = (node: HTMLElement): boolean => Array.from(node.childNodes).some((child) => child.nodeType !== 8);

  let appearance = 0;
  let showing = false;

  /** Puts the overlay where its anchor says: against the parent's box, or at a point. */
  function update(node: HTMLElement): void {
    const parent = parentOf(node);
    const present = hasContent(node);
    if (present && !showing) {
      // It just appeared, so it is above what was open before it.
      appearance = nextAppearance();
    }
    showing = present;
    if (!present) {
      return;
    }
    const at = props.at;
    const box = at !== undefined ? { x: at[0], y: at[1], width: 0, height: 0 } : parent?.getBoundingClientRect();
    if (box === undefined) {
      return;
    }
    const anchor = { x: box.x, y: box.y, width: box.width, height: box.height };
    node.style.minWidth = props.matchWidth ? `${anchor.width}px` : "";
    node.style.zIndex = String(stacking(props.layer ?? 0, appearance));
    const placed = place(
      anchor,
      { width: node.offsetWidth, height: node.offsetHeight },
      { width: window.innerWidth, height: window.innerHeight },
      {
        side: props.side ?? "bottom",
        alignment: props.align ?? "start",
        offset: props.offset ?? 0,
        crossOffset: props.crossOffset ?? 0,
        flip: props.flip ?? true,
        shift: props.shift ?? true,
        margin: props.margin ?? 4,
        atPoint: at !== undefined,
      },
    );
    node.style.left = `${Math.round(placed.x)}px`;
    node.style.top = `${Math.round(placed.y)}px`;
  }

  /** Moves the element into the layer, and keeps it placed for as long as it is there. */
  function portal(node: HTMLElement): () => void {
    return untrack(() => {
      // The marker the element was written in, before it moves.
      const holder = node.parentElement!;
      const stop: (() => void)[] = [];
      layerElement().append(node);
      stop.push(registerOverlay(node, holder));

      const relayout = (): void => update(node);
      const parent = holder.parentElement;
      if (parent !== null) {
        stop.push(watchLayout(parent, relayout));
      }
      // Its own size changing, and the content coming and going.
      stop.push(watchLayout(node, relayout, true));
      window.addEventListener("resize", relayout);
      stop.push(() => window.removeEventListener("resize", relayout));
      const changes = new MutationObserver(relayout);
      changes.observe(node, { childList: true, subtree: true, characterData: true });
      stop.push(() => changes.disconnect());

      // A button went down outside it, and outside any overlay opened from inside it.
      const pressed = (event: PointerEvent): void => {
        if (props.onoutside === undefined || !showing || insideOverlay(node, event.target as Node)) {
          return;
        }
        const outside: OutsideEvent = {
          ...pointerEventOf(event, node.getBoundingClientRect(), 1),
          anchor: parent?.contains(event.target as Node) ?? false,
        };
        props.onoutside(outside);
      };
      document.addEventListener("pointerdown", pressed, true);
      stop.push(() => document.removeEventListener("pointerdown", pressed, true));
      relayout();

      return () => {
        for (const undo of stop) {
          undo();
        }
        // The element was moved out of the place Svelte put it in, so Svelte does not remove it.
        node.remove();
      };
    });
  }

  /** Gives it keyboard focus. It must be `focusable`. */
  export function focus(): void {
    ix.focus(element);
  }

  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  export function blur(): void {
    ix.blur(element);
  }
</script>

<div class="g-overlay-marker" style="display:none"
  ><!-- svelte-ignore a11y_no_noninteractive_tabindex --><div
    bind:this={element}
    class="g-overlay"
    style={css}
    data-passthrough={props.passthrough ? "" : undefined}
    tabindex={props.focusable ? -1 : undefined}
    {@attach ix.attach}
    {@attach portal}
  >{@render props.children?.()}</div></div>
