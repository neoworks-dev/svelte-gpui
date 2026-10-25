import type { Color, Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme, type TypeRole } from "../theme";
import { borderStyle, disabledOpacity, focusStyle, pillRadius, pressedSolidOpacity, stateTransition, type StateStyles } from "./look";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "accent" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

const heights: Record<ButtonSize, number> = { sm: 32, md: 40, lg: 48 };
const gaps: Record<ButtonSize, number> = { sm: 6, md: 8, lg: 8 };
const horizontalPadding: Record<ButtonSize, number> = { sm: 14, md: 20, lg: 28 };
const typeRoles: Record<ButtonSize, TypeRole> = { sm: "bodySmall", md: "body", lg: "bodyLarge" };

export const buttonIconSizes: Record<ButtonSize, number> = { sm: 14, md: 16, lg: 18 };

interface VariantColors {
  background?: Color;
  color: Color;
  borderColor?: Color;
  hover: Style;
  active: Style;
}

function solidColors(background: Color, color: Color, hoverBackground: Color): VariantColors {
  return {
    background,
    color,
    hover: { background: hoverBackground },
    active: { background: hoverBackground, opacity: pressedSolidOpacity },
  };
}

function variantColors(theme: Theme, variant: ButtonVariant): VariantColors {
  const { colors } = theme;
  if (variant === "primary") {
    return solidColors(colors.fill, colors.onFill, colors.fillHover);
  }
  if (variant === "accent") {
    return solidColors(colors.accent, colors.onAccent, colors.accentHover);
  }
  if (variant === "danger") {
    return solidColors(colors.dangerSoft, colors.danger, colors.dangerSoftHover);
  }
  if (variant === "ghost") {
    return {
      color: colors.muted,
      hover: { background: colors.hover, color: colors.text },
      active: { background: colors.pressed, color: colors.text },
    };
  }
  return {
    color: colors.text,
    borderColor: colors.lineStrong,
    hover: { background: colors.hover },
    active: { background: colors.pressed },
  };
}

export interface ButtonOptions {
  variant: ButtonVariant;
  size: ButtonSize;
  full: boolean;
  round: boolean;
}

function shapeStyle(options: ButtonOptions): Style {
  const height = heights[options.size];
  if (options.round) {
    return { width: height, height, radius: pillRadius(height) };
  }
  const shape: Style = { height, paddingX: horizontalPadding[options.size], radius: pillRadius(height) };
  if (options.full) {
    shape.width = "full";
  }
  return shape;
}

export function buttonLook(theme: Theme, options: ButtonOptions): StateStyles {
  const colors = variantColors(theme, options.variant);
  return {
    style: {
      direction: "row",
      align: "center",
      justify: "center",
      gap: gaps[options.size],
      ...typeStyle(typeRoles[options.size]),
      fontWeight: 500,
      nowrap: true,
      cursor: "pointer",
      transition: stateTransition,
      color: colors.color,
      background: colors.background,
      ...borderStyle(colors.borderColor),
      ...shapeStyle(options),
    },
    hover: colors.hover,
    active: colors.active,
    focus: focusStyle(theme, { hasBackground: colors.background !== undefined, hasBorder: colors.borderColor !== undefined }),
    disabledStyle: { opacity: disabledOpacity },
  };
}
