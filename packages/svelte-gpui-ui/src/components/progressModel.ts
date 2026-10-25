// The numbers behind the progress indicator: how full it is, where its indeterminate bar is, and the dots of its ring.

export interface Point {
  x: number;
  y: number;
}

/** The viewBox the circular indicator is specified in, so `thickness` means the same as on the web. */
export const ringViewBox = 48;

/** The share of the ring an indeterminate indicator covers, in percent of the circle. */
export const indeterminateArcPercent = 28;

export const indeterminateBarShare = 0.4;
export const indeterminateBarMilliseconds = 1400;
export const indeterminateRingMilliseconds = 1000;

/** `value` out of `max` as a percentage held between 0 and 100. An empty or missing range reads as 0. */
export function percentOf(value: number | undefined, max: number): number {
  if (value === undefined || max <= 0) {
    return 0;
  }
  return Math.min(100, Math.max(0, (value / max) * 100));
}

/** How far a repeating animation that started at `startedAt` has got through its current round, from 0 up to 1. */
export function cycleFraction(now: number, startedAt: number, periodMilliseconds: number): number {
  return ((now - startedAt) % periodMilliseconds) / periodMilliseconds;
}

/**
 * The left edge of the sliding bar of an indeterminate linear indicator: from just out of sight on the left, to just
 * out of sight on the right, easing in and out. `eased` is the cycle fraction after the easing.
 */
export function indeterminateBarLeft(eased: number, trackWidth: number): number {
  const barWidth = trackWidth * indeterminateBarShare;
  return -barWidth + (trackWidth + barWidth) * eased;
}

export interface RingGeometry {
  /** Pixels from one side of the ring to the other. */
  size: number;
  /** Stroke width in viewBox units out of 48. */
  thickness: number;
}

/** Pixels per viewBox unit. */
function ringScale(geometry: RingGeometry): number {
  return geometry.size / ringViewBox;
}

export function ringStrokeWidth(geometry: RingGeometry): number {
  return geometry.thickness * ringScale(geometry);
}

/** Radius of the ring's centre line, in pixels. */
export function ringRadius(geometry: RingGeometry): number {
  return ((ringViewBox - geometry.thickness) / 2) * ringScale(geometry);
}

/**
 * Centres of the dots that draw an arc of the ring as a round-capped stroke. The arc starts at `startDegrees`, measured
 * clockwise from twelve o'clock, and runs `sweepDegrees` clockwise. Dots are spaced closely enough to overlap into a
 * smooth line.
 */
export function arcDots(geometry: RingGeometry, startDegrees: number, sweepDegrees: number): Point[] {
  if (sweepDegrees <= 0) {
    return [];
  }
  const radius = ringRadius(geometry);
  const centre = geometry.size / 2;
  const spacing = Math.max(0.5, ringStrokeWidth(geometry) / 4);
  const dotCount = Math.max(1, Math.ceil(((sweepDegrees * Math.PI) / 180) * radius / spacing));
  const dots: Point[] = [];
  for (let dotIndex = 0; dotIndex <= dotCount; dotIndex++) {
    const degrees = startDegrees + (sweepDegrees * dotIndex) / dotCount;
    const radians = ((degrees - 90) * Math.PI) / 180;
    dots.push({ x: centre + radius * Math.cos(radians), y: centre + radius * Math.sin(radians) });
  }
  return dots;
}
