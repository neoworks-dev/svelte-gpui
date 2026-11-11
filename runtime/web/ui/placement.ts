// Where an overlay goes: the same rules as the host's (`host/src/render/overlay.rs`), so a popover
// opens on the same side, and flips and shifts at the same points, on both targets.

export type Side = "top" | "bottom" | "left" | "right";
export type Alignment = "start" | "center" | "end";

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Size {
  width: number;
  height: number;
}

/** What the app asked for. */
export interface Wanted {
  side: Side;
  alignment: Alignment;
  /** The gap between the anchor and the overlay, along the side. */
  offset: number;
  /** A shift along the edge, toward the end. */
  crossOffset: number;
  /** Moves to the opposite side when the preferred one has too little room. */
  flip: boolean;
  /** Slides back inside the window when it would stick out. */
  shift: boolean;
  /** The least distance kept from the window's edges. */
  margin: number;
  /** The anchor is a point rather than a box, so alignment flips like the side does. */
  atPoint: boolean;
}

const OPPOSITE: Record<Side, Side> = { top: "bottom", bottom: "top", left: "right", right: "left" };
const OPPOSITE_ALIGNMENT: Record<Alignment, Alignment> = { start: "end", center: "center", end: "start" };

const vertical = (side: Side): boolean => side === "top" || side === "bottom";

/** The preferred side, or the opposite one when the preferred has too little room and the other has more. */
function chosenSide(anchor: Rect, size: Size, window: Size, wanted: Wanted): Side {
  if (!wanted.flip) {
    return wanted.side;
  }
  const room = (side: Side): number => {
    switch (side) {
      case "bottom":
        return window.height - wanted.margin - (anchor.y + anchor.height) - wanted.offset;
      case "top":
        return anchor.y - wanted.offset - wanted.margin;
      case "right":
        return window.width - wanted.margin - (anchor.x + anchor.width) - wanted.offset;
      case "left":
        return anchor.x - wanted.offset - wanted.margin;
    }
  };
  const needed = vertical(wanted.side) ? size.height : size.width;
  const preferred = room(wanted.side);
  const other = room(OPPOSITE[wanted.side]);
  return preferred < needed && other > preferred ? OPPOSITE[wanted.side] : wanted.side;
}

/** Where the overlay's top left corner goes for a side and an alignment, before keeping it in the window. */
function corner(anchor: Rect, size: Size, wanted: Wanted, side: Side, alignment: Alignment): [number, number] {
  const along = (start: number, length: number, overlay: number): number => {
    switch (alignment) {
      case "start":
        return start + wanted.crossOffset;
      case "center":
        return start + (length - overlay) / 2 + wanted.crossOffset;
      case "end":
        return start + length - overlay - wanted.crossOffset;
    }
  };
  switch (side) {
    case "bottom":
      return [along(anchor.x, anchor.width, size.width), anchor.y + anchor.height + wanted.offset];
    case "top":
      return [along(anchor.x, anchor.width, size.width), anchor.y - wanted.offset - size.height];
    case "right":
      return [anchor.x + anchor.width + wanted.offset, along(anchor.y, anchor.height, size.height)];
    case "left":
      return [anchor.x - wanted.offset - size.width, along(anchor.y, anchor.height, size.height)];
  }
}

/** Moves a span of `length` so it lies inside `0..limit` with `margin` to spare; one too long for the room starts at the margin. */
const clampInside = (start: number, length: number, limit: number, margin: number): number =>
  Math.max(Math.min(start, limit - margin - length), margin);

/**
 * The top left corner of an overlay of `size` placed against `anchor` in a window of size `window`, and the
 * side it ended up on.
 */
export function place(anchor: Rect, size: Size, window: Size, wanted: Wanted): { x: number; y: number; side: Side } {
  const side = chosenSide(anchor, size, window, wanted);
  let alignment = wanted.alignment;
  if (wanted.flip && wanted.atPoint) {
    const [x, y] = corner(anchor, size, wanted, side, alignment);
    const [start, end, limit] = vertical(side) ? [x, x + size.width, window.width] : [y, y + size.height, window.height];
    if (start < wanted.margin || end > limit - wanted.margin) {
      alignment = OPPOSITE_ALIGNMENT[alignment];
    }
  }
  let [x, y] = corner(anchor, size, wanted, side, alignment);
  if (wanted.shift) {
    x = clampInside(x, size.width, window.width, wanted.margin);
    y = clampInside(y, size.height, window.height, wanted.margin);
  }
  return { x, y, side };
}
