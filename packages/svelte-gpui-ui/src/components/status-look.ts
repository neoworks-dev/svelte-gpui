import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { toneColors, type StatusTone } from "./status-tone";

export const statusBadgeIconSize = 12;

/** The pill of a status badge, which sits at the start of its parent unless the parent's layout says otherwise. */
export function statusBadgeLook(theme: Theme, tone: StatusTone): Style {
  const colors = toneColors(theme, tone);
  return {
    direction: "row",
    align: "center",
    alignSelf: "start",
    gap: 6,
    height: 24,
    paddingX: 10,
    radius: 12,
    shrink: 0,
    background: colors.background,
    color: colors.color,
  };
}

export function statusBadgeTextLook(): Style {
  return { ...typeStyle("caption"), fontWeight: 500, nowrap: true };
}

export function statusBadgeDotLook(theme: Theme, tone: StatusTone): Style {
  return { size: 6, radius: 3, shrink: 0, background: toneColors(theme, tone).dot };
}

export type IconCircleSize = "sm" | "md" | "lg";

const iconCircleBoxes: Record<IconCircleSize, { box: number; radius: number; icon: number }> = {
  sm: { box: 32, radius: 12, icon: 16 },
  md: { box: 40, radius: 12, icon: 18 },
  lg: { box: 48, radius: 16, icon: 22 },
};

export function iconCircleIconSize(size: IconCircleSize): number {
  return iconCircleBoxes[size].icon;
}

export function iconCircleLook(theme: Theme, tone: StatusTone, size: IconCircleSize): Style {
  const { box, radius } = iconCircleBoxes[size];
  return {
    direction: "row",
    align: "center",
    justify: "center",
    shrink: 0,
    size: box,
    radius,
    background: toneColors(theme, tone).background,
    color: toneColors(theme, tone).color,
  };
}
