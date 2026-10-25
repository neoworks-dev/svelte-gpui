import { describe, expect, test } from "bun:test";
import { gamutOverflow, mixOklch, parseOklch, toCssColor, transparentColor } from "../packages/svelte-gpui-ui/scripts/color";
import { evaluateColor, generateTokens, parseBlocks, splitOutsideParentheses } from "../packages/svelte-gpui-ui/scripts/tokens";
import chromeColors from "./fixtures/chrome-oklch.json";

// Channels of a "#rrggbb" string.
const channels = (hex: string): number[] => [1, 3, 5].map((start) => parseInt(hex.slice(start, start + 2), 16));

describe("oklch to hex", () => {
  test("the sRGB primaries and the extremes come out exact", () => {
    expect(toCssColor(parseOklch("oklch(1 0 0)"))).toBe("#ffffff");
    expect(toCssColor(parseOklch("oklch(0 0 0)"))).toBe("#000000");
    expect(toCssColor(parseOklch("oklch(0.62796 0.25768 29.234)"))).toBe("#ff0000");
    expect(toCssColor(parseOklch("oklch(0.86644 0.29483 142.495)"))).toBe("#00ff00");
    expect(toCssColor(parseOklch("oklch(0.45201 0.31321 264.052)"))).toBe("#0000ff");
  });

  // Chromium's canvas resolved each of these; its 8-bit pipeline lands within one step of exact arithmetic.
  test("every colour in tokens.css is within one step of what Chromium draws", () => {
    for (const [text, expected] of Object.entries(chromeColors)) {
      const actual = channels(toCssColor(parseOklch(text)));
      actual.forEach((channel, index) => expect(Math.abs(channel - expected[index])).toBeLessThanOrEqual(1));
    }
  });

  test("alpha comes out as rgba()", () => {
    expect(toCssColor(parseOklch("oklch(0 0 0 / 0.04)"))).toBe("rgba(0, 0, 0, 0.04)");
    expect(toCssColor(parseOklch("oklch(1 0 0 / 0.14)"))).toBe("rgba(255, 255, 255, 0.14)");
  });

  test("a colour outside sRGB is clipped, and reported", () => {
    const larchText = parseOklch("oklch(0.470 0.122 72)");
    expect(gamutOverflow(larchText)).toBeGreaterThan(30);
    expect(toCssColor(larchText)).toBe("#844c00");
    expect(gamutOverflow(parseOklch("oklch(0.5 0.05 100)"))).toBe(0);
  });
});

describe("color-mix", () => {
  test("mixes in oklch the way Chromium does", () => {
    const dark = parseOklch("oklch(0.205 0.004 100)");
    const light = parseOklch("oklch(0.993 0.0015 100)");
    expect(toCssColor(mixOklch(dark, 0.86, light))).toBe("#323230");
  });

  test("mixing with transparent keeps the colour and scales its alpha", () => {
    const line = parseOklch("oklch(0.855 0.004 100)");
    expect(toCssColor(mixOklch(line, 0.45, transparentColor))).toBe("rgba(208, 207, 204, 0.45)");
  });

  test("hues take the shorter way round", () => {
    const mixed = mixOklch(parseOklch("oklch(0.5 0.1 350)"), 0.5, parseOklch("oklch(0.5 0.1 10)"));
    expect(mixed.hue).toBe(0);
  });
});

describe("tokens.css", () => {
  const stylesheet = `
    @theme { --text-body: 1.0625rem; --text-body--line-height: 1.6; --radius-card: 1rem;
      --shadow-overlay: 0 0 0 1px color-mix(in oklch, var(--line) 45%, transparent), 0 16px 40px -16px oklch(0 0 0 / 0.16); }
    @theme inline { --color-line: var(--line); --color-fill: var(--fill); }
    :root { --grey-1: oklch(0.993 0.0015 100); --grey-6: oklch(0.855 0.004 100); --grey-12: oklch(0.205 0.004 100);
      --line: var(--grey-6); --fill: var(--grey-12); color-scheme: light; }
    .scope-dark { --grey-6: oklch(0.330 0.004 100); color-scheme: dark; }
  `;

  test("splits at the separators outside parentheses only", () => {
    expect(splitOutsideParentheses("0 1px color-mix(in oklch, a 45%, b), 2px", ",")).toEqual([
      "0 1px color-mix(in oklch, a 45%, b)",
      "2px",
    ]);
  });

  test("reads blocks with their selectors", () => {
    const blocks = parseBlocks("/* note { */ :root,\n.scope-dark { --a: 1; --b: 2 }");
    expect(blocks).toEqual([{ selectors: [":root", ".scope-dark"], declarations: [["--a", "1"], ["--b", "2"]] }]);
  });

  test("resolves a role against the scope's own scale", () => {
    const scope = new Map([["--line", "var(--grey-6)"], ["--grey-6", "oklch(0.330 0.004 100)"]]);
    expect(toCssColor(evaluateColor("var(--line)", scope))).toBe("#363533");
  });

  test("the dark scope redefines the scale its roles point at", () => {
    const module = generateTokens(stylesheet).module;
    expect(module).toContain('line: "#d0cfcc"');
    expect(module).toContain('line: "#363533"');
  });

  test("shadows are resolved per scope, with their spread", () => {
    const module = generateTokens(stylesheet).module;
    expect(module).toContain('{ x: 0, y: 0, blur: 0, spread: 1, color: "rgba(208, 207, 204, 0.45)" }');
    expect(module).toContain('{ x: 0, y: 16, blur: 40, spread: -16, color: "rgba(0, 0, 0, 0.16)" }');
  });

  test("type sizes become pixels, and unset tracking and weight take CSS's defaults", () => {
    const module = generateTokens(stylesheet).module;
    expect(module).toContain("fontSize: { minimum: 17, viewportPercent: 0, maximum: 17 }");
    expect(module).toContain("letterSpacing: 0");
    expect(module).toContain("fontWeight: 400");
    expect(module).toContain("radii: { card: 16 }");
  });
});
