// The arithmetic of a wheel column: which row is centred, where a wheel turn or a drag lands, which rows are drawn.

export const wheelItemHeight = 32;
export const wheelVisibleItems = 5;
export const wheelHeight = wheelItemHeight * wheelVisibleItems;

/** Wheel movement, in pixels, that turns the column by one row. */
export const wheelPixelsPerItem = 24;

/** A pointer drag shorter than this still counts as a press on a row. */
export const dragThreshold = 4;

/** `index` brought into `0` to `length - 1` by going round. */
export function wrapIndex(index: number, length: number): number {
  return ((index % length) + length) % length;
}

export function clampIndex(index: number, length: number): number {
  return Math.min(Math.max(index, 0), length - 1);
}

/**
 * The index to turn the column to so that `values[valueIndex]` is centred. A wrapping column takes the nearest
 * copy of the value to `currentIndex`, so the turn goes the short way round; a bounded one has only one copy.
 */
export function targetIndexFor(currentIndex: number, valueIndex: number, length: number, wrap: boolean): number {
  if (!wrap) {
    return valueIndex;
  }
  const shortestTurn = wrapIndex(valueIndex - currentIndex + Math.floor(length / 2), length) - Math.floor(length / 2);
  return currentIndex + shortestTurn;
}

/** The index after turning by `steps` rows. A bounded column stops at its ends. */
export function indexAfterTurn(currentIndex: number, steps: number, length: number, wrap: boolean): number {
  if (!wrap) {
    return clampIndex(currentIndex + steps, length);
  }
  return currentIndex + steps;
}

/** The row centred after a drag of `dragPixels` downwards from `startIndex`, which pulls earlier rows into the centre. */
export function indexAfterDrag(startIndex: number, dragPixels: number): number {
  return startIndex - dragPixels / wheelItemHeight;
}

export interface WheelTurn {
  steps: number;
  accumulated: number;
}

/**
 * Wheel movement `deltaY` added to what is still unspent in `accumulated`. A turn that follows a pause moves one row at
 * once, so a single notch of a mouse wheel never gets lost; the rest of a long gesture moves a row per
 * `wheelPixelsPerItem`.
 */
export function turnAfterWheel(accumulated: number, deltaY: number, idle: boolean): WheelTurn {
  if (deltaY === 0) {
    return { steps: 0, accumulated };
  }
  let total = accumulated + deltaY;
  if (idle) {
    total = Math.sign(deltaY) * wheelPixelsPerItem;
  }
  const steps = Math.trunc(total / wheelPixelsPerItem);
  return { steps, accumulated: total - steps * wheelPixelsPerItem };
}

/** The rows around the centre that need drawing while the column is at `position` (a fractional index). */
export function visibleIndexes(position: number, length: number, wrap: boolean): number[] {
  const centre = Math.round(position);
  const reach = Math.floor(wheelVisibleItems / 2) + 1;
  const indexes: number[] = [];
  for (let index = centre - reach; index <= centre + reach; index++) {
    if (wrap || (index >= 0 && index < length)) {
      indexes.push(index);
    }
  }
  return indexes;
}

/** The pixel offset of row `index` from the top of the column while the column is at `position`. */
export function rowTop(index: number, position: number): number {
  return (index - position) * wheelItemHeight + Math.floor(wheelVisibleItems / 2) * wheelItemHeight;
}

export function valueAt(values: number[], index: number): number {
  return values[wrapIndex(index, values.length)];
}
