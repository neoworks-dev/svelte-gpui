// The surfaces floating content is drawn on: frosted menus, the plain surface of dialogs and sheets, and the scrim.

import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { withAlpha } from "./colorMix";
import { stateTransition, visibleShadows } from "./look";

/** The web's canvas at 80% over a backdrop blur. The host has no blur, so the canvas is nearly opaque instead. */
const glassOpacity = 0.96;

/** The panel of a menu, a select list or a popover. */
export function glassPanelLook(theme: Theme): Style {
  return {
    background: withAlpha(theme.colors.canvas, glassOpacity),
    radius: theme.radii.card,
    shadow: visibleShadows(theme.shadows.overlay),
    color: theme.colors.text,
  };
}

/** The dimmed, normally blurred, layer behind a dialog or modal sheet. */
export function scrimColor(theme: Theme): string {
  return withAlpha(theme.colors.canvas, 0.7);
}

/** The card of a dialog or sheet. Corners are left to the caller since sheets round only the edge facing the page. */
export function surfacePanelLook(theme: Theme): Style {
  return {
    background: theme.colors.surface,
    color: theme.colors.text,
    shadow: visibleShadows(theme.shadows.overlay),
  };
}

export const menuPanelPadding = 6;
export const menuPanelMinimumWidth = 208;
export const optionRowPaddingX = 12;
export const optionRowPaddingY = 8;
export const optionRowIconSize = 18;
export const optionRowCheckSize = 16;

/** One row of a menu or a select list. `active` is the keyboard or pointer highlight. */
export function optionRowLook(theme: Theme, state: { active: boolean; danger: boolean; disabled: boolean }): Style {
  const row: Style = {
    direction: "row",
    align: "center",
    gap: 12,
    width: "full",
    shrink: 0,
    paddingX: optionRowPaddingX,
    paddingY: optionRowPaddingY,
    radius: theme.radii.control,
    cursor: "pointer",
    background: "transparent",
    color: theme.colors.text,
    transition: stateTransition,
  };
  if (state.active) {
    row.background = theme.colors.hover;
  }
  if (state.danger) {
    row.color = theme.colors.danger;
  }
  if (state.disabled) {
    row.opacity = 0.4;
    row.cursor = "default";
  }
  return row;
}

/** The small title above a group of menu items. */
export function menuHeadingLook(theme: Theme): Style {
  return { ...typeStyle("label"), uppercase: true, color: theme.colors.muted, paddingX: 12, paddingTop: 8, paddingBottom: 4 };
}

export function menuShortcutLook(theme: Theme): Style {
  return { ...typeStyle("caption"), color: theme.colors.faint, paddingLeft: 24, shrink: 0 };
}

/** The height of a one-line row: padding plus the body line. */
export function optionRowHeight(withDescription: boolean): number {
  const bodyLine = typeStyle("body").lineHeight as number;
  if (!withDescription) {
    return optionRowPaddingY * 2 + bodyLine;
  }
  return optionRowPaddingY * 2 + bodyLine + (typeStyle("bodySmall").lineHeight as number);
}
