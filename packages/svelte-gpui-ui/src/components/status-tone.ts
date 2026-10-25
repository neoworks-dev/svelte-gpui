import type { Color } from "@neoworks/svelte-gpui/ui";
import type { Theme } from "../theme";

export type SemanticTone = "success" | "danger" | "warning" | "info" | "neutral";
/** The colour names are the older aliases for `success`, `danger`, `warning`, `info` and `neutral`. */
export type StatusTone = SemanticTone | "green" | "red" | "amber" | "blue" | "violet";

const semanticTones: Record<StatusTone, SemanticTone> = {
  success: "success",
  danger: "danger",
  warning: "warning",
  info: "info",
  neutral: "neutral",
  green: "success",
  red: "danger",
  amber: "warning",
  blue: "info",
  violet: "neutral",
};

export function semanticToneOf(tone: StatusTone): SemanticTone {
  return semanticTones[tone];
}

export interface ToneColors {
  /** The soft fill behind the tone's text or icon. */
  background: Color;
  /** The tone's text and icon colour. */
  color: Color;
  /** The small dot of a status badge. */
  dot: Color;
}

export function toneColors(theme: Theme, tone: StatusTone): ToneColors {
  const { colors } = theme;
  const semanticTone = semanticTones[tone];
  if (semanticTone === "success") {
    return { background: colors.successSoft, color: colors.success, dot: colors.success };
  }
  if (semanticTone === "danger") {
    return { background: colors.dangerSoft, color: colors.danger, dot: colors.danger };
  }
  if (semanticTone === "warning") {
    return { background: colors.warningSoft, color: colors.warning, dot: colors.warning };
  }
  if (semanticTone === "info") {
    return { background: colors.infoSoft, color: colors.info, dot: colors.info };
  }
  return { background: colors.sunken, color: colors.muted, dot: colors.faint };
}
