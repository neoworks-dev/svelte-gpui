// Where the rows of a list sit when their heights differ. Rows whose height is known (measured
// once they were drawn) count as that; every other row counts as the average of the measured
// ones, or `estimate` before any is. Two Fenwick trees keep the sum of the measured heights and
// how many rows are measured, so the top of any row, and the row at any offset, cost a few
// dozen steps however long the list is.

/**
 * Layout rounds sizes to whole pixels in a way that depends on where a row sits, so the same row
 * measures a pixel taller or shorter as it moves. A change smaller than this is that, not a change
 * of the row, and would otherwise move every row below it and set off another measurement.
 */
const NOISE = 1.5;

export class Heights {
  /** The number of rows. */
  length: number;
  /** The height rows count as before any has been measured. */
  readonly estimate: number;
  private measured: Float64Array;
  private sums: Float64Array;
  private counts: Int32Array;
  private measuredCount = 0;
  private measuredSum = 0;

  constructor(length: number, estimate: number) {
    this.length = length;
    this.estimate = Math.max(1, estimate);
    this.measured = new Float64Array(0);
    this.sums = new Float64Array(0);
    this.counts = new Int32Array(0);
    this.build(length, new Float64Array(length));
  }

  /** What an unmeasured row counts as. */
  get average(): number {
    return this.measuredCount === 0 ? this.estimate : this.measuredSum / this.measuredCount;
  }

  /** The total height of every row. */
  get total(): number {
    return this.top(this.length);
  }

  /** Whether row `index` has been measured. */
  isMeasured(index: number): boolean {
    return this.measured[index] > 0;
  }

  /** The height of row `index`, measured or estimated. */
  height(index: number): number {
    const known = this.measured[index];
    return known > 0 ? known : this.average;
  }

  /** The distance from the top of the list to the top of row `index`. `top(length)` is the total. */
  top(index: number): number {
    const clamped = Math.min(Math.max(index, 0), this.length);
    let sum = 0;
    let count = 0;
    for (let at = clamped; at > 0; at -= at & -at) {
      sum += this.sums[at];
      count += this.counts[at];
    }
    return sum + (clamped - count) * this.average;
  }

  /** The row that contains `offset`: the last one whose top is at or above it. */
  indexAt(offset: number): number {
    let low = 0;
    let high = this.length - 1;
    while (low < high) {
      const middle = (low + high + 1) >> 1;
      if (this.top(middle) <= offset) {
        low = middle;
      } else {
        high = middle - 1;
      }
    }
    return Math.max(low, 0);
  }

  /** Records the measured height of row `index`. Returns whether that changed anything. */
  set(index: number, height: number): boolean {
    if (index < 0 || index >= this.length || !(height > 0)) {
      return false;
    }
    const before = this.measured[index];
    if (before > 0 ? Math.abs(before - height) < NOISE : false) {
      return false;
    }
    this.measured[index] = height;
    this.measuredSum += height - before;
    if (before === 0) {
      this.measuredCount++;
      this.add(this.counts, index + 1, 1);
    }
    this.add(this.sums, index + 1, height - before);
    return true;
  }

  /**
   * Replaces what is known: a list of `length` rows whose heights are `heights`, where 0 stands for
   * a row not measured yet.
   */
  replace(length: number, heights: ArrayLike<number>): void {
    this.length = length;
    const values = new Float64Array(length);
    for (let index = 0; index < length; index++) {
      values[index] = heights[index] > 0 ? heights[index] : 0;
    }
    this.build(length, values);
  }

  /**
   * Forgets the height of every row outside `from` up to (not including) `to`, and keeps the rest.
   * This is what a list does when its width changes: rows that are built report their own height
   * again as part of the same change, and what they reported must not be wiped by it.
   */
  forgetOutside(from: number, to: number): void {
    const kept = new Float64Array(this.length);
    for (let index = Math.max(from, 0); index < Math.min(to, this.length); index++) {
      kept[index] = this.measured[index];
    }
    this.build(this.length, kept);
  }

  /** Changes how many rows there are, keeping what is known about the ones that remain. */
  resize(length: number): void {
    if (length === this.length) {
      return;
    }
    if (length < this.length || length > this.measured.length) {
      const kept = new Float64Array(length);
      kept.set(this.measured.subarray(0, Math.min(length, this.length)));
      this.length = length;
      this.build(length, kept, Math.max(length, 1) * (length > this.measured.length ? 2 : 1));
      return;
    }
    // Room is left over from an earlier growth, and unmeasured rows add nothing to the trees.
    this.length = length;
  }

  /** The heights as measured, 0 for rows not measured yet. */
  snapshot(): Float64Array {
    return this.measured.slice(0, this.length);
  }

  private build(length: number, values: Float64Array, capacity = length): void {
    this.measured = new Float64Array(capacity);
    this.measured.set(values.subarray(0, length));
    this.sums = new Float64Array(capacity + 1);
    this.counts = new Int32Array(capacity + 1);
    this.measuredCount = 0;
    this.measuredSum = 0;
    for (let index = 0; index < length; index++) {
      const height = this.measured[index];
      if (height > 0) {
        this.sums[index + 1] = height;
        this.counts[index + 1] = 1;
        this.measuredCount++;
        this.measuredSum += height;
      }
    }
    // Each node passes what it holds up to the one that covers it, which builds the trees in one pass.
    for (let at = 1; at <= capacity; at++) {
      const parent = at + (at & -at);
      if (parent <= capacity) {
        this.sums[parent] += this.sums[at];
        this.counts[parent] += this.counts[at];
      }
    }
  }

  private add(tree: Float64Array | Int32Array, at: number, delta: number): void {
    for (; at < tree.length; at += at & -at) {
      tree[at] += delta;
    }
  }
}
