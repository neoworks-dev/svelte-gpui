// The NeoWorks theme as plain data: colours, type scale, radii and shadows for each scope.
// `tokens.generated.ts` holds the values; this file gives them their types and shapes.

import type { Color, Shadow, Style } from "@neoworks/svelte-gpui/ui";
import { tokens } from "./tokens.generated";

export type ThemeName = keyof typeof tokens.themes;
export type ColorRole = keyof typeof tokens.themes.light.colors;
export type ScaleName = keyof typeof tokens.themes.light.scales;
export type ScaleStep = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;
export type TypeRole = keyof typeof tokens.typeScale;
export type RadiusRole = keyof typeof tokens.radii;
export type ShadowRole = keyof typeof tokens.themes.light.shadows;
export type EasingRole = keyof typeof tokens.easing;

/** The twelve steps of a colour. 1 to 2 are page backgrounds, 3 to 5 component fills, 6 to 8 borders, 9 to 10 solid fills, 11 to 12 text. */
export type ColorScale = Record<ScaleStep, Color>;

/**
 * A type role. `fontSize` follows CSS `clamp(minimum, viewportPercent vw, maximum)` in pixels, so a role with
 * `viewportPercent` 0 is fixed. `letterSpacing` is in em; `Style` has no letter spacing yet, so `typeStyle` leaves it out.
 */
export interface TypeRoleTokens {
  fontSize: { minimum: number; viewportPercent: number; maximum: number };
  /** A multiple of the font size. */
  lineHeight: number;
  letterSpacing: number;
  fontWeight: number;
}

export interface Theme {
  name: ThemeName;
  isDark: boolean;
  colors: Record<ColorRole, Color>;
  scales: Record<ScaleName, ColorScale>;
  /** Each shadow is a list of layers, drawn together. */
  shadows: Record<ShadowRole, Shadow[]>;
  radii: Record<RadiusRole, number>;
  typeScale: Record<TypeRole, TypeRoleTokens>;
  /** Cubic bezier control points `[x1, y1, x2, y2]`. `Style.transition` only takes the named easings. */
  easing: Record<EasingRole, number[]>;
}

function assembleTheme(name: ThemeName): Theme {
  const generated = tokens.themes[name];
  return {
    name,
    isDark: generated.colorScheme === "dark",
    colors: generated.colors,
    scales: generated.scales,
    shadows: generated.shadows,
    radii: tokens.radii,
    typeScale: tokens.typeScale,
    easing: tokens.easing,
  };
}

/** `lightInverse` and `darkInverse` are the dark blocks that sit on a light or dark page. */
export const themes: Record<ThemeName, Theme> = {
  light: assembleTheme("light"),
  dark: assembleTheme("dark"),
  lightInverse: assembleTheme("lightInverse"),
  darkInverse: assembleTheme("darkInverse"),
};

/** The theme for a block that reverses the page's colours, such as a dark band on a light page. */
export function inverseTheme(theme: Theme): Theme {
  if (theme.isDark) {
    return themes.darkInverse;
  }
  return themes.lightInverse;
}

/** The window width the fluid type roles are resolved against when none is given. */
export const referenceViewportWidth = 1440;

export function resolveFontSize(role: TypeRole, viewportWidth: number = referenceViewportWidth): number {
  const { minimum, viewportPercent, maximum } = tokens.typeScale[role].fontSize;
  const preferred = (viewportWidth * viewportPercent) / 100;
  return Math.min(maximum, Math.max(minimum, preferred));
}

/** Size, line height and weight of a type role, to spread into a `Text` or `View` style. */
export function typeStyle(role: TypeRole, viewportWidth: number = referenceViewportWidth): Style {
  const fontSize = resolveFontSize(role, viewportWidth);
  const { lineHeight, fontWeight } = tokens.typeScale[role];
  return { fontFamily: "sans", fontSize, lineHeight: fontSize * lineHeight, fontWeight };
}
