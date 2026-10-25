import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme, type TypeRole } from "../theme";
import { ring } from "./ring";
import { disabledOpacity, focusStyle, stateTransition, type StateStyles } from "./look";

export type SelectSize = "sm" | "md";
/** `surface` and `ghost` are compact pills for toolbars and rows, `outlined` and `filled` are full-height form fields with a label. */
export type SelectVariant = "surface" | "ghost" | "outlined" | "filled";

export const selectIconSizes: Record<SelectSize, number> = { sm: 14, md: 16 };
export const selectFieldHeight = 52;
export const selectListMaximumHeight = 288;
export const selectPanelMinimumWidth = 192;

const compactHeights: Record<SelectSize, number> = { sm: 32, md: 40 };
const compactPaddings: Record<SelectSize, number> = { sm: 14, md: 16 };
const compactTypeRoles: Record<SelectSize, TypeRole> = { sm: "bodySmall", md: "body" };
const compactGaps: Record<SelectSize, number> = { sm: 6, md: 8 };

export function isFieldVariant(variant: SelectVariant): boolean {
  return variant === "outlined" || variant === "filled";
}

export interface SelectTriggerState {
  variant: SelectVariant;
  size: SelectSize;
  open: boolean;
  error: boolean;
  disabled: boolean;
}

function baseTrigger(disabled: boolean): Style {
  const base: Style = {
    direction: "row",
    align: "center",
    justify: "between",
    width: "full",
    radius: 999,
    cursor: "pointer",
    transition: stateTransition,
  };
  if (disabled) {
    base.opacity = disabledOpacity;
    base.cursor = "default";
  }
  return base;
}

function compactTrigger(theme: Theme, state: SelectTriggerState): StateStyles {
  const { colors } = theme;
  const shape: Style = {
    ...baseTrigger(state.disabled),
    ...typeStyle(compactTypeRoles[state.size]),
    height: compactHeights[state.size],
    gap: compactGaps[state.size],
    paddingX: compactPaddings[state.size],
  };
  if (state.variant === "ghost") {
    return {
      style: { ...shape, color: colors.muted },
      hover: { background: colors.hover, color: colors.text },
      focus: focusStyle(theme, { hasBackground: false, hasBorder: false }),
    };
  }
  return {
    style: { ...shape, color: colors.text, borderWidth: 1, borderColor: colors.lineStrong },
    hover: { background: colors.hover },
    focus: focusStyle(theme, { hasBackground: false, hasBorder: true }),
  };
}

/** A ring shadow also fills a box that has no background, so the outlined field recolours its border instead. */
function fieldBorderAndRing(theme: Theme, state: SelectTriggerState): Style {
  const { colors } = theme;
  const filled = state.variant === "filled";
  if (state.error && filled) {
    return { borderColor: "transparent", background: colors.sunken, shadow: ring(colors.danger, 2) };
  }
  if (state.error) {
    return { borderColor: colors.danger };
  }
  if (state.open && filled) {
    return { borderColor: "transparent", background: colors.sunken, shadow: ring(colors.text, 2) };
  }
  if (state.open) {
    return { borderColor: colors.text };
  }
  if (filled) {
    return { borderColor: "transparent", background: colors.sunken };
  }
  return { borderColor: colors.line };
}

function fieldHover(theme: Theme, state: SelectTriggerState): Style {
  if (state.error || state.open) {
    return {};
  }
  if (state.variant === "filled") {
    return { background: theme.colors.pressed };
  }
  return { borderColor: theme.colors.lineStrong };
}

function fieldFocus(theme: Theme, state: SelectTriggerState): Style {
  if (state.variant === "filled") {
    return { shadow: ring(theme.colors.text, 2) };
  }
  return { borderColor: theme.colors.text };
}

function fieldTrigger(theme: Theme, state: SelectTriggerState): StateStyles {
  const style: Style = {
    ...baseTrigger(state.disabled),
    ...typeStyle("body"),
    minHeight: selectFieldHeight,
    paddingX: 20,
    borderWidth: 1,
    color: theme.colors.text,
    ...fieldBorderAndRing(theme, state),
  };
  return {
    style,
    hover: fieldHover(theme, state),
    focus: fieldFocus(theme, state),
  };
}

export function selectTriggerLook(theme: Theme, state: SelectTriggerState): StateStyles {
  if (isFieldVariant(state.variant)) {
    return fieldTrigger(theme, state);
  }
  return compactTrigger(theme, state);
}

/** The label of an outlined or filled select, which sits inside the field and moves up once there is a value. */
export function selectFieldLabelLook(theme: Theme, state: { floated: boolean; error: boolean; open: boolean }): Style {
  let color = theme.colors.muted;
  if (state.error) {
    color = theme.colors.danger;
  } else if (state.open) {
    color = theme.colors.text;
  }
  if (state.floated) {
    return { ...typeStyle("caption"), position: "absolute", left: 20, top: 6, color };
  }
  return { ...typeStyle("body"), position: "absolute", left: 20, top: 14, color };
}
