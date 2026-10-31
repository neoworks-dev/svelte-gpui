// The style object a primitive takes. It crosses to the host as JSON, which parses
// each distinct string once; host/src/style.rs reads exactly these names.

/** Pixels, a share of the parent (`"50%"`, `"full"`), or `"auto"`. */
export type Size = number | `${number}%` | "full" | "auto";

/** `"#rgb"`, `"#rrggbb"`, `"#rrggbbaa"`, `"rgb(…)"`, `"rgba(…)"` or `"transparent"`. */
export type Color = string;

/** One value for every side, `[vertical, horizontal]`, or `[top, right, bottom, left]`. */
export type Sides<T> = T | [T, T] | [T, T, T, T];

export type Align = "start" | "center" | "end" | "stretch" | "baseline";

export interface Shadow {
  x?: number;
  y?: number;
  blur?: number;
  spread?: number;
  color?: Color;
}

/** How a canvas stroke or line is drawn. */
export interface Pen {
  color?: Color;
  width?: number;
}

export interface Transition {
  duration?: number;
  delay?: number;
  easing?: "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out";
}

export interface Style {
  // Layout. Every primitive is a flex container; views stack their children.
  display?: "flex" | "none";
  direction?: "row" | "column";
  wrap?: boolean;
  /** Grow from nothing to fill what is left, as React Native's `flex: 1`. */
  flex?: number;
  grow?: number;
  shrink?: number;
  basis?: Size;
  align?: Align;
  alignSelf?: Align;
  justify?: "start" | "center" | "end" | "between" | "around";
  gap?: number;
  rowGap?: number;
  columnGap?: number;
  padding?: Sides<number>;
  paddingX?: number;
  paddingY?: number;
  paddingTop?: number;
  paddingRight?: number;
  paddingBottom?: number;
  paddingLeft?: number;
  margin?: Sides<Size>;
  marginX?: Size;
  marginY?: Size;
  marginTop?: Size;
  marginRight?: Size;
  marginBottom?: Size;
  marginLeft?: Size;

  // Size and position.
  width?: Size;
  height?: Size;
  /** Width and height at once. */
  size?: Size;
  minWidth?: Size;
  minHeight?: Size;
  maxWidth?: Size;
  maxHeight?: Size;
  aspectRatio?: number;
  position?: "relative" | "absolute";
  inset?: Sides<Size>;
  top?: Size;
  right?: Size;
  bottom?: Size;
  left?: Size;
  overflow?: "visible" | "hidden";
  /** Moves the element from where layout put it, in pixels: `[x, y]`. */
  translate?: [number, number];
  /**
   * Multiplies every length in the element and everything inside it, as CSS's `zoom` does:
   * sizes, positions, padding, gaps, borders, radii, shadows and text. Zooms inside multiply.
   */
  zoom?: number;

  // Paint.
  background?: Color;
  opacity?: number;
  borderWidth?: Sides<number>;
  borderTopWidth?: number;
  borderRightWidth?: number;
  borderBottomWidth?: number;
  borderLeftWidth?: number;
  borderColor?: Color;
  /** All corners, or `[topLeft, topRight, bottomRight, bottomLeft]`. */
  radius?: number | [number, number, number, number];
  shadow?: Shadow | Shadow[] | null;
  cursor?: "default" | "pointer" | "text" | "grab" | "grabbing" | "crosshair" | "col-resize" | "row-resize" | "nwse-resize" | "nesw-resize" | "not-allowed";
  /** Eases changes of colour and opacity, including hover: a duration in milliseconds, or the details. */
  transition?: number | Transition;

  // Text. Children inherit these.
  color?: Color;
  fontSize?: number;
  fontWeight?: number | "normal" | "medium" | "semibold" | "bold";
  /** `"sans"`, `"mono"`, or the name of an installed font. */
  fontFamily?: string;
  /** In pixels. */
  lineHeight?: number;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  textAlign?: "left" | "center" | "right";
  nowrap?: boolean;
  /**
   * `"anywhere"` breaks a word longer than the line between letters, as CSS does, so a long URL
   * or path wraps inside its box instead of widening it. By default (`"normal"`) a text is never
   * narrower than its longest word.
   */
  overflowWrap?: "normal" | "anywhere";
  /** One line, cut off with an ellipsis. */
  truncate?: boolean;
  maxLines?: number;
  uppercase?: boolean;
  /**
   * For an element inside a `Text`, which sits on the line's baseline: how far up it moves, in
   * pixels, as in CSS. A negative value lowers it, so a formula's depth hangs below the line.
   */
  verticalAlign?: number;
}

/** A style as the host reads it, or nothing. */
export function encode(style: Style | null | undefined): string | undefined {
  if (style === null || style === undefined) {
    return undefined;
  }
  return JSON.stringify(style);
}

/** Lays styles over each other, later ones winning, skipping the ones that are off. */
export function merge(...styles: (Style | null | undefined | false)[]): Style {
  const out: Style = {};
  for (const style of styles) {
    if (style) {
      Object.assign(out, style);
    }
  }
  return out;
}
