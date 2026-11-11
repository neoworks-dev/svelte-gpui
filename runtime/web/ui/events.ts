// What the DOM reports, as what the host reports: the event objects the props of every primitive
// hand their handlers (docs/events.md). The functions take the few fields they read, so a test can
// give them plain objects.

import type { DroppedFile, FileDragEvent, KeyEvent, Modifiers, PointerEvent, WheelEvent } from "../../ui/events";

interface HasModifiers {
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}

export function modifiersOf(event: HasModifiers): Modifiers {
  return { ctrl: event.ctrlKey, shift: event.shiftKey, alt: event.altKey, meta: event.metaKey };
}

/** The place an element's box starts at, in the window. */
export interface Origin {
  left: number;
  top: number;
}

export type ButtonName = "left" | "middle" | "right";

const BUTTONS: readonly ButtonName[] = ["left", "middle", "right"];

/** Whether a button, in the DOM's numbering (0 left, 1 middle, 2 right), is one of those named. */
export function takesButton(button: number, buttons: readonly ButtonName[] | undefined): boolean {
  return buttons === undefined || buttons.includes(BUTTONS[button]);
}

interface PointerFields extends HasModifiers {
  clientX: number;
  clientY: number;
  button: number;
}

/** A pointer event in the element's own coordinates and in the window's. `clicks` counts presses in a row. */
export function pointerEventOf(event: PointerFields, origin: Origin, clicks = 1): PointerEvent {
  return {
    x: event.clientX - origin.left,
    y: event.clientY - origin.top,
    windowX: event.clientX,
    windowY: event.clientY,
    button: event.button,
    clicks,
    ...modifiersOf(event),
  };
}

interface WheelFields extends HasModifiers {
  deltaX: number;
  deltaY: number;
  deltaMode: number;
}

/** The pixels a line and a page are worth, for wheels that report either. */
const LINE = 16;
const PAGE = 800;

export function wheelEventOf(event: WheelFields): WheelEvent {
  const unit = event.deltaMode === 1 ? LINE : event.deltaMode === 2 ? PAGE : 1;
  return { dx: event.deltaX * unit, dy: event.deltaY * unit, ...modifiersOf(event) };
}

interface KeyFields extends HasModifiers {
  key: string;
  code: string;
  repeat: boolean;
  preventDefault(): void;
  stopPropagation(): void;
}

/** A key, which can stop what the page would do with it and stop it reaching what is around. */
export function keyEventOf(event: KeyFields): KeyEvent {
  return {
    key: event.key,
    code: event.code,
    repeat: event.repeat,
    ...modifiersOf(event),
    preventDefault: () => event.preventDefault(),
    stopPropagation: () => event.stopPropagation(),
  };
}

/** A file a page was given, which is everything the web lets an app know of it. */
export type { DroppedFile };

interface DragFields {
  clientX: number;
  clientY: number;
}

/**
 * Files dragged over an element. A browser tells a page the paths of no file, so `paths` is empty
 * and the files themselves, once dropped, are in `files`. Until they drop, only how many there are is known.
 */
export function fileDragEventOf(event: DragFields, origin: Origin, count: number, files?: DroppedFile[]): FileDragEvent {
  const dropped: FileDragEvent = {
    x: event.clientX - origin.left,
    y: event.clientY - origin.top,
    windowX: event.clientX,
    windowY: event.clientY,
    paths: [],
    count,
  };
  if (files !== undefined) {
    dropped.files = files;
  }
  return dropped;
}

/** Counts presses in a row on one element, so that a double click arrives as `clicks: 2`. */
export class ClickCounter {
  private count = 0;
  private at = 0;
  private x = 0;
  private y = 0;

  /** The count for a press at this time and place: one more than the last if it came soon after, close by. */
  press(time: number, x: number, y: number, within = 500, near = 5): number {
    const again = this.count > 0 && time - this.at <= within && Math.hypot(x - this.x, y - this.y) <= near;
    this.count = again ? this.count + 1 : 1;
    this.at = time;
    this.x = x;
    this.y = y;
    return this.count;
  }
}
