import type { Color, Style } from "@neoworks/svelte-gpui/ui";
import type { Theme } from "../theme";
import { borderStyle, disabledOpacity, focusStyle, pillRadius, pressedSolidOpacity, stateTransition, type StateStyles } from "./look";

export type IconButtonVariant = "standard" | "filled" | "tonal" | "outlined";
export type IconButtonSize = "sm" | "md" | "lg";

const sizes: Record<IconButtonSize, number> = { sm: 32, md: 40, lg: 48 };

export const iconButtonIconSizes: Record<IconButtonSize, number> = { sm: 16, md: 20, lg: 24 };

interface Colors {
  background?: Color;
  color: Color;
  borderColor?: Color;
  hover: Style;
  active: Style;
}

function quietColors(theme: Theme, selected: boolean): Colors {
  const { colors } = theme;
  const pressed = { background: colors.pressed, color: colors.text };
  if (selected) {
    return { color: colors.text, hover: { background: colors.hover }, active: pressed };
  }
  return { color: colors.muted, hover: { background: colors.hover, color: colors.text }, active: pressed };
}

function filledColors(theme: Theme, toggledOff: boolean): Colors {
  const { colors } = theme;
  if (toggledOff) {
    return {
      background: colors.sunken,
      color: colors.muted,
      hover: { background: colors.hover, color: colors.text },
      active: { background: colors.pressed, color: colors.text },
    };
  }
  return {
    background: colors.fill,
    color: colors.onFill,
    hover: { background: colors.fillHover },
    active: { background: colors.fillHover, opacity: pressedSolidOpacity },
  };
}

function tonalColors(theme: Theme, selected: boolean): Colors {
  const { colors } = theme;
  if (selected) {
    return { background: colors.pressed, color: colors.text, hover: {}, active: {} };
  }
  return {
    background: colors.sunken,
    color: colors.text,
    hover: { background: colors.hover },
    active: { background: colors.pressed },
  };
}

function outlinedColors(theme: Theme, selected: boolean): Colors {
  const { colors } = theme;
  if (selected) {
    return filledColors(theme, false);
  }
  return {
    color: colors.text,
    borderColor: colors.lineStrong,
    hover: { background: colors.hover },
    active: { background: colors.pressed },
  };
}

export interface IconButtonOptions {
  variant: IconButtonVariant;
  size: IconButtonSize;
  /** `undefined` for a plain button, otherwise the toggle's current state. */
  selected: boolean | undefined;
}

function variantColors(theme: Theme, options: IconButtonOptions): Colors {
  const selected = options.selected === true;
  if (options.variant === "standard") {
    return quietColors(theme, selected);
  }
  if (options.variant === "filled") {
    return filledColors(theme, options.selected === false);
  }
  if (options.variant === "tonal") {
    return tonalColors(theme, selected);
  }
  return outlinedColors(theme, selected);
}

export function iconButtonLook(theme: Theme, options: IconButtonOptions): StateStyles {
  const colors = variantColors(theme, options);
  const side = sizes[options.size];
  return {
    style: {
      direction: "row",
      align: "center",
      justify: "center",
      size: side,
      radius: pillRadius(side),
      cursor: "pointer",
      transition: stateTransition,
      color: colors.color,
      background: colors.background,
      ...borderStyle(colors.borderColor),
    },
    hover: colors.hover,
    active: colors.active,
    focus: focusStyle(theme, { hasBackground: colors.background !== undefined, hasBorder: colors.borderColor !== undefined }),
    disabledStyle: { opacity: disabledOpacity },
  };
}
