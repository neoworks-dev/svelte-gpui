// What every component's look is made of: the style objects a primitive takes for each of its states.

import type { Color, Shadow, Style, Transition } from "@neoworks/svelte-gpui/ui";
import type { Theme } from "../theme";

/** The styles a primitive takes: `style` at rest, and the ones laid over it while hovered, pressed, focused or disabled. */
export interface StateStyles {
  style: Style;
  hover?: Style;
  active?: Style;
  focus?: Style;
  disabledStyle?: Style;
}

export const stateTransition: Transition = { duration: 300, easing: "ease-out" };

export const disabledOpacity = 0.4;

/** The pressed look of a solid fill, which the web draws by scaling the element down to 96%. */
export const pressedSolidOpacity = 0.88;

/** The host draws no shadow whose blur is 0, so a hard-edged ring is given the smallest blur that shows. */
const smallestVisibleBlur = 1;

/** A ring `width` pixels wide around a box. */
export function ring(width: number, color: Color): Shadow {
  return { x: 0, y: 0, blur: smallestVisibleBlur, spread: width, color };
}

/** A ring in the focus colour, standing in for the web's outline, which `Style` has no equivalent of. */
export function focusRing(theme: Theme): Shadow {
  return ring(2, theme.colors.focus);
}

export interface Appearance {
  /** The box paints its own background, which a ring shadow can then sit behind. */
  hasBackground: boolean;
  hasBorder: boolean;
}

/**
 * Focus for a box. A shadow also fills a box with no background of its own, so only a box that has one gets the ring;
 * one with a border recolours it, and one with neither takes the hover wash.
 */
export function focusStyle(theme: Theme, appearance: Appearance, shadowsBelow: Shadow[] = []): Style {
  if (appearance.hasBackground) {
    return { shadow: [...shadowsBelow, focusRing(theme)] };
  }
  if (appearance.hasBorder) {
    return { borderColor: theme.colors.focus, background: theme.colors.hover };
  }
  return { background: theme.colors.hover, color: theme.colors.text };
}

/** The theme's shadows with every blur-0 layer, such as the overlay's 1px outline, made visible to the host. */
export function visibleShadows(shadows: Shadow[]): Shadow[] {
  return shadows.map((shadow) => {
    if (shadow.blur === 0) {
      return { ...shadow, blur: smallestVisibleBlur };
    }
    return shadow;
  });
}

/** A radius that makes the shorter side of a box a half circle. */
export function pillRadius(height: number): number {
  return height / 2;
}

function parseHex(color: Color): [number, number, number] {
  if (!/^#[0-9a-f]{6}$/i.test(color)) {
    throw new Error(`only #rrggbb colours can be blended: ${color}`);
  }
  return [1, 3, 5].map((start) => parseInt(color.slice(start, start + 2), 16)) as [number, number, number];
}

function hexByte(value: number): string {
  return Math.round(value).toString(16).padStart(2, "0");
}

/** `overlay` laid over `base` at `overlayShare` of full strength, as the web's faint state layer over a background. */
export function blendColors(base: Color, overlay: Color, overlayShare: number): Color {
  const baseChannels = parseHex(base);
  const overlayChannels = parseHex(overlay);
  const blended = baseChannels.map((channel, index) => channel + (overlayChannels[index] - channel) * overlayShare);
  return `#${blended.map(hexByte).join("")}`;
}

/** The hover and pressed backgrounds of a surface that washes with the text colour, as Card and list rows do. */
export function stateLayerBackgrounds(theme: Theme, background: Color): { hover: Color; active: Color } {
  return {
    hover: blendColors(background, theme.colors.text, 0.04),
    active: blendColors(background, theme.colors.text, 0.08),
  };
}

/** A 1px border in `color`, or none when there is no colour. */
export function borderStyle(color: Color | undefined): Style {
  if (color === undefined) {
    return {};
  }
  return { borderWidth: 1, borderColor: color };
}

/** How a component whose root is a wrapper sits in its parent: centred across it unless the caller says otherwise. */
export function wrapperAlignment(style: Style | undefined): Style {
  if (style === undefined || style.alignSelf === undefined) {
    return { alignSelf: "center" };
  }
  return { alignSelf: style.alignSelf };
}
