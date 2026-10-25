import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { withAlpha } from "./colorMix";
import { blendColors, disabledOpacity, stateTransition, type StateStyles } from "./look";

export type ListVariant = "card" | "plain";
export type ListLines = 1 | 2 | 3;

export const listItemIconCircleSize = 40;
export const listItemIconSize = 20;
export const listItemChevronSize = 16;

/**
 * The box around the rows. When the rows draw a line above themselves, the box clips the first one, which has
 * nothing above it to divide from.
 */
export function listLook(theme: Theme, variant: ListVariant, dividers: boolean): Style {
  const style: Style = { direction: "column", color: theme.colors.text };
  if (dividers) {
    style.overflow = "hidden";
  }
  if (variant === "plain") {
    return style;
  }
  return {
    ...style,
    overflow: "hidden",
    radius: theme.radii.card,
    borderWidth: 1,
    borderColor: theme.colors.line,
    background: theme.colors.surface,
  };
}

/** The rows' own column, pulled up by the line the first row draws above itself. */
export function listRowsLook(dividers: boolean): Style {
  if (dividers) {
    return { direction: "column", marginTop: -1 };
  }
  return { direction: "column" };
}

export interface ListItemOptions {
  interactive: boolean;
  selected: boolean;
  disabled: boolean;
  /** Draws a line above the row. */
  dividers: boolean;
}

/** The faint wash of the text colour over the row, as a background that already includes the row's own. */
function wash(theme: Theme, selected: boolean, share: number): string {
  if (selected) {
    return blendColors(theme.colors.sunken, theme.colors.text, share);
  }
  return withAlpha(theme.colors.text, share);
}

export function listItemLook(theme: Theme, options: ListItemOptions): StateStyles {
  const style: Style = {
    direction: "row",
    align: "center",
    gap: 16,
    minHeight: 56,
    paddingX: 16,
    paddingY: 12,
    width: "full",
    color: theme.colors.text,
  };
  if (options.selected) {
    style.background = theme.colors.sunken;
  }
  if (options.dividers) {
    style.borderTopWidth = 1;
    style.borderColor = theme.colors.line;
  }
  if (options.disabled) {
    style.opacity = disabledOpacity;
  }
  if (!options.interactive) {
    return { style };
  }
  return {
    style: { ...style, cursor: "pointer", transition: stateTransition },
    hover: { background: wash(theme, options.selected, 0.04) },
    active: { background: wash(theme, options.selected, 0.08) },
  };
}

/** An inset focus ring, drawn over the row so it takes no room. */
export function listItemFocusRingLook(theme: Theme): Style {
  return { position: "absolute", inset: 0, borderWidth: 2, borderColor: theme.colors.focus };
}

export function listItemIconCircleLook(theme: Theme): Style {
  return {
    direction: "row",
    align: "center",
    justify: "center",
    size: listItemIconCircleSize,
    radius: listItemIconCircleSize / 2,
    shrink: 0,
    background: theme.colors.sunken,
    color: theme.colors.text,
  };
}

export function listItemOverlineLook(theme: Theme): Style {
  return { ...typeStyle("label"), color: theme.colors.muted, uppercase: true, truncate: true };
}

export function listItemHeadlineLook(): Style {
  return { ...typeStyle("body"), truncate: true };
}

/** One line is cut off with an ellipsis. Two or three are clamped at the last line without one, because the host measures an ellipsis over several lines as one. */
export function listItemSupportingTextLook(theme: Theme, lines: ListLines): Style {
  const style: Style = { ...typeStyle("bodySmall"), color: theme.colors.muted };
  if (lines === 1) {
    return { ...style, truncate: true };
  }
  return { ...style, maxLines: lines };
}

export function listItemTrailingTextLook(theme: Theme): Style {
  return { ...typeStyle("bodySmall"), color: theme.colors.muted, nowrap: true, shrink: 0 };
}
