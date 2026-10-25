// OKLCH arithmetic for turning the design system's CSS colours into the hex and rgba()
// strings svelte-gpui's style parser reads. Mixing follows CSS Color 4: premultiplied
// by alpha, hues along the shorter arc.

export interface Oklch {
  lightness: number;
  chroma: number;
  /** In degrees. `null` when the colour has none, as `transparent` does. */
  hue: number | null;
  alpha: number;
}

export const transparentColor: Oklch = { lightness: 0, chroma: 0, hue: null, alpha: 0 };

const oklchPattern = /^oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)(?:\s*\/\s*([\d.]+))?\s*\)$/;

export function parseOklch(text: string): Oklch {
  const match = oklchPattern.exec(text.trim());
  if (match === null) {
    throw new Error(`not an oklch() colour: ${text}`);
  }
  let alpha = 1;
  if (match[4] !== undefined) {
    alpha = Number(match[4]);
  }
  return { lightness: Number(match[1]), chroma: Number(match[2]), hue: Number(match[3]), alpha };
}

function mixHue(first: number | null, second: number | null, firstShare: number): number | null {
  if (first === null) {
    return second;
  }
  if (second === null) {
    return first;
  }
  const shorterArc = ((((second - first) % 360) + 540) % 360) - 180;
  return (first + shorterArc * (1 - firstShare) + 360) % 360;
}

/** `color-mix(in oklch, first firstShare, second)`, with `firstShare` between 0 and 1. */
export function mixOklch(first: Oklch, firstShare: number, second: Oklch): Oklch {
  const secondShare = 1 - firstShare;
  const firstWeight = first.alpha * firstShare;
  const secondWeight = second.alpha * secondShare;
  const alpha = firstWeight + secondWeight;
  if (alpha === 0) {
    return transparentColor;
  }
  return {
    lightness: (first.lightness * firstWeight + second.lightness * secondWeight) / alpha,
    chroma: (first.chroma * firstWeight + second.chroma * secondWeight) / alpha,
    hue: mixHue(first.hue, second.hue, firstShare),
    alpha,
  };
}

function hueOrZero(hue: number | null): number {
  if (hue === null) {
    return 0;
  }
  return hue;
}

function encodeGamma(linear: number): number {
  const magnitude = Math.abs(linear);
  if (magnitude <= 0.0031308) {
    return linear * 12.92;
  }
  return Math.sign(linear) * (1.055 * Math.pow(magnitude, 1 / 2.4) - 0.055);
}

type Vector = [number, number, number];

// CSS Color 4: OKLab to LMS, LMS to XYZ (D65), XYZ to linear sRGB.
const lmsToXyz: Vector[] = [
  [1.2268798758459243, -0.5578149944602171, 0.2813910456659647],
  [-0.0405757452148008, 1.112286803280317, -0.0717110580655164],
  [-0.0763729366746601, -0.4214933324022432, 1.5869240198367816],
];
const xyzToLinearSrgb: Vector[] = [
  [3.2409699419045226, -1.537383177570094, -0.4986107602930034],
  [-0.9692436362808796, 1.8759675015077202, 0.041555057407175],
  [0.0556300796969936, -0.2039769588889765, 1.0569715142428786],
];

function multiply(matrix: Vector[], vector: Vector): Vector {
  const [first, second, third] = matrix.map((row) => row[0] * vector[0] + row[1] * vector[1] + row[2] * vector[2]);
  return [first, second, third];
}

/** Gamma-encoded sRGB channels from 0 to 1, outside that range when the colour is out of gamut. */
export function toSrgbChannels(color: Oklch): Vector {
  const hueRadians = (hueOrZero(color.hue) * Math.PI) / 180;
  const a = color.chroma * Math.cos(hueRadians);
  const b = color.chroma * Math.sin(hueRadians);
  const lmsRoot: Vector = [
    color.lightness + 0.3963377773761749 * a + 0.2158037573099136 * b,
    color.lightness - 0.1055613458156586 * a - 0.0638541728258133 * b,
    color.lightness - 0.0894841775298119 * a - 1.2914855480194092 * b,
  ];
  const lms: Vector = [lmsRoot[0] ** 3, lmsRoot[1] ** 3, lmsRoot[2] ** 3];
  const linear = multiply(xyzToLinearSrgb, multiply(lmsToXyz, lms));
  return [encodeGamma(linear[0]), encodeGamma(linear[1]), encodeGamma(linear[2])];
}

function toByte(channel: number): number {
  return Math.round(Math.min(1, Math.max(0, channel)) * 255);
}

/** How far the colour lies outside sRGB, in 8-bit steps of its worst channel. Zero when inside. */
export function gamutOverflow(color: Oklch): number {
  const overflows = toSrgbChannels(color).map((channel) => Math.max(0, -channel, channel - 1) * 255);
  return Math.max(...overflows);
}

function hexByte(value: number): string {
  return value.toString(16).padStart(2, "0");
}

/** `#rrggbb` for an opaque colour, `rgba(r, g, b, a)` otherwise. Out-of-gamut channels are clipped. */
export function toCssColor(color: Oklch): string {
  const [red, green, blue] = toSrgbChannels(color).map(toByte);
  if (color.alpha >= 1) {
    return `#${hexByte(red)}${hexByte(green)}${hexByte(blue)}`;
  }
  return `rgba(${red}, ${green}, ${blue}, ${Number(color.alpha.toFixed(4))})`;
}
