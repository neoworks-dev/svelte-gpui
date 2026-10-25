import type { Color, Shadow, Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { focusRing, stateLayerBackgrounds, stateTransition, visibleShadows, type StateStyles } from "./look";

export type CardVariant = "outlined" | "filled" | "elevated";
export type CardPadding = "none" | "sm" | "md" | "lg";

const paddings: Record<CardPadding, number> = { none: 0, sm: 16, md: 20, lg: 24 };

export function cardPadding(padding: CardPadding): number {
  return paddings[padding];
}

function cardBackground(theme: Theme, variant: CardVariant): Color {
  if (variant === "filled") {
    return theme.colors.sunken;
  }
  return theme.colors.surface;
}

function variantStyle(theme: Theme, variant: CardVariant): Style {
  const background = cardBackground(theme, variant);
  if (variant === "outlined") {
    return { background, borderWidth: 1, borderColor: theme.colors.line };
  }
  if (variant === "elevated") {
    return { background, shadow: theme.shadows.lift };
  }
  return { background };
}

export interface CardOptions {
  variant: CardVariant;
  interactive: boolean;
}

function interactiveStates(theme: Theme, options: CardOptions): Omit<StateStyles, "style"> {
  const layers = stateLayerBackgrounds(theme, cardBackground(theme, options.variant));
  const hover: Style = { background: layers.hover };
  if (options.variant === "elevated") {
    hover.shadow = visibleShadows(theme.shadows.overlay);
  }
  return {
    hover,
    active: { background: layers.active },
    focus: { shadow: [...variantShadows(theme, options.variant), focusRing(theme)] },
  };
}

function variantShadows(theme: Theme, variant: CardVariant): Shadow[] {
  if (variant === "elevated") {
    return theme.shadows.lift;
  }
  return [];
}

/** The card's box. A card that can be pressed also washes with the text colour on hover and press. */
export function cardLook(theme: Theme, options: CardOptions): StateStyles {
  const style: Style = {
    direction: "column",
    overflow: "hidden",
    radius: theme.radii.card,
    color: theme.colors.text,
    ...variantStyle(theme, options.variant),
  };
  if (!options.interactive) {
    return { style };
  }
  return { style: { ...style, cursor: "pointer", transition: stateTransition }, ...interactiveStates(theme, options) };
}

export interface CardTextOptions {
  hasHeadline: boolean;
  hasSubhead: boolean;
}

export function headlineLook(theme: Theme, viewportWidth: number): Style {
  return { ...typeStyle("heading", viewportWidth), color: theme.colors.text };
}

export function subheadLook(theme: Theme, options: CardTextOptions): Style {
  const title = typeStyle("title");
  if (options.hasHeadline) {
    return { ...title, color: theme.colors.muted, marginTop: 4 };
  }
  return { ...title, color: theme.colors.text };
}

export function supportingTextLook(theme: Theme, options: CardTextOptions): Style {
  return { ...typeStyle("body"), color: theme.colors.muted, marginTop: supportingTextSpacing(options) };
}

function supportingTextSpacing(options: CardTextOptions): number {
  if (options.hasHeadline) {
    return 16;
  }
  if (options.hasSubhead) {
    return 4;
  }
  return 0;
}
