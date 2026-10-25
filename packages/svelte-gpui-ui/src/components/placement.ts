export type PlacementSide = "top" | "bottom" | "left" | "right";
export type PlacementAlign = "start" | "center" | "end";

/** A side, optionally followed by where along it the floating box lines up: `"bottom"`, `"top-end"`. */
export type FloatingPlacement = PlacementSide | `${PlacementSide}-${"start" | "end"}`;

export interface ParsedPlacement {
  side: PlacementSide;
  align: PlacementAlign;
}

/** Splits a placement into the `side` and `align` the overlay primitives take. A bare side is centred. */
export function parsePlacement(placement: FloatingPlacement): ParsedPlacement {
  const [side, alignment] = placement.split("-");
  if (alignment === "start" || alignment === "end") {
    return { side: side as PlacementSide, align: alignment };
  }
  return { side: side as PlacementSide, align: "center" };
}

/** Where a modal layer puts its panel: against an edge, in the middle, or filling the window. */
export type ModalPlacement = "center" | "bottom" | "start" | "end" | "fill";

/** How a modal panel arrives: slid in from an edge, or only faded in, since the host cannot scale. */
export type ModalEntrance = "rise" | "from-start" | "from-end" | "fade";
