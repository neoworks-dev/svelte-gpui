// The overlays that are open, oldest first. Escape closes only the newest, so a select inside a
// popover closes first, and a second Escape closes the popover.

export type CloseReason = "escape" | "outside" | "select" | "toggle" | "api";

/** What a popover's trigger and content can use. */
export interface PopoverApi {
  open: boolean;
  show(): void;
  toggle(): void;
  close(reason?: CloseReason): void;
}

interface Entry {
  close(reason: CloseReason): void;
}

const open: Entry[] = [];

/** Records an open overlay. Returns the function that removes it again. */
export function pushOverlay(entry: Entry): () => void {
  open.push(entry);
  return () => {
    const index = open.indexOf(entry);
    if (index >= 0) {
      open.splice(index, 1);
    }
  };
}

/** Whether `entry` is the newest open overlay. */
export function isTopmost(entry: Entry): boolean {
  return open[open.length - 1] === entry;
}

/** An element focus can be given back to: the host's own nodes, or the DOM's. */
export interface FocusTarget {
  parentNode: unknown;
  focus(): void;
}

/** The element that has keyboard focus, to give focus back to when an overlay closes. */
export function focusedElement(): FocusTarget | null {
  return ((globalThis as any).document?.activeElement as FocusTarget | null | undefined) ?? null;
}

/** Gives focus back to an element, if it is still there. */
export function restoreFocus(element: FocusTarget | null): void {
  if (element !== null && element.parentNode !== null) {
    element.focus();
  }
}
