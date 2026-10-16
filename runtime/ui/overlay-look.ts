// The default look of floating panels: popovers, menus, selects and tooltips. A panel takes a
// `style`, so any of these can be replaced or spread into your own.

import type { Style } from "./style";

export const colors = {
  panel: "#1c1c21",
  border: "#34343c",
  text: "#f4f4f5",
  dim: "#a1a1aa",
  highlight: "#ffffff14",
  accent: "#8b5cf6",
  danger: "#f87171",
};

/** A floating card. */
export const panel: Style = {
  background: colors.panel,
  borderWidth: 1,
  borderColor: colors.border,
  radius: 8,
  padding: 4,
  shadow: { y: 8, blur: 24, color: "#00000099" },
  color: colors.text,
};

/** A small label that appears on hover. */
export const tip: Style = {
  background: "#27272a",
  borderWidth: 1,
  borderColor: colors.border,
  radius: 5,
  paddingX: 8,
  paddingY: 4,
  maxWidth: 280,
  fontSize: 12,
  color: colors.text,
  shadow: { y: 4, blur: 12, color: "#00000080" },
};
