import type { Color, Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { borderStyle, focusStyle, pillRadius, pressedSolidOpacity, stateTransition, visibleShadows, type StateStyles } from "./look";

export type FabVariant = "primary" | "surface" | "accent";
export type FabSize = "sm" | "md" | "lg";

const heights: Record<FabSize, number> = { sm: 40, md: 56, lg: 96 };

export const fabIconSizes: Record<FabSize, number> = { sm: 20, md: 24, lg: 36 };

interface VariantColors {
  background: Color;
  color: Color;
  borderColor?: Color;
  hover: Style;
  active: Style;
}

function variantColors(theme: Theme, variant: FabVariant): VariantColors {
  const { colors } = theme;
  if (variant === "surface") {
    return {
      background: colors.surface,
      color: colors.text,
      borderColor: colors.line,
      hover: { background: colors.hover },
      active: { background: colors.pressed },
    };
  }
  if (variant === "accent") {
    return {
      background: colors.accent,
      color: colors.onAccent,
      hover: { background: colors.accentHover },
      active: { background: colors.accentHover, opacity: pressedSolidOpacity },
    };
  }
  return {
    background: colors.fill,
    color: colors.onFill,
    hover: { background: colors.fillHover },
    active: { background: colors.fillHover, opacity: pressedSolidOpacity },
  };
}

export interface FabOptions {
  variant: FabVariant;
  size: FabSize;
  extended: boolean;
}

function shapeStyle(options: FabOptions): Style {
  const height = heights[options.size];
  const shape: Style = { height, minWidth: height, radius: pillRadius(height) };
  if (options.extended) {
    return { ...shape, gap: 12, paddingX: 20 };
  }
  return shape;
}

/** A floating button. It rests on the lift shadow and rises to the overlay one while hovered. */
export function fabLook(theme: Theme, options: FabOptions): StateStyles {
  const colors = variantColors(theme, options.variant);
  const lift = visibleShadows(theme.shadows.lift);
  return {
    style: {
      direction: "row",
      align: "center",
      justify: "center",
      shrink: 0,
      cursor: "pointer",
      transition: stateTransition,
      color: colors.color,
      background: colors.background,
      shadow: lift,
      ...borderStyle(colors.borderColor),
      ...shapeStyle(options),
    },
    hover: { ...colors.hover, shadow: visibleShadows(theme.shadows.overlay) },
    active: { ...colors.active, shadow: lift },
    focus: focusStyle(theme, { hasBackground: true, hasBorder: false }, lift),
  };
}

export function fabLabelLook(): Style {
  return { ...typeStyle("body"), fontWeight: 500, nowrap: true };
}
