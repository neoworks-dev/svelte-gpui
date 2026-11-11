import { describe, expect, test } from "bun:test";
import {
  cssText,
  fontFamilyCss,
  layered,
  MONO,
  PROPERTIES,
  SANS,
  sidesOf,
  toCss,
  transitionCss,
} from "../../runtime/web/ui/css";
import type { Style } from "../../runtime/ui/style";

/** One value for every property of a style. Typed as `Required`, so a property added to `Style` fails to compile here. */
const everything: Required<Style> = {
  display: "none",
  direction: "row",
  wrap: true,
  flex: 1,
  grow: 2,
  shrink: 0,
  basis: "50%",
  align: "center",
  alignSelf: "end",
  justify: "between",
  gap: 4,
  rowGap: 5,
  columnGap: 6,
  padding: [1, 2],
  paddingX: 3,
  paddingY: 4,
  paddingTop: 5,
  paddingRight: 6,
  paddingBottom: 7,
  paddingLeft: 8,
  margin: [1, 2, 3, 4],
  marginX: "auto",
  marginY: 2,
  marginTop: 3,
  marginRight: 4,
  marginBottom: 5,
  marginLeft: 6,
  width: 100,
  height: "full",
  size: 10,
  minWidth: 1,
  minHeight: 2,
  maxWidth: 3,
  maxHeight: 4,
  aspectRatio: 1.5,
  position: "absolute",
  inset: 0,
  top: 1,
  right: 2,
  bottom: 3,
  left: 4,
  overflow: "hidden",
  translate: [3, -4],
  zoom: 2,
  background: "#112233",
  opacity: 0.5,
  borderWidth: 2,
  borderTopWidth: 1,
  borderRightWidth: 3,
  borderBottomWidth: 4,
  borderLeftWidth: 5,
  borderColor: "#fff",
  radius: [1, 2, 3, 4],
  shadow: { x: 1, y: 2, blur: 3, spread: 4, color: "#000" },
  cursor: "pointer",
  transition: 120,
  color: "#abc",
  fontSize: 16,
  fontWeight: "bold",
  fontFamily: "mono",
  lineHeight: 20,
  italic: true,
  underline: true,
  strikethrough: true,
  textAlign: "center",
  nowrap: true,
  overflowWrap: "anywhere",
  truncate: true,
  maxLines: 3,
  uppercase: true,
  verticalAlign: -5,
};

describe("toCss", () => {
  test("every property of a style says something in CSS", () => {
    expect(Object.keys(everything).sort()).toEqual(Object.keys(PROPERTIES).sort());
    for (const property of Object.keys(PROPERTIES) as (keyof Style)[]) {
      expect(Object.keys(toCss({ [property]: everything[property] })).length, property).toBeGreaterThan(0);
    }
  });

  test("an empty style, or none, comes to nothing", () => {
    expect(toCss({})).toEqual({});
    expect(toCss(undefined)).toEqual({});
    expect(toCss(null)).toEqual({});
  });

  test("sizes are pixels, shares, `full` and `auto`", () => {
    expect(toCss({ width: 120, height: "50%", minWidth: "full", marginLeft: "auto" })).toMatchObject({
      width: "120px",
      height: "50%",
      "min-width": "100%",
      "margin-left": "auto",
    });
    // There is no `max-width: auto`: no limit is `none`.
    expect(toCss({ maxWidth: "auto", maxHeight: 40 })).toMatchObject({ "max-width": "none", "max-height": "40px" });
  });

  test("`size` is both, and a later width wins", () => {
    expect(toCss({ size: 24 })).toMatchObject({ width: "24px", height: "24px" });
    expect(toCss({ size: 24, width: 50 })).toMatchObject({ width: "50px", height: "24px" });
  });

  test("sides take one, two or four values, and the specific ones override", () => {
    expect(sidesOf(4)).toEqual([4, 4, 4, 4]);
    expect(sidesOf([1, 2])).toEqual([1, 2, 1, 2]);
    expect(sidesOf([1, 2, 3, 4])).toEqual([1, 2, 3, 4]);
    expect(toCss({ padding: [6, 12], paddingLeft: 1 })).toMatchObject({
      "padding-top": "6px",
      "padding-right": "12px",
      "padding-bottom": "6px",
      "padding-left": "1px",
    });
    expect(toCss({ margin: "auto", marginX: 2, marginTop: 9 })).toMatchObject({
      "margin-top": "9px",
      "margin-right": "2px",
      "margin-bottom": "auto",
      "margin-left": "2px",
    });
    expect(toCss({ paddingX: 5, paddingY: 7 })).toMatchObject({
      "padding-left": "5px",
      "padding-right": "5px",
      "padding-top": "7px",
      "padding-bottom": "7px",
    });
    expect(toCss({ inset: [1, 2], left: 9 })).toMatchObject({ top: "1px", right: "2px", bottom: "1px", left: "9px" });
  });

  test("`flex` grows from nothing, and a flexible child may shrink below its content", () => {
    expect(toCss({ flex: 1 })).toMatchObject({ flex: "1 1 0%", "min-width": "0", "min-height": "0" });
    expect(toCss({ grow: 1 })).toMatchObject({ "flex-grow": "1", "min-width": "0" });
    expect(toCss({ flex: 1, minWidth: 120 })).toMatchObject({ "min-width": "120px", "min-height": "0" });
    expect(toCss({ shrink: 0, basis: 80 })).toEqual({ "flex-shrink": "0", "flex-basis": "80px" });
  });

  test("alignment names become flexbox's", () => {
    expect(toCss({ align: "start", alignSelf: "end", justify: "between" })).toEqual({
      "align-items": "flex-start",
      "align-self": "flex-end",
      "justify-content": "space-between",
    });
    expect(toCss({ justify: "around", align: "baseline", wrap: false })).toMatchObject({
      "justify-content": "space-around",
      "align-items": "baseline",
      "flex-wrap": "nowrap",
    });
  });

  test("a border needs a style and, without a colour, is transparent, as in the host", () => {
    expect(toCss({ borderWidth: 1 })).toMatchObject({
      "border-style": "solid",
      "border-color": "transparent",
      "border-top-width": "1px",
      "border-left-width": "1px",
    });
    expect(toCss({ borderWidth: 1, borderBottomWidth: 3, borderColor: "#fff" })).toMatchObject({
      "border-color": "#fff",
      "border-bottom-width": "3px",
      "border-top-width": "1px",
    });
    // A side alone leaves the others at nothing.
    expect(toCss({ borderLeftWidth: 2 })).toMatchObject({ "border-left-width": "2px", "border-top-width": "0px" });
  });

  test("radius, shadows, translate and zoom", () => {
    expect(toCss({ radius: 6 })["border-radius"]).toBe("6px");
    expect(toCss({ radius: [1, 2, 3, 4] })["border-radius"]).toBe("1px 2px 3px 4px");
    expect(toCss({ shadow: { y: 8, blur: 24, color: "#00000099" } })["box-shadow"]).toBe("0px 8px 24px 0px #00000099");
    expect(toCss({ shadow: {} })["box-shadow"]).toBe("0px 0px 0px 0px rgba(0, 0, 0, 0.3)");
    expect(toCss({ shadow: [{ x: 1 }, { y: 2, spread: 3, color: "red" }] })["box-shadow"]).toBe(
      "1px 0px 0px 0px rgba(0, 0, 0, 0.3), 0px 2px 0px 3px red",
    );
    expect(toCss({ shadow: null })["box-shadow"]).toBe("none");
    expect(toCss({ translate: [3, -4] }).translate).toBe("3px -4px");
    expect(toCss({ zoom: 1.5 }).zoom).toBe("1.5");
    // A zoom that is not a positive number means nothing, as in the host.
    expect(toCss({ zoom: 0 })).toEqual({});
  });

  test("a transition eases the colours and the opacity", () => {
    expect(transitionCss(150)).toBe(
      "background-color 150ms ease, color 150ms ease, border-color 150ms ease, opacity 150ms ease",
    );
    expect(transitionCss({ duration: 200, delay: 50, easing: "ease-out" })).toBe(
      "background-color 200ms ease-out 50ms, color 200ms ease-out 50ms, border-color 200ms ease-out 50ms, opacity 200ms ease-out 50ms",
    );
    expect(transitionCss({})).toContain("150ms ease");
    expect(toCss({ transition: 90 }).transition).toContain("opacity 90ms");
  });

  test("text properties", () => {
    expect(toCss({ fontWeight: "semibold" })["font-weight"]).toBe("600");
    expect(toCss({ fontWeight: 300 })["font-weight"]).toBe("300");
    expect(toCss({ fontSize: 13, lineHeight: 18, textAlign: "right", uppercase: true })).toEqual({
      "font-size": "13px",
      "line-height": "18px",
      "text-align": "right",
      "text-transform": "uppercase",
    });
    expect(toCss({ italic: true, underline: true, strikethrough: true })).toEqual({
      "font-style": "italic",
      "text-decoration-line": "underline line-through",
    });
    expect(toCss({ underline: false })["text-decoration-line"]).toBe("none");
    expect(toCss({ verticalAlign: -5 })["vertical-align"]).toBe("-5px");
    expect(toCss({ overflowWrap: "anywhere" })["overflow-wrap"]).toBe("anywhere");
    expect(toCss({ nowrap: true })["white-space"]).toBe("nowrap");
  });

  test("fonts: the host's two, and an installed one with the sans stack behind it", () => {
    expect(SANS.startsWith('"Geist"')).toBe(true);
    expect(MONO.startsWith('"JetBrains Mono"')).toBe(true);
    expect(fontFamilyCss("sans")).toBe(SANS);
    expect(fontFamilyCss("mono")).toBe(MONO);
    expect(fontFamilyCss("Inter")).toBe(`Inter, ${SANS}`);
    expect(fontFamilyCss("Source Serif 4")).toBe(`"Source Serif 4", ${SANS}`);
  });

  test("truncate is one line with an ellipsis, and maxLines clamps", () => {
    expect(toCss({ truncate: true })).toEqual({ overflow: "hidden", "text-overflow": "ellipsis", "white-space": "nowrap" });
    expect(toCss({ maxLines: 2 })).toEqual({
      display: "-webkit-box",
      "-webkit-box-orient": "vertical",
      "-webkit-line-clamp": "2",
      overflow: "hidden",
    });
    expect(toCss({ maxLines: 2, display: "none" }).display).toBe("none");
  });

  test("`display: none` and the rest of the layout pass through", () => {
    expect(toCss({ display: "none", direction: "row", position: "absolute", overflow: "hidden", cursor: "col-resize" })).toEqual({
      display: "none",
      "flex-direction": "row",
      position: "absolute",
      overflow: "hidden",
      cursor: "col-resize",
    });
    expect(toCss({ aspectRatio: 1.5, opacity: 0.25, background: "#112233" })).toEqual({
      "aspect-ratio": "1.5",
      opacity: "0.25",
      "background-color": "#112233",
    });
  });

  test("numbers that are not numbers are left out", () => {
    expect(toCss({ gap: Number.NaN, fontSize: Infinity, flex: Number.NaN })).toEqual({});
  });

  test("css text is the declarations joined", () => {
    expect(cssText({ color: "red", "font-size": "12px" })).toBe("color:red;font-size:12px");
    expect(cssText({})).toBe("");
  });
});

describe("states over a style", () => {
  const base: Style = { background: "#111", color: "#eee" };

  test("they lay over it, the last one winning, and only the ones that hold", () => {
    expect(layered(base, [])).toBe(base);
    expect(layered(base, [false, undefined])).toBe(base);
    expect(layered(base, [{ background: "#222" }, false])).toEqual({ background: "#222", color: "#eee" });
    expect(layered(base, [{ background: "#222" }, { background: "#333" }])).toEqual({ background: "#333", color: "#eee" });
    expect(layered(undefined, [{ color: "red" }])).toEqual({ color: "red" });
  });

  test("hover over a style that has a transition keeps it", () => {
    const css = toCss(layered({ background: "#111", transition: 120 }, [{ background: "#222" }]));
    expect(css["background-color"]).toBe("#222");
    expect(css.transition).toContain("120ms");
  });
});
