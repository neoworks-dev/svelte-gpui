import type { Style } from "@neoworks/svelte-gpui/ui";
import type { Theme } from "../theme";

export type DividerOrientation = "horizontal" | "vertical";
export type DividerInset = "none" | "start" | "both";

const horizontalMargins: Record<DividerInset, Style> = {
  none: {},
  start: { marginLeft: 64 },
  both: { marginX: 16 },
};

const verticalMargins: Record<DividerInset, Style> = {
  none: {},
  start: { marginTop: 16 },
  both: { marginY: 16 },
};

export interface DividerOptions {
  orientation: DividerOrientation;
  inset: DividerInset;
  strong: boolean;
}

function lineColor(theme: Theme, strong: boolean) {
  if (strong) {
    return theme.colors.lineStrong;
  }
  return theme.colors.line;
}

export function dividerLook(theme: Theme, options: DividerOptions): Style {
  const background = lineColor(theme, options.strong);
  if (options.orientation === "vertical") {
    return { width: 1, alignSelf: "stretch", shrink: 0, background, ...verticalMargins[options.inset] };
  }
  return { height: 1, shrink: 0, background, ...horizontalMargins[options.inset] };
}
