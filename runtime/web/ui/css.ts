// A style object as CSS. This is the browser's counterpart of `host/src/style/parse.rs`: it reads
// the same `Style` type, every property in docs/styles.md, and says what each one means in CSS.
// It is a pure function, so what a property comes to can be tested without a page.
//
// The defaults of the host that CSS does not share are in `base.ts`: every element is a flex
// container, positioned, border-box.

import type { Shadow, Sides, Size, Style, Transition } from "../../ui/style";

/** CSS declarations, by property name (`"padding-left"`), in the order to apply them. */
export type Css = Record<string, string>;

/** The faces the host ships. `sans` is its default font. */
export const SANS = `"Geist", ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`;
export const MONO = `"JetBrains Mono", ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas, "DejaVu Sans Mono", monospace`;

const WEIGHTS = { normal: "400", medium: "500", semibold: "600", bold: "700" } as const;

const px = (value: number): string => `${value}px`;

/** A `Size`: pixels, a share of the parent, `"full"` or `"auto"`. */
function length(value: Size): string {
  if (typeof value === "number") {
    return px(value);
  }
  return value === "full" ? "100%" : value;
}

/** One value for every side, two (vertical, horizontal) or four (top, right, bottom, left), as top, right, bottom, left. */
export function sidesOf<T>(value: Sides<T>): [T, T, T, T] {
  if (!Array.isArray(value)) {
    return [value, value, value, value];
  }
  if (value.length === 2) {
    return [value[0], value[1], value[0], value[1]];
  }
  return [value[0], value[1], value[2], value[3]] as [T, T, T, T];
}

const finite = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value);

function shadowCss(shadow: Shadow): string {
  const { x = 0, y = 0, blur = 0, spread = 0, color = "rgba(0, 0, 0, 0.3)" } = shadow;
  return `${px(x)} ${px(y)} ${px(blur)} ${px(spread)} ${color}`;
}

/** The families CSS needs to draw a `fontFamily`: the two the host ships, or an installed font with the sans one behind it. */
export function fontFamilyCss(family: string): string {
  if (family === "sans") {
    return SANS;
  }
  if (family === "mono") {
    return MONO;
  }
  return `${/^[\w-]+$/.test(family) ? family : JSON.stringify(family)}, ${SANS}`;
}

/** The properties a `transition` eases: the ones the host eases. */
const EASED = ["background-color", "color", "border-color", "opacity"];

export function transitionCss(value: number | Transition): string {
  const { duration = 150, delay = 0, easing = "ease" } = typeof value === "number" ? { duration: value } : value;
  const timing = `${duration}ms ${easing}${delay > 0 ? ` ${delay}ms` : ""}`;
  return EASED.map((property) => `${property} ${timing}`).join(", ");
}

/** The CSS a style comes to. A property the style does not set is not in the result. */
export function toCss(style: Style | null | undefined): Css {
  const css: Css = {};
  if (style === null || style === undefined) {
    return css;
  }
  const set = (name: string, value: string | undefined): void => {
    if (value !== undefined) {
      css[name] = value;
    }
  };
  const lengthOf = (value: Size | undefined, none = "auto"): string | undefined =>
    value === undefined ? undefined : value === "auto" ? none : length(value);
  const pixels = (value: number | undefined): string | undefined => (finite(value) ? px(value) : undefined);

  // Layout.
  set("display", style.display);
  set("flex-direction", style.direction);
  if (style.wrap !== undefined) {
    set("flex-wrap", style.wrap ? "wrap" : "nowrap");
  }
  if (finite(style.flex)) {
    // As React Native's `flex: 1`: grow from nothing.
    set("flex", `${style.flex} 1 0%`);
  }
  if (finite(style.grow)) {
    set("flex-grow", String(style.grow));
  }
  if (finite(style.shrink)) {
    set("flex-shrink", String(style.shrink));
  }
  set("flex-basis", lengthOf(style.basis));
  const alignment = { start: "flex-start", end: "flex-end", center: "center", stretch: "stretch", baseline: "baseline" };
  if (style.align !== undefined) {
    set("align-items", alignment[style.align]);
  }
  if (style.alignSelf !== undefined) {
    set("align-self", alignment[style.alignSelf]);
  }
  const justification = { start: "flex-start", center: "center", end: "flex-end", between: "space-between", around: "space-around" };
  if (style.justify !== undefined) {
    set("justify-content", justification[style.justify]);
  }
  set("gap", pixels(style.gap));
  set("row-gap", pixels(style.rowGap));
  set("column-gap", pixels(style.columnGap));

  const edges = ["top", "right", "bottom", "left"] as const;
  if (style.padding !== undefined) {
    sidesOf(style.padding).forEach((value, index) => set(`padding-${edges[index]}`, pixels(value)));
  }
  set("padding-left", pixels(style.paddingX));
  set("padding-right", pixels(style.paddingX));
  set("padding-top", pixels(style.paddingY));
  set("padding-bottom", pixels(style.paddingY));
  set("padding-top", pixels(style.paddingTop));
  set("padding-right", pixels(style.paddingRight));
  set("padding-bottom", pixels(style.paddingBottom));
  set("padding-left", pixels(style.paddingLeft));

  if (style.margin !== undefined) {
    sidesOf(style.margin).forEach((value, index) => set(`margin-${edges[index]}`, lengthOf(value)));
  }
  set("margin-left", lengthOf(style.marginX));
  set("margin-right", lengthOf(style.marginX));
  set("margin-top", lengthOf(style.marginY));
  set("margin-bottom", lengthOf(style.marginY));
  set("margin-top", lengthOf(style.marginTop));
  set("margin-right", lengthOf(style.marginRight));
  set("margin-bottom", lengthOf(style.marginBottom));
  set("margin-left", lengthOf(style.marginLeft));

  // Size and position.
  set("width", lengthOf(style.size));
  set("height", lengthOf(style.size));
  set("width", lengthOf(style.width));
  set("height", lengthOf(style.height));
  set("min-width", lengthOf(style.minWidth));
  set("min-height", lengthOf(style.minHeight));
  // There is no `max-width: auto`: no limit is `none`.
  set("max-width", lengthOf(style.maxWidth, "none"));
  set("max-height", lengthOf(style.maxHeight, "none"));
  if (finite(style.aspectRatio)) {
    set("aspect-ratio", String(style.aspectRatio));
  }
  set("position", style.position);
  if (style.inset !== undefined) {
    sidesOf(style.inset).forEach((value, index) => set(edges[index], lengthOf(value)));
  }
  set("top", lengthOf(style.top));
  set("right", lengthOf(style.right));
  set("bottom", lengthOf(style.bottom));
  set("left", lengthOf(style.left));
  set("overflow", style.overflow);
  if (style.translate !== undefined) {
    set("translate", `${px(style.translate[0])} ${px(style.translate[1])}`);
  }
  if (finite(style.zoom) && style.zoom > 0) {
    set("zoom", String(style.zoom));
  }

  // Paint.
  set("background-color", style.background);
  if (finite(style.opacity)) {
    set("opacity", String(style.opacity));
  }
  const widths: (number | undefined)[] = [undefined, undefined, undefined, undefined];
  if (style.borderWidth !== undefined) {
    sidesOf(style.borderWidth).forEach((value, index) => (widths[index] = value));
  }
  const specific = [style.borderTopWidth, style.borderRightWidth, style.borderBottomWidth, style.borderLeftWidth];
  specific.forEach((value, index) => value !== undefined && (widths[index] = value));
  if (widths.some((value) => value !== undefined)) {
    // The host draws no border where none is set, and a border with no colour is transparent.
    set("border-style", "solid");
    set("border-color", style.borderColor ?? "transparent");
    widths.forEach((value, index) => set(`border-${edges[index]}-width`, pixels(value ?? 0)));
  } else {
    set("border-color", style.borderColor);
  }
  if (style.radius !== undefined) {
    set("border-radius", Array.isArray(style.radius) ? style.radius.map(px).join(" ") : pixels(style.radius));
  }
  if (style.shadow !== undefined) {
    const shadows = style.shadow === null ? [] : Array.isArray(style.shadow) ? style.shadow : [style.shadow];
    set("box-shadow", shadows.length === 0 ? "none" : shadows.map(shadowCss).join(", "));
  }
  set("cursor", style.cursor);
  if (style.transition !== undefined) {
    set("transition", transitionCss(style.transition));
  }

  // Text.
  set("color", style.color);
  set("font-size", pixels(style.fontSize));
  if (style.fontWeight !== undefined) {
    set("font-weight", typeof style.fontWeight === "number" ? String(style.fontWeight) : WEIGHTS[style.fontWeight]);
  }
  if (style.fontFamily !== undefined) {
    set("font-family", fontFamilyCss(style.fontFamily));
  }
  set("line-height", pixels(style.lineHeight));
  if (style.italic !== undefined) {
    set("font-style", style.italic ? "italic" : "normal");
  }
  if (style.underline !== undefined || style.strikethrough !== undefined) {
    const lines = [style.underline ? "underline" : "", style.strikethrough ? "line-through" : ""].filter(Boolean);
    set("text-decoration-line", lines.length === 0 ? "none" : lines.join(" "));
  }
  set("text-align", style.textAlign);
  if (style.nowrap !== undefined) {
    set("white-space", style.nowrap ? "nowrap" : "normal");
  }
  set("overflow-wrap", style.overflowWrap);
  if (style.truncate) {
    set("overflow", "hidden");
    set("text-overflow", "ellipsis");
    set("white-space", "nowrap");
  }
  if (finite(style.maxLines) && style.maxLines > 0) {
    if (style.display !== "none") {
      set("display", "-webkit-box");
    }
    set("-webkit-box-orient", "vertical");
    set("-webkit-line-clamp", String(style.maxLines));
    set("overflow", "hidden");
  }
  if (style.uppercase !== undefined) {
    set("text-transform", style.uppercase ? "uppercase" : "none");
  }
  set("vertical-align", pixels(style.verticalAlign));
  return css;
}

/** Declarations as the text of a `style` attribute. */
export function cssText(css: Css): string {
  return Object.entries(css)
    .map(([property, value]) => `${property}:${value}`)
    .join(";");
}

/**
 * A style, then the states laid over it in the order focus, hover, active (the last to apply wins),
 * as the host does. A state that does not hold passes `undefined`.
 */
export function layered(
  style: Style | undefined,
  states: readonly (Style | undefined | false)[],
): Style | undefined {
  const held = states.filter((state): state is Style => Boolean(state));
  return held.length === 0 ? style : Object.assign({}, style, ...held);
}
