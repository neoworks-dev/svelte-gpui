import type { Color, Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { withAlpha } from "./colorMix";
import { disabledOpacity, stateTransition } from "./look";

export type TabsVariant = "primary" | "secondary";

export const tabIconSize = 22;

export interface TabState {
  active: boolean;
  hovered: boolean;
  pressed: boolean;
  focused: boolean;
  hasIcon: boolean;
  disabled: boolean;
}

export interface TabsOptions {
  variant: TabsVariant;
  fullWidth: boolean;
}

function tabType(options: TabsOptions, hasIcon: boolean): Style {
  if (options.variant === "primary" && !hasIcon) {
    return { ...typeStyle("body"), minHeight: 48 };
  }
  if (options.variant === "primary") {
    return { ...typeStyle("bodySmall"), minHeight: 64, paddingY: 8 };
  }
  return { ...typeStyle("bodySmall"), minHeight: 48 };
}

function tabColor(theme: Theme, state: TabState): Color {
  if (state.active || state.hovered) {
    return theme.colors.text;
  }
  return theme.colors.muted;
}

function tabLayout(options: TabsOptions, hasIcon: boolean): Style {
  const layout: Style = { align: "center", justify: "center", gap: 8, paddingX: 16, nowrap: true, shrink: 0 };
  if (options.variant === "primary" && hasIcon) {
    layout.direction = "column";
  } else {
    layout.direction = "row";
  }
  if (options.fullWidth) {
    return { ...layout, flex: 1, shrink: 1, minWidth: 0 };
  }
  return layout;
}

function tabCursor(disabled: boolean): Style {
  if (disabled) {
    return { cursor: "not-allowed", opacity: disabledOpacity };
  }
  return { cursor: "pointer" };
}

export function tabLook(theme: Theme, options: TabsOptions, state: TabState): Style {
  return {
    position: "relative",
    fontWeight: 500,
    color: tabColor(theme, state),
    transition: stateTransition,
    ...tabType(options, state.hasIcon),
    ...tabLayout(options, state.hasIcon),
    ...tabCursor(state.disabled),
  };
}

function washBackground(theme: Theme, state: TabState): Color {
  if (state.pressed) {
    return withAlpha(theme.colors.text, 0.08);
  }
  if (state.hovered) {
    return withAlpha(theme.colors.text, 0.04);
  }
  return "transparent";
}

/** The faint wash laid over a hovered or pressed tab, which carries the focus ring too. */
export function tabWashLook(theme: Theme, state: TabState): Style {
  const wash: Style = {
    position: "absolute",
    inset: 4,
    radius: theme.radii.control,
    background: washBackground(theme, state),
    transition: stateTransition,
  };
  if (state.focused) {
    return { ...wash, borderWidth: 2, borderColor: theme.colors.focus };
  }
  return wash;
}

function indicatorOpacity(active: boolean): number {
  if (active) {
    return 1;
  }
  return 0;
}

function indicatorHeight(variant: TabsVariant): number {
  if (variant === "primary") {
    return 2;
  }
  return 1;
}

/** The line under the active tab. */
export function tabIndicatorLook(theme: Theme, options: TabsOptions, active: boolean): Style {
  return {
    position: "absolute",
    left: 12,
    right: 12,
    bottom: 0,
    height: indicatorHeight(options.variant),
    radius: 999,
    background: theme.colors.text,
    opacity: indicatorOpacity(active),
    transition: stateTransition,
  };
}

export function tabCountLook(theme: Theme): Style {
  return {
    ...typeStyle("caption"),
    fontWeight: 500,
    color: theme.colors.muted,
    background: theme.colors.sunken,
    radius: 999,
    paddingX: 6,
  };
}

export function tabListLook(theme: Theme, fullWidth: boolean): Style {
  const list: Style = { direction: "row", borderBottomWidth: 1, borderColor: theme.colors.line };
  if (fullWidth) {
    return { ...list, width: "full" };
  }
  return list;
}
