// What View, Button and Overlay share: turning the DOM's events into the host's, as docs/events.md
// describes them, and the states (hovered, pressed, focused) that `hover`, `active` and `focus`
// styles lay over a style.
//
// A listener is only added for a handler the app gave, as in the host, where a listener is what
// makes an element take the event. The press handlers stop the event where they handle it, so
// nothing bubbles: the innermost element with a handler for it hears it, and an element without
// one lets it through to what is around it. Keys are the exception: they bubble, then reach the window.

import type { ViewProps } from "../../ui/props";
import {
  ClickCounter,
  fileDragEventOf,
  keyEventOf,
  pointerEventOf,
  takesButton,
  wheelEventOf,
} from "./events";
import { giveUpFocus, trackFocus } from "./focus";
import { watchLayout } from "./layout";

/** The props that make an element react. */
export type Interactive = Omit<ViewProps, "children" | "style">;

const hasFiles = (event: DragEvent): boolean => Array.from(event.dataTransfer?.types ?? []).includes("Files");

/** Whether a press landed on something that takes text, which keeps focus on it even inside `keepFocus`. */
const inField = (target: EventTarget | null): boolean =>
  target instanceof Element && target.closest("input, textarea, select, [contenteditable]") !== null;

export interface Interaction {
  readonly hovered: boolean;
  readonly pressed: boolean;
  readonly focused: boolean;
  /** Whether the element needs the pointer for itself: no text selecting, no touch panning. */
  readonly grabbing: boolean;
  /** Whether it has anything that listens for the pointer, which is what lets it take presses at all. */
  readonly listening: boolean;
  /** Wires the element up; use it as `{@attach interaction.attach}`. */
  attach(node: HTMLElement): () => void;
  focus(node: HTMLElement | undefined): void;
  blur(node: HTMLElement | undefined): void;
}

/** `read` gives the element's props as they are now; it is read again when they change. */
export function interaction(read: () => Interactive): Interaction {
  trackFocus();
  let hovered = $state(false);
  let pressed = $state(false);
  let focused = $state(false);

  // What the app gave, as flags, so a listener is only taken off and put on when a handler comes or goes.
  const wantsPress = $derived(read().onpress !== undefined);
  const wantsHover = $derived(read().onhover !== undefined || read().hover !== undefined);
  const wantsActive = $derived(read().active !== undefined);
  const wantsFocus = $derived(read().onfocus !== undefined || read().onblur !== undefined || read().focus !== undefined);
  const wantsDown = $derived(read().ondown !== undefined);
  const wantsMove = $derived(read().onmove !== undefined);
  const wantsWheel = $derived(read().onwheel !== undefined);
  const wantsLayout = $derived(read().onlayout !== undefined);
  const wantsResize = $derived(read().onresize !== undefined);
  const wantsKey = $derived(read().onkey !== undefined);
  const wantsKeepFocus = $derived(read().keepFocus === true);
  const wantsFiles = $derived(
    read().ondragenter !== undefined ||
      read().ondragover !== undefined ||
      read().ondragleave !== undefined ||
      read().ondrop !== undefined,
  );
  const wants = {
    get press() {
      return wantsPress;
    },
    get hover() {
      return wantsHover;
    },
    get active() {
      return wantsActive;
    },
    get focus() {
      return wantsFocus;
    },
    get down() {
      return wantsDown;
    },
    get move() {
      return wantsMove;
    },
    get wheel() {
      return wantsWheel;
    },
    get layout() {
      return wantsLayout;
    },
    get resize() {
      return wantsResize;
    },
    get key() {
      return wantsKey;
    },
    get keepFocus() {
      return wantsKeepFocus;
    },
    get files() {
      return wantsFiles;
    },
  };
  const grabbing = $derived(wantsDown || wantsMove);
  const wantsFocusable = $derived(read().focusable === true);
  // As in the host, where only an element with a handler takes the pointer: one without lets presses, hovers
  // and the wheel through to what is behind it, such as a label laid over a canvas.
  const listening = $derived(
    wantsPress ||
      wantsHover ||
      wantsActive ||
      wantsDown ||
      wantsMove ||
      wantsWheel ||
      wantsFiles ||
      wantsFocus ||
      wantsFocusable ||
      wantsKeepFocus,
  );

  function attach(node: HTMLElement): () => void {
    const stops: (() => void)[] = [];
    const on = <E extends Event>(
      target: EventTarget,
      type: string,
      handler: (event: E) => void,
      options?: AddEventListenerOptions,
    ): void => {
      target.addEventListener(type, handler as EventListener, options);
      stops.push(() => target.removeEventListener(type, handler as EventListener, options));
    };
    const origin = (): { left: number; top: number } => node.getBoundingClientRect();
    const clicks = new ClickCounter();

    if (wants.press) {
      on<MouseEvent>(node, "click", (event) => {
        const props = read();
        if (event.button !== 0 || props.onpress === undefined) {
          return;
        }
        event.stopPropagation();
        const box = node.getBoundingClientRect();
        // A key pressing a button reports no pointer: the middle of the element is where it was.
        const keyboard = event.detail === 0 && event.clientX === 0 && event.clientY === 0;
        const { ctrlKey, shiftKey, altKey, metaKey } = event;
        props.onpress(
          pointerEventOf(
            {
              clientX: keyboard ? box.left + box.width / 2 : event.clientX,
              clientY: keyboard ? box.top + box.height / 2 : event.clientY,
              button: 0,
              ctrlKey,
              shiftKey,
              altKey,
              metaKey,
            },
            box,
            Math.max(event.detail, 1),
          ),
        );
      });
    }

    if (wants.hover) {
      on<PointerEvent>(node, "pointerenter", (event) => {
        if (event.pointerType === "touch") {
          return;
        }
        hovered = true;
        read().onhover?.(true);
      });
      on<PointerEvent>(node, "pointerleave", (event) => {
        if (event.pointerType === "touch") {
          return;
        }
        hovered = false;
        read().onhover?.(false);
      });
    }

    // The pointer is captured for the life of a press by listening on the window, which hears the moves and
    // the release wherever they happen. `setPointerCapture` would send the click that follows to this element
    // as well, and take it from the button inside that was pressed.
    let captured: number | null = null;
    const release = (): void => {
      captured = null;
      pressed = false;
    };
    const finish = (event: PointerEvent): void => {
      if (event.pointerId !== captured) {
        return;
      }
      window.removeEventListener("pointermove", follow, true);
      window.removeEventListener("pointerup", finish, true);
      window.removeEventListener("pointercancel", finish, true);
      release();
      read().onup?.(pointerEventOf(event, origin(), 1));
    };
    const follow = (event: PointerEvent): void => {
      if (event.pointerId === captured) {
        read().onmove?.(pointerEventOf(event, origin(), 1));
      }
    };
    if (wants.down || wants.active) {
      on<PointerEvent>(node, "pointerdown", (event) => {
        const props = read();
        if (event.button === 0 && wants.active) {
          pressed = true;
          // A release anywhere ends the press, even when nothing here was listening for it.
          const end = (): void => {
            pressed = false;
            window.removeEventListener("pointerup", end, true);
            window.removeEventListener("pointercancel", end, true);
          };
          window.addEventListener("pointerup", end, true);
          window.addEventListener("pointercancel", end, true);
        }
        if (props.ondown === undefined || !takesButton(event.button, props.buttons)) {
          return;
        }
        event.stopPropagation();
        captured = event.pointerId;
        window.addEventListener("pointermove", follow, true);
        window.addEventListener("pointerup", finish, true);
        window.addEventListener("pointercancel", finish, true);
        const box = node.getBoundingClientRect();
        props.ondown(pointerEventOf(event, box, clicks.press(event.timeStamp, event.clientX, event.clientY)));
      });
    }
    if (wants.move) {
      on<PointerEvent>(node, "pointermove", (event) => {
        if (captured === null) {
          read().onmove?.(pointerEventOf(event, origin(), 1));
        }
      });
    }

    if (wants.wheel) {
      on<WheelEvent>(node, "wheel", (event) => read().onwheel?.(wheelEventOf(event)), { passive: true });
    }

    if (wants.key) {
      on<KeyboardEvent>(node, "keydown", (event) => {
        // A key of an input method's composition belongs to the method.
        if (!event.isComposing) {
          read().onkey?.(keyEventOf(event));
        }
      });
    }

    if (wants.focus) {
      on(node, "focus", () => {
        focused = true;
        read().onfocus?.();
      });
      on(node, "blur", () => {
        focused = false;
        read().onblur?.();
      });
    }

    if (wants.keepFocus) {
      on<MouseEvent>(node, "mousedown", (event) => {
        if (!inField(event.target)) {
          event.preventDefault();
        }
      });
    }

    if (wants.layout) {
      stops.push(watchLayout(node, (layout) => read().onlayout?.(layout)));
    }
    if (wants.resize) {
      stops.push(watchLayout(node, (layout) => read().onresize?.(layout), true));
    }

    if (wants.files) {
      // Dragging into and out of children raises enter and leave on the way, so what is inside is counted.
      let inside = 0;
      const fileEvent = (event: DragEvent, files?: File[]) =>
        fileDragEventOf(event, origin(), files?.length ?? event.dataTransfer?.items.length ?? 0, files);
      on<DragEvent>(node, "dragenter", (event) => {
        if (!hasFiles(event)) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        if (inside++ === 0) {
          read().ondragenter?.(fileEvent(event));
        }
      });
      on<DragEvent>(node, "dragover", (event) => {
        if (!hasFiles(event)) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        event.dataTransfer!.dropEffect = "copy";
        read().ondragover?.(fileEvent(event));
      });
      on<DragEvent>(node, "dragleave", (event) => {
        if (!hasFiles(event)) {
          return;
        }
        event.stopPropagation();
        if (--inside <= 0) {
          inside = 0;
          read().ondragleave?.(fileEvent(event));
        }
      });
      on<DragEvent>(node, "drop", (event) => {
        if (!hasFiles(event)) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        inside = 0;
        read().ondrop?.(fileEvent(event, Array.from(event.dataTransfer?.files ?? [])));
      });
    }

    return () => {
      for (const stop of stops) {
        stop();
      }
      window.removeEventListener("pointermove", follow, true);
      window.removeEventListener("pointerup", finish, true);
      window.removeEventListener("pointercancel", finish, true);
      hovered = pressed = focused = false;
    };
  }

  return {
    get hovered() {
      return hovered;
    },
    get pressed() {
      return pressed;
    },
    get focused() {
      return focused;
    },
    get grabbing() {
      return grabbing;
    },
    get listening() {
      return listening;
    },
    attach,
    focus: (node) => node?.focus(),
    blur: (node) => node && giveUpFocus(node),
  };
}
