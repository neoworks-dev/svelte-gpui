import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { focusRing, stateLayerBackgrounds, stateTransition, type StateStyles } from "./look";

export const listRowChevronSize = 16;

/** The outlined box around the row. */
export function listRowFrameLook(theme: Theme, focused: boolean): Style {
  const frame: Style = {
    overflow: "hidden",
    radius: theme.radii.control,
    borderWidth: 1,
    borderColor: theme.colors.line,
    background: theme.colors.surface,
  };
  if (focused) {
    frame.shadow = [focusRing(theme)];
  }
  return frame;
}

/** The row itself. A row that can be pressed washes with the text colour on hover and press. */
export function listRowLook(theme: Theme, interactive: boolean): StateStyles {
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
  if (!interactive) {
    return { style };
  }
  const layers = stateLayerBackgrounds(theme, theme.colors.surface);
  return {
    style: { ...style, cursor: "pointer", transition: stateTransition },
    hover: { background: layers.hover },
    active: { background: layers.active },
  };
}

export function listRowTitleLook(): Style {
  return { ...typeStyle("body"), truncate: true };
}

export function listRowSubtitleLook(theme: Theme): Style {
  return { ...typeStyle("bodySmall"), color: theme.colors.muted, maxLines: 1 };
}
