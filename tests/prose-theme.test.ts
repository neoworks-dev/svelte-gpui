import { describe, expect, test } from "bun:test";
import { proseTheme } from "../packages/svelte-gpui-editor/src/themes";

const colors = { text: "#e4e4e7", heading: "#fafafa", muted: "#71717a", link: "#60a5fa", code: "#fbbf24" };

describe("proseTheme", () => {
  test("draws the text in the sans-serif and headings smaller by level", () => {
    const theme = proseTheme(colors);
    expect(theme.fontFamily).toBe("Geist");
    const size = (name: string) => {
      const style = theme.syntax?.[name];
      if (typeof style === "string") {
        return undefined;
      }
      return style?.size;
    };
    expect(size("title.1")).toBeGreaterThan(size("title.2") ?? 0);
    expect(size("title.2")).toBeGreaterThan(size("title.3") ?? 0);
  });

  test("draws code, and what a code block's language highlights in it, in monospace", () => {
    const syntax = proseTheme(colors).syntax ?? {};
    for (const name of ["text.literal", "embedded", "keyword", "string", "punctuation.bracket"]) {
      expect(syntax[name]).toEqual({ color: colors.code, family: "JetBrainsMono Nerd Font Mono" });
    }
  });
});
