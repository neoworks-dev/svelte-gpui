import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";

export function sectionHeaderLook(): Style {
  return { direction: "row", align: "end", justify: "between", gap: 16, paddingBottom: 12 };
}

export function sectionHeaderTitleLook(theme: Theme): Style {
  return { ...typeStyle("title"), color: theme.colors.text };
}

export function sectionHeaderDescriptionLook(theme: Theme): Style {
  return { ...typeStyle("bodySmall"), color: theme.colors.muted, marginTop: 2 };
}

export function sectionHeaderActionLook(theme: Theme): Style {
  return { ...typeStyle("bodySmall"), direction: "row", align: "center", shrink: 0, color: theme.colors.text };
}
