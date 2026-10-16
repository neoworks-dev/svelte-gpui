// Dragging inside the app. A `Draggable` starts a drag when the pointer, held down on it, has
// moved a few pixels. The pointer is captured, so it keeps hearing moves wherever they go, and
// each move is matched against the `DropTarget`s, which are known by where they were laid out.
// The innermost target under the pointer that accepts the dragged data hears `dragover`, and
// the one under it at release hears `drop`.

import type { Layout } from "./events";

/** What a drop target is told about a drag. */
export interface DragInfo<T = unknown> {
  /** What the draggable carries. */
  data: T;
  /** The pointer inside the target, from its top left. */
  x: number;
  y: number;
  /** The pointer in the window. */
  windowX: number;
  windowY: number;
  /** The target's size, for working out which half or edge the pointer is nearer. */
  width: number;
  height: number;
}

/** A drag in progress, readable from anywhere: show a hint on every target that could take it. */
export const dragging: { active: boolean; data: unknown; x: number; y: number } = $state({
  active: false,
  data: undefined,
  x: 0,
  y: 0,
});

export interface Target {
  bounds: Layout;
  accept?: (data: any) => boolean;
  enter(info: DragInfo): void;
  leave(): void;
  over(info: DragInfo): void;
  drop(info: DragInfo): void;
}

const targets = new Set<Target>();
let current: Target | null = null;

/** Makes a target known. Returns the function that removes it. */
export function addTarget(target: Target): () => void {
  targets.add(target);
  return () => {
    targets.delete(target);
    if (current === target) {
      current = null;
    }
  };
}

/** Whether a target would take the data being dragged now. */
export function accepts(target: { accept?: (data: any) => boolean }): boolean {
  return dragging.active && (target.accept === undefined || target.accept(dragging.data));
}

/** The innermost target containing the point that accepts the data: the smallest box wins, a later one on a tie. */
function targetAt(x: number, y: number, data: unknown): Target | null {
  let best: Target | null = null;
  let bestArea = Infinity;
  for (const target of targets) {
    const box = target.bounds;
    if (x < box.x || x > box.x + box.width || y < box.y || y > box.y + box.height) {
      continue;
    }
    if (target.accept !== undefined && !target.accept(data)) {
      continue;
    }
    const area = box.width * box.height;
    if (area <= bestArea) {
      best = target;
      bestArea = area;
    }
  }
  return best;
}

function infoFor(target: Target, windowX: number, windowY: number): DragInfo {
  const box = target.bounds;
  return {
    data: dragging.data,
    x: windowX - box.x,
    y: windowY - box.y,
    windowX,
    windowY,
    width: box.width,
    height: box.height,
  };
}

/** Begins a drag carrying `data`. */
export function beginDrag(data: unknown, windowX: number, windowY: number): void {
  dragging.active = true;
  dragging.data = data;
  dragging.x = windowX;
  dragging.y = windowY;
  moveDrag(windowX, windowY);
}

/** The pointer moved: enter, leave and over reach the targets. */
export function moveDrag(windowX: number, windowY: number): void {
  dragging.x = windowX;
  dragging.y = windowY;
  const target = targetAt(windowX, windowY, dragging.data);
  if (target !== current) {
    current?.leave();
    current = target;
    current?.enter(infoFor(target!, windowX, windowY));
  }
  current?.over(infoFor(current, windowX, windowY));
}

/** Ends the drag. With `drop`, the target under the pointer takes it; the result says whether one did. */
export function endDrag(drop: boolean): boolean {
  const target = current;
  let dropped = false;
  if (target !== null && drop) {
    target.drop(infoFor(target, dragging.x, dragging.y));
    dropped = true;
  }
  target?.leave();
  current = null;
  dragging.active = false;
  dragging.data = undefined;
  return dropped;
}
