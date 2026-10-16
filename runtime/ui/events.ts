// What the host reports with each event. Pointer positions come in the element's own
// coordinates (`x`, `y`) when the element is measured, and always in the window's.

export interface Modifiers {
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
}

export interface PointerEvent extends Modifiers {
  x: number;
  y: number;
  windowX: number;
  windowY: number;
  /** 0 left, 1 middle, 2 right; -1 for movement. */
  button: number;
  clicks: number;
}

export interface WheelEvent extends Modifiers {
  dx: number;
  dy: number;
}

export interface KeyEvent extends Modifiers {
  /** `"Enter"`, `"ArrowLeft"`, `"F1"`, or the character typed. */
  key: string;
  code: string;
  repeat: boolean;
  /** Stops the host's own handling, such as a field typing the key. */
  preventDefault(): void;
  /** Stops the key going further up, and to `window`. */
  stopPropagation(): void;
}

export interface Layout {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Files dragged in from another program. `paths` are absolute; only `ondrop` makes them readable by the app. */
export interface FileDragEvent {
  /** The pointer inside the element, and in the window. */
  x: number;
  y: number;
  windowX: number;
  windowY: number;
  paths: string[];
  count: number;
}

/** A button went down outside an overlay. */
export interface OutsideEvent extends PointerEvent {
  /** It went down on the element the overlay is anchored to, which usually handles the press itself. */
  anchor: boolean;
}

export interface ScrollPosition {
  x: number;
  y: number;
  /** How far it can scroll. */
  maxX: number;
  maxY: number;
}

/** Turns an optional callback into a listener, so an absent one adds no listener at all. */
export function listener<E, R>(callback: ((value: R) => void) | undefined, read: (event: E) => R) {
  if (callback === undefined) {
    return undefined;
  }
  return (event: E) => callback(read(event));
}

export const itself = <E>(event: E): E => event;
