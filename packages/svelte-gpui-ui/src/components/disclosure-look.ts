import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { withAlpha } from "./colorMix";
import { stateTransition, type StateStyles } from "./look";

export type DisclosureGroupVariant = "plain" | "card";

export const disclosureChevronSize = 20;
export const disclosureIconSize = 20;
const disclosureIconCircleSize = 40;

/** A disclosure in a group draws a line above itself; the group clips the first one. */
export function disclosureLook(theme: Theme, inGroup: boolean): Style {
  const style: Style = { direction: "column", color: theme.colors.text };
  if (inGroup) {
    style.borderTopWidth = 1;
    style.borderColor = theme.colors.line;
  }
  return style;
}

/** The line that is always visible and opens the section. It washes with the text colour on hover and press. */
export function disclosureHeaderLook(theme: Theme): StateStyles {
  return {
    style: {
      direction: "row",
      align: "center",
      gap: 16,
      minHeight: 56,
      paddingX: 20,
      paddingY: 16,
      width: "full",
      cursor: "pointer",
      transition: stateTransition,
    },
    hover: { background: withAlpha(theme.colors.text, 0.04) },
    active: { background: withAlpha(theme.colors.text, 0.08) },
  };
}

/** An inset focus ring, drawn over the header so it takes no room. */
export function disclosureFocusRingLook(theme: Theme): Style {
  return { position: "absolute", inset: 0, borderWidth: 2, borderColor: theme.colors.focus };
}

export function disclosureIconCircleLook(theme: Theme): Style {
  return {
    direction: "row",
    align: "center",
    justify: "center",
    size: disclosureIconCircleSize,
    radius: disclosureIconCircleSize / 2,
    shrink: 0,
    background: theme.colors.sunken,
    color: theme.colors.text,
  };
}

export function disclosureTitleLook(): Style {
  return typeStyle("title");
}

export function disclosureSupportingTextLook(theme: Theme): Style {
  return { ...typeStyle("body"), color: theme.colors.muted };
}

export function disclosureContentLook(theme: Theme): Style {
  return { ...typeStyle("bodyLarge"), color: theme.colors.muted, paddingX: 20, paddingBottom: 20 };
}

/** The box around the disclosures, and what clips the line the first one draws above itself. */
export function disclosureGroupLook(theme: Theme, variant: DisclosureGroupVariant): Style {
  const style: Style = { direction: "column", overflow: "hidden" };
  if (variant === "plain") {
    return {
      ...style,
      borderTopWidth: 1,
      borderBottomWidth: 1,
      borderColor: theme.colors.line,
    };
  }
  return {
    ...style,
    radius: theme.radii.card,
    borderWidth: 1,
    borderColor: theme.colors.line,
    background: theme.colors.surface,
  };
}

export function disclosureGroupRowsLook(): Style {
  return { direction: "column", marginTop: -1 };
}
