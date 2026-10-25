import type { Color, Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { ring } from "./look";

export type BadgeTone = "danger" | "neutral" | "accent";

export const defaultBadgeMaximum = 99;

/** Whether a badge shows at all: a dot always does, a count only when it is above zero. */
export function isBadgeVisible(count: number | undefined, dot: boolean): boolean {
  if (dot) {
    return true;
  }
  return count !== undefined && count > 0;
}

/** The count's text, `"99+"` once it is above `maximum`. */
export function badgeCountText(count: number | undefined, maximum: number): string {
  if (count === undefined) {
    return "";
  }
  if (count > maximum) {
    return `${maximum}+`;
  }
  return String(count);
}

function toneColors(theme: Theme, tone: BadgeTone): { background: Color; color: Color } {
  const { colors } = theme;
  if (tone === "neutral") {
    return { background: colors.fill, color: colors.onFill };
  }
  if (tone === "accent") {
    return { background: colors.accent, color: colors.onAccent };
  }
  return { background: colors.rock, color: colors.onAccent };
}

export interface BadgeOptions {
  tone: BadgeTone;
  dot: boolean;
  /** Pinned to the corner of what it labels, instead of standing alone. */
  attached: boolean;
}

function cornerStyle(options: BadgeOptions): Style {
  if (!options.attached) {
    return {};
  }
  if (options.dot) {
    return { position: "absolute", top: 0, right: 0 };
  }
  return { position: "absolute", top: -6, right: -8 };
}

function sizeStyle(dot: boolean): Style {
  if (dot) {
    return { size: 10, radius: 5 };
  }
  return { height: 16, minWidth: 16, paddingX: 4, radius: 8 };
}

/** The badge's own box. The ring in the page colour keeps it readable where it overlaps an icon. */
export function badgeLook(theme: Theme, options: BadgeOptions): Style {
  const colors = toneColors(theme, options.tone);
  return {
    direction: "row",
    align: "center",
    justify: "center",
    ...colors,
    shadow: ring(2, theme.colors.canvas),
    ...sizeStyle(options.dot),
    ...cornerStyle(options),
  };
}

export function badgeTextLook(): Style {
  return { ...typeStyle("label"), nowrap: true };
}
