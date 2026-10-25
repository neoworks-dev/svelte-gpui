import type { Color, Shadow, Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { disabledOpacity, focusStyle, pillRadius, pressedSolidOpacity, stateTransition, type StateStyles } from "./look";

export type ChipVariant = "assist" | "filter" | "input" | "suggestion";

const chipHeight = 32;

export const chipIconSize = 16;
export const chipTrailingIconSize = 14;

export interface ChipOptions {
  /** A filter chip that is applied. */
  applied: boolean;
  elevated: boolean;
  hasLeading: boolean;
  hasTrailingIcon: boolean;
  /** An input chip with a remove button beside it. */
  removable: boolean;
}

interface ChipColors {
  borderColor: Color;
  background?: Color;
  color: Color;
  hoverBackground: Color;
  shadow: Shadow[];
}

function chipColors(theme: Theme, options: ChipOptions): ChipColors {
  const { colors } = theme;
  if (options.applied) {
    return { borderColor: "transparent", background: colors.fill, color: colors.onFill, hoverBackground: colors.fillHover, shadow: [] };
  }
  if (options.elevated) {
    return {
      borderColor: "transparent",
      background: colors.surface,
      color: colors.text,
      hoverBackground: colors.hover,
      shadow: theme.shadows.lift,
    };
  }
  return { borderColor: colors.lineStrong, color: colors.text, hoverBackground: colors.hover, shadow: [] };
}

function paddingLeft(options: ChipOptions): number {
  if (options.hasLeading) {
    return 12;
  }
  return 14;
}

function paddingRight(options: ChipOptions): number {
  if (options.removable) {
    return 8;
  }
  if (options.hasTrailingIcon) {
    return 12;
  }
  return 14;
}

/** A removable chip's main part is the left half of a pill, joined to its remove button. */
function shapeStyle(options: ChipOptions): Style {
  const radius = pillRadius(chipHeight);
  if (options.removable) {
    return { borderRightWidth: 0, radius: [radius, 0, 0, radius] };
  }
  return { radius };
}

export function chipLook(theme: Theme, options: ChipOptions): StateStyles {
  const colors = chipColors(theme, options);
  return {
    style: {
      direction: "row",
      align: "center",
      gap: 6,
      height: chipHeight,
      paddingLeft: paddingLeft(options),
      paddingRight: paddingRight(options),
      ...typeStyle("bodySmall"),
      nowrap: true,
      cursor: "pointer",
      transition: stateTransition,
      borderWidth: 1,
      borderColor: colors.borderColor,
      color: colors.color,
      background: colors.background,
      shadow: colors.shadow,
      ...shapeStyle(options),
    },
    hover: { background: colors.hoverBackground },
    active: { background: colors.hoverBackground, opacity: pressedSolidOpacity },
    focus: focusStyle(theme, { hasBackground: colors.background !== undefined, hasBorder: true }, colors.shadow),
    disabledStyle: { opacity: disabledOpacity },
  };
}

const removeSide = 32;

/** The remove button of an input chip: the right half of the pill, without a border on the side it joins. */
export function chipRemoveLook(theme: Theme): StateStyles {
  const { colors } = theme;
  const radius = pillRadius(chipHeight);
  return {
    style: {
      direction: "row",
      align: "center",
      justify: "center",
      width: removeSide,
      height: chipHeight,
      radius: [0, radius, radius, 0],
      borderWidth: [1, 1, 1, 0],
      borderColor: colors.lineStrong,
      color: colors.muted,
      cursor: "pointer",
      transition: stateTransition,
    },
    hover: { background: colors.hover, color: colors.text },
    active: { background: colors.pressed, color: colors.text },
    focus: focusStyle(theme, { hasBackground: false, hasBorder: true }),
    disabledStyle: { opacity: disabledOpacity },
  };
}
