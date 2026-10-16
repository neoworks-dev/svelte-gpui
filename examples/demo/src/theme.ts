// The demo's look. A theme in svelte-gpui is plain data: style objects and
// colours, imported where they are used and spread into each other.

import type { Style } from "@neoworks/svelte-gpui/ui";

export const colors = {
  window: "#0c0c0e",
  sidebar: "#111114",
  surface: "#16161a",
  raised: "#1f1f24",
  hover: "#26262c",
  pressed: "#2e2e35",
  border: "#26262c",
  strongBorder: "#3a3a42",
  text: "#f4f4f5",
  dim: "#a1a1aa",
  faint: "#6b6b74",
  accent: "#8b5cf6",
  accentSoft: "#8b5cf622",
  accentText: "#c4b5fd",
  green: "#4ade80",
  amber: "#fbbf24",
  red: "#f87171",
  blue: "#60a5fa",
  pink: "#f472b6",
};

export const card: Style = {
  background: colors.surface,
  borderWidth: 1,
  borderColor: colors.border,
  radius: 10,
  padding: 16,
  gap: 12,
};

export const cardTitle: Style = { fontSize: 13, fontWeight: "semibold", color: colors.text };
export const caption: Style = { fontSize: 12, color: colors.faint };
export const dim: Style = { color: colors.dim };
export const mono: Style = { fontFamily: "mono", fontSize: 12 };
export const row: Style = { direction: "row", align: "center", gap: 8 };

export const button: Style = {
  direction: "row",
  align: "center",
  gap: 6,
  paddingX: 12,
  paddingY: 6,
  radius: 6,
  background: colors.raised,
  borderWidth: 1,
  borderColor: colors.strongBorder,
  transition: 120,
};
export const buttonHover: Style = { background: colors.hover };
export const buttonActive: Style = { background: colors.pressed };

export const primaryButton: Style = { ...button, background: colors.accent, borderColor: colors.accent };
export const primaryHover: Style = { background: "#7c4ddf" };

export const field: Style = {
  paddingX: 10,
  paddingY: 7,
  radius: 6,
  background: colors.window,
  borderWidth: 1,
  borderColor: colors.strongBorder,
};
export const fieldFocus: Style = { borderColor: colors.accent };
