import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { ring } from "./ring";
import { disabledOpacity, focusStyle, pillRadius, stateTransition, type StateStyles } from "./look";
import { glassPanelLook } from "./overlay-look";

export const datePanelWidth = 320;
export const datePanelPadding = 16;
export const dayCellSize = 36;
export const monthCellHeight = 40;
export const monthCellGap = 4;
export const pickerPanelOffset = 8;
export const timePanelPadding = 12;
export const timeWheelWidth = 64;

const fieldHeight = 52;

/**
 * Focus for a picker button. A focus style that sets `color` makes the host drop the button's own font size and weight,
 * so a box with neither a background nor a border takes only the hover wash.
 */
function pickerFocusStyle(theme: Theme, appearance: { hasBackground: boolean; hasBorder: boolean }): Style {
  if (appearance.hasBackground || appearance.hasBorder) {
    return focusStyle(theme, appearance);
  }
  return { background: theme.colors.hover };
}

export function datePanelLook(theme: Theme): Style {
  return { ...glassPanelLook(theme), padding: datePanelPadding, width: datePanelWidth };
}

export function timePanelLook(theme: Theme): Style {
  return { ...glassPanelLook(theme), padding: timePanelPadding };
}

/** The previous and next buttons sit at the edges and the title button takes the middle. */
export const dateHeaderLook: Style = { direction: "row", align: "center", justify: "between", marginBottom: 12 };

export const dateRowLook: Style = { direction: "row", width: "full" };

export function weekdayHeaderLook(theme: Theme): Style {
  return { ...typeStyle("caption"), flex: 1, textAlign: "center", paddingY: 4, color: theme.colors.muted };
}

export function headerTitleLook(theme: Theme): StateStyles {
  return {
    style: {
      ...typeStyle("body"),
      fontWeight: 500,
      paddingX: 12,
      paddingY: 4,
      radius: 999,
      color: theme.colors.text,
      cursor: "pointer",
      transition: stateTransition,
    },
    hover: { background: theme.colors.hover },
    focus: pickerFocusStyle(theme, { hasBackground: false, hasBorder: false }),
  };
}

/** The heading-sized button that opens the calendar from a page header. */
export function dateTitleTriggerLook(theme: Theme): StateStyles {
  return {
    style: {
      ...typeStyle("title"),
      direction: "row",
      align: "center",
      gap: 6,
      paddingX: 12,
      paddingY: 4,
      radius: 999,
      color: theme.colors.text,
      cursor: "pointer",
      transition: stateTransition,
    },
    hover: { background: theme.colors.hover },
    focus: pickerFocusStyle(theme, { hasBackground: false, hasBorder: false }),
  };
}

export interface DayState {
  selected: boolean;
  today: boolean;
  inViewedMonth: boolean;
  selectable: boolean;
}

/** One day of the grid. A border is always drawn, transparent unless it is today, so every day keeps the same size. */
export function dayLook(theme: Theme, state: DayState): StateStyles {
  const { colors } = theme;
  const style: Style = {
    ...typeStyle("bodySmall"),
    direction: "row",
    align: "center",
    justify: "center",
    size: dayCellSize,
    radius: pillRadius(dayCellSize),
    borderWidth: 1,
    borderColor: "transparent",
    color: colors.text,
    cursor: "pointer",
    transition: stateTransition,
  };
  const focus = pickerFocusStyle(theme, { hasBackground: state.selected, hasBorder: state.today && !state.selected });
  if (state.selected) {
    return {
      style: { ...style, background: colors.fill, color: colors.onFill, fontWeight: 500 },
      hover: { background: colors.fillHover },
      focus,
      disabledStyle: { opacity: disabledOpacity },
    };
  }
  if (state.today) {
    style.borderColor = colors.lineStrong;
  }
  if (!state.inViewedMonth) {
    style.color = colors.faint;
  }
  return {
    style,
    hover: { background: colors.hover },
    focus,
    disabledStyle: { opacity: 0.3, cursor: "default" },
  };
}

export function monthLook(theme: Theme, selected: boolean): StateStyles {
  const { colors } = theme;
  const style: Style = {
    ...typeStyle("body"),
    direction: "row",
    align: "center",
    justify: "center",
    flex: 1,
    height: monthCellHeight,
    radius: pillRadius(monthCellHeight),
    color: colors.text,
    cursor: "pointer",
    transition: stateTransition,
  };
  const focus = pickerFocusStyle(theme, { hasBackground: selected, hasBorder: false });
  if (selected) {
    return {
      style: { ...style, background: colors.fill, color: colors.onFill, fontWeight: 500 },
      hover: { background: colors.fillHover },
      focus,
    };
  }
  return { style, hover: { background: colors.hover }, focus };
}

/** The sunken pill of the time field. The ring shows while the picker is open or the value is wrong. */
export function timeFieldLook(theme: Theme, state: { open: boolean; error: boolean }): StateStyles {
  const { colors } = theme;
  const style: Style = {
    ...typeStyle("body"),
    direction: "row",
    align: "center",
    justify: "between",
    width: "full",
    minHeight: fieldHeight,
    paddingX: 20,
    paddingTop: 20,
    paddingBottom: 4,
    radius: 999,
    borderWidth: 1,
    borderColor: "transparent",
    background: colors.sunken,
    color: colors.text,
    cursor: "pointer",
    transition: 200,
  };
  if (state.error) {
    style.shadow = ring(colors.danger, 2);
  } else if (state.open) {
    style.shadow = ring(colors.text, 2);
  }
  return {
    style,
    hover: { background: colors.pressed },
    focus: focusStyle(theme, { hasBackground: true, hasBorder: false }),
  };
}

export function timeFieldLabelLook(theme: Theme, state: { open: boolean; error: boolean }): Style {
  let color = theme.colors.muted;
  if (state.error) {
    color = theme.colors.danger;
  } else if (state.open) {
    color = theme.colors.text;
  }
  return { ...typeStyle("caption"), position: "absolute", left: 20, top: 6, color };
}

export function supportingTextLook(theme: Theme, error: boolean): Style {
  let color = theme.colors.muted;
  if (error) {
    color = theme.colors.danger;
  }
  return { ...typeStyle("caption"), marginTop: 6, paddingX: 20, color };
}
