import type { Style } from "@neoworks/svelte-gpui/ui";

export const colors = {
  window: "#0c0c0e",
  surface: "#16161a",
  raised: "#1f1f24",
  hover: "#26262c",
  border: "#26262c",
  strongBorder: "#3a3a42",
  text: "#f4f4f5",
  dim: "#a1a1aa",
  faint: "#6b6b74",
  accent: "#f97316",
  green: "#4ade80",
  red: "#f87171",
};

export const card: Style = {
  background: colors.surface,
  borderWidth: 1,
  borderColor: colors.border,
  radius: 10,
  padding: 16,
  gap: 12,
};

export const button: Style = {
  paddingX: 12,
  paddingY: 6,
  radius: 6,
  background: colors.raised,
  borderWidth: 1,
  borderColor: colors.strongBorder,
  transition: 120,
};

export const field: Style = {
  flex: 1,
  paddingX: 10,
  paddingY: 7,
  radius: 6,
  background: colors.window,
  borderWidth: 1,
  borderColor: colors.strongBorder,
};

export const mono: Style = { fontFamily: "mono", fontSize: 12, color: colors.dim };
