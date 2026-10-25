import { describe, expect, test } from "bun:test";
import { badgeCountText, badgeLook, isBadgeVisible } from "../packages/svelte-gpui-ui/src/components/badge-look";
import { buttonLook } from "../packages/svelte-gpui-ui/src/components/button-look";
import { cardLook } from "../packages/svelte-gpui-ui/src/components/card-look";
import { chipLook, chipRemoveLook } from "../packages/svelte-gpui-ui/src/components/chip-look";
import { dividerLook } from "../packages/svelte-gpui-ui/src/components/divider-look";
import { iconButtonLook } from "../packages/svelte-gpui-ui/src/components/icon-button-look";
import { listRowLook } from "../packages/svelte-gpui-ui/src/components/list-row-look";
import {
  blendColors,
  focusStyle,
  visibleShadows,
  wrapperAlignment,
} from "../packages/svelte-gpui-ui/src/components/look";
import { themes, typeStyle } from "../packages/svelte-gpui-ui/src/theme";

const light = themes.light;
const dark = themes.dark;

describe("blendColors", () => {
  test("lays the overlay over the base at the given share", () => {
    expect(blendColors("#000000", "#ffffff", 0.5)).toBe("#808080");
    expect(blendColors("#102030", "#102030", 0.4)).toBe("#102030");
    expect(blendColors("#ffffff", "#000000", 0.04)).toBe("#f5f5f5");
  });

  test("refuses a colour it cannot read, so a wrong state layer is not drawn silently", () => {
    expect(() => blendColors("rgba(0, 0, 0, 0.5)", "#ffffff", 0.5)).toThrow();
  });
});

describe("shadows the host can draw", () => {
  test("a blur-0 layer gets the smallest visible blur and others stay as they are", () => {
    const [outline, drop] = visibleShadows(light.shadows.overlay);
    expect(outline.blur).toBe(1);
    expect(outline.spread).toBe(1);
    expect(drop.blur).toBe(40);
  });

  test("a focus ring is only drawn behind a box that has a background", () => {
    expect(focusStyle(light, { hasBackground: true, hasBorder: false }).shadow).toHaveLength(1);
    expect(focusStyle(light, { hasBackground: false, hasBorder: true }).shadow).toBeUndefined();
    expect(focusStyle(light, { hasBackground: false, hasBorder: true }).borderColor).toBe(light.colors.focus);
    expect(focusStyle(light, { hasBackground: false, hasBorder: false }).background).toBe(light.colors.hover);
  });

  test("a wrapper centres itself unless told otherwise", () => {
    expect(wrapperAlignment(undefined)).toEqual({ alignSelf: "center" });
    expect(wrapperAlignment({ alignSelf: "start" })).toEqual({ alignSelf: "start" });
  });
});

describe("buttonLook", () => {
  const options = { variant: "primary", size: "md", full: false, round: false } as const;

  test("a primary button is the fill colour in either theme", () => {
    expect(buttonLook(light, options).style.background).toBe(light.colors.fill);
    expect(buttonLook(dark, options).style.background).toBe(dark.colors.fill);
    expect(buttonLook(light, options).hover?.background).toBe(light.colors.fillHover);
  });

  test("sizes are 32, 40 and 48 pixels high and fully rounded", () => {
    const heights = (["sm", "md", "lg"] as const).map((size) => buttonLook(light, { ...options, size }).style);
    expect(heights.map((style) => style.height)).toEqual([32, 40, 48]);
    expect(heights.map((style) => style.radius)).toEqual([16, 20, 24]);
  });

  test("round makes a circle and full takes the parent's width", () => {
    const round = buttonLook(light, { ...options, round: true }).style;
    expect([round.width, round.height]).toEqual([40, 40]);
    expect(buttonLook(light, { ...options, full: true }).style.width).toBe("full");
    expect(buttonLook(light, options).style.width).toBeUndefined();
  });

  test("a secondary button has a border and no background, and focus recolours the border", () => {
    const look = buttonLook(light, { ...options, variant: "secondary" });
    expect(look.style.background).toBeUndefined();
    expect(look.style.borderColor).toBe(light.colors.lineStrong);
    expect(look.focus?.borderColor).toBe(light.colors.focus);
  });

  test("a disabled button dims to 40 percent", () => {
    expect(buttonLook(light, options).disabledStyle?.opacity).toBe(0.4);
  });

  test("text takes its size and weight from the type scale", () => {
    expect(buttonLook(light, options).style.fontSize).toBe(typeStyle("body").fontSize);
    expect(buttonLook(light, options).style.fontWeight).toBe(500);
  });
});

describe("iconButtonLook", () => {
  const options = { variant: "filled", size: "md", selected: undefined } as const;

  test("a filled toggle that is off is quiet, and on is solid", () => {
    expect(iconButtonLook(light, { ...options, selected: false }).style.background).toBe(light.colors.sunken);
    expect(iconButtonLook(light, { ...options, selected: true }).style.background).toBe(light.colors.fill);
    expect(iconButtonLook(light, options).style.background).toBe(light.colors.fill);
  });

  test("an outlined toggle fills when on", () => {
    const outlined = { ...options, variant: "outlined" } as const;
    expect(iconButtonLook(light, outlined).style.borderColor).toBe(light.colors.lineStrong);
    expect(iconButtonLook(light, { ...outlined, selected: true }).style.background).toBe(light.colors.fill);
  });

  test("a standard button is muted until selected or hovered", () => {
    const standard = { ...options, variant: "standard" } as const;
    expect(iconButtonLook(light, standard).style.color).toBe(light.colors.muted);
    expect(iconButtonLook(light, { ...standard, selected: true }).style.color).toBe(light.colors.text);
    expect(iconButtonLook(light, standard).hover?.color).toBe(light.colors.text);
  });
});

describe("chipLook", () => {
  const options = { applied: false, elevated: false, hasLeading: false, hasTrailingIcon: false, removable: false };

  test("padding depends on what is beside the label", () => {
    expect(chipLook(light, options).style.paddingLeft).toBe(14);
    expect(chipLook(light, { ...options, hasLeading: true }).style.paddingLeft).toBe(12);
    expect(chipLook(light, { ...options, hasTrailingIcon: true }).style.paddingRight).toBe(12);
    expect(chipLook(light, { ...options, removable: true }).style.paddingRight).toBe(8);
  });

  test("an applied filter is the fill colour with no visible border", () => {
    const style = chipLook(light, { ...options, applied: true }).style;
    expect(style.background).toBe(light.colors.fill);
    expect(style.borderColor).toBe("transparent");
  });

  test("an elevated chip sits on the lift shadow, and keeps it when focused", () => {
    const look = chipLook(light, { ...options, elevated: true });
    expect(look.style.shadow).toEqual(light.shadows.lift);
    expect(look.focus?.shadow).toHaveLength(light.shadows.lift.length + 1);
  });

  test("a removable chip and its remove button join into one pill", () => {
    const main = chipLook(light, { ...options, removable: true }).style;
    const remove = chipRemoveLook(light).style;
    expect(main.radius).toEqual([16, 0, 0, 16]);
    expect(main.borderRightWidth).toBe(0);
    expect(remove.radius).toEqual([0, 16, 16, 0]);
    expect(remove.borderWidth).toEqual([1, 1, 1, 0]);
  });
});

describe("badge", () => {
  test("a count shows only above zero, a dot always", () => {
    expect(isBadgeVisible(undefined, false)).toBe(false);
    expect(isBadgeVisible(0, false)).toBe(false);
    expect(isBadgeVisible(1, false)).toBe(true);
    expect(isBadgeVisible(0, true)).toBe(true);
  });

  test("a count above the maximum reads as the maximum and a plus", () => {
    expect(badgeCountText(99, 99)).toBe("99");
    expect(badgeCountText(100, 99)).toBe("99+");
    expect(badgeCountText(12, 9)).toBe("9+");
    expect(badgeCountText(undefined, 99)).toBe("");
  });

  test("tones use the design system's colours and an attached badge is pinned to a corner", () => {
    const options = { tone: "danger", dot: false, attached: false } as const;
    expect(badgeLook(light, options).background).toBe(light.colors.rock);
    expect(badgeLook(light, { ...options, tone: "neutral" }).background).toBe(light.colors.fill);
    expect(badgeLook(light, options).position).toBeUndefined();
    expect(badgeLook(light, { ...options, attached: true })).toMatchObject({ position: "absolute", top: -6, right: -8 });
    expect(badgeLook(light, { ...options, attached: true, dot: true })).toMatchObject({ top: 0, right: 0, size: 10 });
  });
});

describe("dividerLook", () => {
  test("a horizontal line is one pixel high and a vertical one one pixel wide", () => {
    expect(dividerLook(light, { orientation: "horizontal", inset: "none", strong: false })).toMatchObject({ height: 1 });
    expect(dividerLook(light, { orientation: "vertical", inset: "none", strong: false })).toMatchObject({ width: 1, alignSelf: "stretch" });
  });

  test("strong uses the stronger line colour and insets indent as the web does", () => {
    expect(dividerLook(light, { orientation: "horizontal", inset: "none", strong: true }).background).toBe(light.colors.lineStrong);
    expect(dividerLook(light, { orientation: "horizontal", inset: "start", strong: false }).marginLeft).toBe(64);
    expect(dividerLook(light, { orientation: "horizontal", inset: "both", strong: false }).marginX).toBe(16);
    expect(dividerLook(light, { orientation: "vertical", inset: "both", strong: false }).marginY).toBe(16);
  });
});

describe("cardLook", () => {
  test("each variant has its own surface", () => {
    expect(cardLook(light, { variant: "outlined", interactive: false }).style).toMatchObject({
      background: light.colors.surface,
      borderColor: light.colors.line,
    });
    expect(cardLook(light, { variant: "filled", interactive: false }).style.background).toBe(light.colors.sunken);
    expect(cardLook(light, { variant: "elevated", interactive: false }).style.shadow).toEqual(light.shadows.lift);
  });

  test("only a card that can be pressed has states, washed with the text colour", () => {
    expect(cardLook(light, { variant: "outlined", interactive: false }).hover).toBeUndefined();
    const look = cardLook(light, { variant: "outlined", interactive: true });
    expect(look.hover?.background).toBe(blendColors(light.colors.surface, light.colors.text, 0.04));
    expect(look.active?.background).toBe(blendColors(light.colors.surface, light.colors.text, 0.08));
  });

  test("an elevated card that can be pressed rises to the overlay shadow on hover", () => {
    const look = cardLook(light, { variant: "elevated", interactive: true });
    expect(look.hover?.shadow).toEqual(visibleShadows(light.shadows.overlay));
  });
});

describe("listRowLook", () => {
  test("a row is 56 pixels tall at least, and only a pressable one washes", () => {
    expect(listRowLook(light, false).style.minHeight).toBe(56);
    expect(listRowLook(light, false).hover).toBeUndefined();
    expect(listRowLook(light, true).hover?.background).toBe(blendColors(light.colors.surface, light.colors.text, 0.04));
  });
});
