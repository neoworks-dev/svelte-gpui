import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme, type TypeRole } from "../theme";
import { glassBackground } from "./navigation-look";
import { stateTransition } from "./look";

export type TopAppBarVariant = "small" | "centered" | "medium" | "large";

const headlineRoles: Record<TopAppBarVariant, TypeRole> = {
  small: "title",
  centered: "title",
  medium: "heading",
  large: "headline",
};

/** Whether the title sits in its own bigger block under the buttons until the page scrolls. */
export function hasHeadlineBlock(variant: TopAppBarVariant): boolean {
  return variant === "medium" || variant === "large";
}

/** The title in the button row: always for small bars, and once scrolled for the ones with a headline block. */
export function showsRowTitle(variant: TopAppBarVariant, scrolled: boolean): boolean {
  return !hasHeadlineBlock(variant) || scrolled;
}

/** The bar. Scrolled, it is glass with no border; at the top of the page it is the canvas itself. */
export function topAppBarLook(theme: Theme, scrolled: boolean): Style {
  const bar: Style = {
    direction: "column",
    width: "full",
    shrink: 0,
    color: theme.colors.text,
    transition: stateTransition,
  };
  if (scrolled) {
    return { ...bar, background: glassBackground(theme.colors.canvas) };
  }
  return { ...bar, background: theme.colors.canvas };
}

export const topAppBarRowLook: Style = {
  direction: "row",
  align: "center",
  gap: 4,
  height: 64,
  paddingX: 12,
};

export const topAppBarSlotLook: Style = { direction: "row", align: "center", gap: 4, shrink: 0 };

export function topAppBarTitleLook(variant: TopAppBarVariant): Style {
  const title: Style = { ...typeStyle("title"), truncate: true };
  if (variant === "centered") {
    return { ...title, textAlign: "center" };
  }
  return title;
}

export function topAppBarTitleSlotLook(): Style {
  return { flex: 1, minWidth: 0, paddingX: 4 };
}

/** The big title under the buttons of `medium` and `large` bars. It folds away once the page scrolls. */
export function topAppBarHeadlineLook(scrolled: boolean): Style {
  const block: Style = { paddingX: 16, paddingBottom: 20 };
  if (scrolled) {
    return { ...block, display: "none" };
  }
  return block;
}

export function topAppBarHeadlineTextLook(variant: TopAppBarVariant): Style {
  return { ...typeStyle(headlineRoles[variant]), truncate: true };
}
