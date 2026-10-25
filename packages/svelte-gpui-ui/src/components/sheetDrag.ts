// Dragging a bottom sheet down by its handle to close it.

/** How far down the handle has to be dragged for the sheet to close. */
export const dismissDistance = 100;

/** Movement under this many pixels still counts as a press on the handle. */
export const dragThreshold = 4;

/** The sheet's downward shift while the pointer is at `currentY`. It never moves up past its resting place. */
export function dragOffset(startY: number, currentY: number): number {
  return Math.max(0, currentY - startY);
}

export type DragOutcome = "close" | "settle" | "press";

/** What releasing the handle does: a drag past the distance closes, a shorter drag springs back, a press closes. */
export function dragOutcome(maximumOffset: number, releaseOffset: number): DragOutcome {
  if (releaseOffset > dismissDistance) {
    return "close";
  }
  if (maximumOffset > dragThreshold) {
    return "settle";
  }
  return "press";
}
