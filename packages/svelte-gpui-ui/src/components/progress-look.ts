import type { Color, Style } from "@neoworks/svelte-gpui/ui";
import type { Theme } from "../theme";
import { withAlpha } from "./colorMix";
import { ringStrokeWidth } from "./progressModel";

export type ProgressTone = "default" | "accent" | "inherit";

export const linearTrackHeight = 4;

/** The web draws the circular track as the indicator colour at 15% opacity. */
const ringTrackOpacity = 0.15;

/** The colour of the filled part. `inherit` follows `color`, which stands in for the surrounding text colour. */
export function progressColor(theme: Theme, tone: ProgressTone, variant: "linear" | "circular", color: Color | undefined): Color {
  if (tone === "accent") {
    return theme.colors.accent;
  }
  if (tone === "inherit" && color !== undefined) {
    return color;
  }
  if (variant === "linear" && tone === "default") {
    return theme.colors.fill;
  }
  return theme.colors.text;
}

export function linearTrackLook(theme: Theme): Style {
  return {
    position: "relative",
    width: "full",
    height: linearTrackHeight,
    radius: linearTrackHeight / 2,
    overflow: "hidden",
    background: theme.colors.pressed,
    shrink: 0,
  };
}

export function linearBarLook(color: Color): Style {
  return { height: "full", radius: linearTrackHeight / 2, background: color };
}

/** The ring behind the arc, drawn as a bordered circle since the canvas has no stroked arcs. */
export function ringTrackLook(size: number, thickness: number, color: Color): Style {
  return {
    position: "absolute",
    left: 0,
    top: 0,
    size,
    radius: size / 2,
    borderWidth: ringStrokeWidth({ size, thickness }),
    borderColor: withAlpha(color, ringTrackOpacity),
  };
}

export function ringLook(size: number): Style {
  return { position: "relative", size, shrink: 0 };
}
