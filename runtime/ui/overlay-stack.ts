// The overlays that are open, oldest first. Escape closes only the newest, so a select inside a
// popover closes first, and a second Escape closes the popover.

import { fakeDocument } from "../dom/globals";
import type { SElement } from "../dom/nodes";

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

/** The element that has keyboard focus, to give focus back to when an overlay closes. */
export function focusedElement(): SElement | null {
  return fakeDocument.activeElement as SElement | null;
}

/** Gives focus back to an element, if it is still there. */
export function restoreFocus(element: SElement | null): void {
  if (element !== null && element.parentNode !== null) {
    element.focus();
  }
}
