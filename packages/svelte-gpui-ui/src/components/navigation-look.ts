import type { Color, Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { withAlpha } from "./colorMix";
import { focusRing, focusStyle, pillRadius, stateTransition, visibleShadows, type StateStyles } from "./look";

/** The share of the page colour a glass surface keeps. The host has no backdrop blur (#58), so the web's 80% would show the content through unblurred. */
export const glassOpacity = 0.96;

export function glassBackground(color: Color): Color {
  return withAlpha(color, glassOpacity);
}

export type DestinationShape = "bar" | "rail";

export interface DestinationState {
  active: boolean;
  hovered: boolean;
  pressed: boolean;
  focused: boolean;
}

export const destinationIconSize = 22;

const indicatorHeight = 32;

const indicatorShapes: Record<DestinationShape, Style> = {
  bar: { width: "full", maxWidth: 64 },
  rail: { width: 56 },
};

function destinationColor(theme: Theme, state: DestinationState): Color {
  if (state.active || state.hovered) {
    return theme.colors.text;
  }
  return theme.colors.muted;
}

function indicatorBackground(theme: Theme, state: DestinationState): Color {
  if (state.active || state.pressed) {
    return theme.colors.pressed;
  }
  if (state.hovered || state.focused) {
    return theme.colors.hover;
  }
  return "transparent";
}

/** One destination: the indicator pill with the icon, and the label under it. */
export function destinationLook(theme: Theme, shape: DestinationShape, state: DestinationState): Style {
  const item: Style = {
    direction: "column",
    align: "center",
    gap: 4,
    minWidth: 0,
    paddingY: 4,
    ...typeStyle("caption"),
    fontWeight: 500,
    color: destinationColor(theme, state),
    cursor: "pointer",
    transition: stateTransition,
  };
  if (shape === "bar") {
    return { ...item, flex: 1, radius: pillRadius(48) };
  }
  return { ...item, width: 64, radius: theme.radii.control };
}

export function destinationIndicatorLook(theme: Theme, shape: DestinationShape, state: DestinationState): Style {
  const indicator: Style = {
    direction: "row",
    align: "center",
    justify: "center",
    height: indicatorHeight,
    radius: pillRadius(indicatorHeight),
    background: indicatorBackground(theme, state),
    transition: stateTransition,
    ...indicatorShapes[shape],
  };
  if (state.focused) {
    indicator.shadow = [focusRing(theme)];
  }
  return indicator;
}

export function destinationLabelLook(): Style {
  return { width: "full", textAlign: "center", truncate: true };
}

/** The floating pill of a NavBar. */
export function navBarLook(theme: Theme): Style {
  return {
    direction: "row",
    align: "stretch",
    justify: "around",
    marginX: 8,
    marginBottom: 8,
    paddingX: 8,
    paddingY: 6,
    radius: 999,
    borderWidth: 1,
    borderColor: theme.colors.line,
    background: glassBackground(theme.colors.surface),
    shadow: visibleShadows(theme.shadows.lift),
  };
}

export function navigationRailLook(theme: Theme): Style {
  return {
    direction: "column",
    align: "center",
    gap: 12,
    width: 80,
    height: "full",
    shrink: 0,
    paddingY: 16,
    borderRightWidth: 1,
    borderColor: theme.colors.line,
    background: theme.colors.surface,
  };
}

export const navigationRailColumn: Style = { direction: "column", align: "center", gap: 12 };

export const navigationRailItems: Style = { ...navigationRailColumn, flex: 1 };

export const drawerWidth = 288;

/** The page behind a modal drawer, which the web also blurs. */
export function drawerScrimBackground(theme: Theme): Color {
  return withAlpha(theme.colors.canvas, 0.7);
}

/** The column of a drawer. A docked one sits on the page; a modal one is a surface with the media radius on its right. */
export function drawerLook(theme: Theme, docked: boolean): Style {
  const column: Style = {
    direction: "column",
    gap: 16,
    width: drawerWidth,
    maxWidth: "full",
    height: "full",
    shrink: 0,
    paddingX: 12,
    paddingY: 16,
    color: theme.colors.text,
  };
  if (docked) {
    return column;
  }
  const media = theme.radii.media;
  return {
    ...column,
    background: theme.colors.surface,
    radius: [0, media, media, 0],
    shadow: visibleShadows(theme.shadows.overlay),
  };
}

export function drawerHeadlineLook(theme: Theme): Style {
  return { ...typeStyle("title"), color: theme.colors.text, paddingX: 16, paddingTop: 8, paddingBottom: 4 };
}

export function drawerSectionHeadingLook(theme: Theme): Style {
  return {
    ...typeStyle("caption"),
    fontWeight: 500,
    color: theme.colors.muted,
    paddingX: 16,
    paddingTop: 8,
    paddingBottom: 8,
  };
}

export function drawerItemGap(docked: boolean): number {
  if (docked) {
    return 6;
  }
  return 2;
}

export function drawerIconSize(docked: boolean): number {
  if (docked) {
    return 18;
  }
  return 22;
}

function drawerRowHeight(docked: boolean): number {
  if (docked) {
    return 40;
  }
  return 48;
}

function drawerRowType(docked: boolean): Style {
  if (docked) {
    return typeStyle("bodySmall");
  }
  return typeStyle("body");
}

export interface DrawerItemOptions {
  docked: boolean;
  active: boolean;
}

function drawerItemEmphasis(theme: Theme, active: boolean): Style {
  if (active) {
    return { fontWeight: 500, color: theme.colors.text, background: theme.colors.hover };
  }
  return { fontWeight: 400, color: theme.colors.muted, background: "transparent" };
}

/** A destination row. The active one is washed with the hover colour; the others are muted until hovered. */
export function drawerItemLook(theme: Theme, options: DrawerItemOptions): StateStyles {
  const { colors } = theme;
  const { active, docked } = options;
  return {
    style: {
      direction: "row",
      align: "center",
      gap: 12,
      height: drawerRowHeight(docked),
      paddingX: 16,
      radius: 999,
      ...drawerRowType(docked),
      ...drawerItemEmphasis(theme, active),
      cursor: "pointer",
      transition: stateTransition,
    },
    hover: { background: colors.hover, color: colors.text },
    active: { background: colors.pressed, color: colors.text },
    focus: focusStyle(theme, { hasBackground: active, hasBorder: false }),
  };
}

export function drawerBadgeLook(theme: Theme): Style {
  return { ...typeStyle("bodySmall"), color: theme.colors.muted };
}
