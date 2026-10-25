import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { focusRing, stateTransition } from "./look";
import { wheelHeight, wheelItemHeight, wheelVisibleItems } from "./wheelModel";

export function wheelColumnLook(theme: Theme): Style {
  return {
    position: "relative",
    width: "full",
    height: wheelHeight,
    overflow: "hidden",
    radius: theme.radii.control,
    cursor: "grab",
  };
}

/** The band behind the centred row. Inset by the focus ring's width so the clipped column still shows the whole ring. */
export function wheelSelectionLook(theme: Theme, focused: boolean): Style {
  const band: Style = {
    position: "absolute",
    left: 2,
    right: 2,
    top: Math.floor(wheelVisibleItems / 2) * wheelItemHeight,
    height: wheelItemHeight,
    radius: theme.radii.control,
    background: theme.colors.hover,
  };
  if (focused) {
    band.shadow = [focusRing(theme)];
  }
  return band;
}

export function wheelItemLook(theme: Theme, top: number, selected: boolean): Style {
  const item: Style = {
    ...typeStyle("body"),
    position: "absolute",
    left: 0,
    top,
    width: "full",
    height: wheelItemHeight,
    direction: "row",
    align: "center",
    justify: "center",
    cursor: "pointer",
    transition: stateTransition,
    color: theme.colors.faint,
  };
  if (selected) {
    item.color = theme.colors.text;
    item.fontWeight = 500;
  }
  return item;
}

export function wheelItemHover(theme: Theme, selected: boolean): Style {
  if (selected) {
    return {};
  }
  return { color: theme.colors.muted };
}
