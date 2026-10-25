export interface SliderScale {
  minimum: number;
  maximum: number;
  step: number;
}

/** Where `current` sits between the ends of the scale, from 0 to 1. */
export function fractionOf(scale: SliderScale, current: number): number {
  if (scale.maximum <= scale.minimum) {
    return 0;
  }
  const fraction = (current - scale.minimum) / (scale.maximum - scale.minimum);
  return Math.min(1, Math.max(0, fraction));
}

function stepDecimals(step: number): number {
  const fraction = String(step).split(".")[1];
  if (fraction === undefined) {
    return 0;
  }
  return fraction.length;
}

/** The nearest multiple of the step, counted from the minimum and kept inside the scale. */
export function snap(scale: SliderScale, raw: number): number {
  const stepped = scale.minimum + Math.round((raw - scale.minimum) / scale.step) * scale.step;
  const rounded = Number(stepped.toFixed(stepDecimals(scale.step)));
  return Math.min(scale.maximum, Math.max(scale.minimum, rounded));
}

/** The value under a pointer `offset` pixels along a track `trackWidth` pixels wide. */
export function valueAtOffset(scale: SliderScale, offset: number, trackWidth: number): number {
  if (trackWidth <= 0) {
    return scale.minimum;
  }
  const ratio = Math.min(1, Math.max(0, offset / trackWidth));
  return snap(scale, scale.minimum + ratio * (scale.maximum - scale.minimum));
}

/** Which thumb a press at `target` should move. A tie goes to the thumb the press lies beyond. */
export function nearestThumbIndex(target: number, range: boolean, rangeValue: [number, number]): number {
  if (!range) {
    return 0;
  }
  const lowDistance = Math.abs(target - rangeValue[0]);
  const highDistance = Math.abs(target - rangeValue[1]);
  if (lowDistance < highDistance) {
    return 0;
  }
  if (highDistance < lowDistance) {
    return 1;
  }
  if (target < rangeValue[0]) {
    return 0;
  }
  return 1;
}

/** The range after moving one thumb to `requested`, which cannot cross the other thumb. */
export function moveRangeThumb(rangeValue: [number, number], index: number, requested: number): [number, number] {
  if (index === 0) {
    return [Math.min(requested, rangeValue[1]), rangeValue[1]];
  }
  return [rangeValue[0], Math.max(requested, rangeValue[0])];
}

/** The value a key moves a thumb to, or undefined when the key does nothing. Shift moves ten steps at once. */
export function valueAfterKey(scale: SliderScale, key: string, current: number, shift: boolean): number | undefined {
  let amount = scale.step;
  if (shift) {
    amount = scale.step * 10;
  }
  if (key === "ArrowRight" || key === "ArrowUp") {
    return snap(scale, current + amount);
  }
  if (key === "ArrowLeft" || key === "ArrowDown") {
    return snap(scale, current - amount);
  }
  if (key === "PageUp") {
    return snap(scale, current + scale.step * 10);
  }
  if (key === "PageDown") {
    return snap(scale, current - scale.step * 10);
  }
  if (key === "Home") {
    return scale.minimum;
  }
  if (key === "End") {
    return scale.maximum;
  }
  return undefined;
}

const maximumTickCount = 40;

/** Positions of the tick marks from 0 to 1, none when there would be too many to read. */
export function tickFractions(scale: SliderScale): number[] {
  if (scale.step <= 0) {
    return [];
  }
  const count = Math.round((scale.maximum - scale.minimum) / scale.step);
  if (count < 1 || count > maximumTickCount) {
    return [];
  }
  return Array.from({ length: count + 1 }, (_, index) => index / count);
}
