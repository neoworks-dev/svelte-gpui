import type { ScrollPosition } from "./events";

/** Where a scroller ends up when moved by `delta` along its axis: never before the start nor past the furthest offset. */
export function offsetAfter(position: ScrollPosition, horizontal: boolean, delta: number): number {
  const offset = horizontal ? position.x : position.y;
  const furthest = horizontal ? position.maxX : position.maxY;
  return Math.min(Math.max(offset + delta, 0), furthest);
}
