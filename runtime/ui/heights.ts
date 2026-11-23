// Where the rows of a list sit when their heights differ. A row counts as its estimate, given when
// it joins the list, until it is measured once it was drawn, and then as what it measured. So
// measuring a row moves only the rows under it, by what it differed from its estimate, and never the
// ones that have not been drawn. A Fenwick tree keeps the sum of the heights, so the top of any row,
// and the row at any offset, cost a few dozen steps however long the list is.

/**
 * Layout rounds sizes to whole pixels in a way that depends on where a row sits, so the same row
 * measures a pixel taller or shorter as it moves. A change smaller than this is that, not a change
 * of the row, and would otherwise move every row below it and set off another measurement.
 */
const NOISE = 1.5;

/** The estimate of row `index`, for the rows that join the list. */
export type Guess = (index: number) => number;

export class Heights {
  /** The number of rows. */
  length: number;
  /** What a row counts as when no `Guess` is given for it. */
  readonly estimate: number;
  private guesses: Float64Array;
  private measured: Float64Array;
  private sums: Float64Array;

  constructor(length: number, estimate: number, guess?: Guess) {
    this.length = length;
    this.estimate = Math.max(1, estimate);
    this.guesses = new Float64Array(0);
    this.measured = new Float64Array(0);
    this.sums = new Float64Array(0);
    this.build(length, new Float64Array(length), this.guessesFor(0, length, guess));
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
    return known > 0 ? known : this.guesses[index];
  }

  /** The distance from the top of the list to the top of row `index`. `top(length)` is the total. */
  top(index: number): number {
    const clamped = Math.min(Math.max(index, 0), this.length);
    let sum = 0;
    for (let at = clamped; at > 0; at -= at & -at) {
      sum += this.sums[at];
    }
    return sum;
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
    if (before > 0 && Math.abs(before - height) < NOISE) {
      return false;
    }
    const counted = this.height(index);
    this.measured[index] = height;
    this.add(index + 1, height - counted);
    return true;
  }

  /**
   * Replaces what is known: a list of `length` rows whose heights are `heights`, where 0 stands for
   * a row not measured yet, and which counts as `guess` says until it is.
   */
  replace(length: number, heights: ArrayLike<number>, guess?: Guess): void {
    this.length = length;
    const values = new Float64Array(length);
    for (let index = 0; index < length; index++) {
      values[index] = heights[index] > 0 ? heights[index] : 0;
    }
    this.build(length, values, this.guessesFor(0, length, guess));
  }

  /**
   * Forgets the height of every row outside `from` up to (not including) `to`, and keeps the rest;
   * the rows forgotten count as their estimates again. This is what a list does when its width
   * changes: rows that are built report their own height again as part of the same change, and what
   * they reported must not be wiped by it.
   */
  forgetOutside(from: number, to: number): void {
    const kept = new Float64Array(this.length);
    for (let index = Math.max(from, 0); index < Math.min(to, this.length); index++) {
      kept[index] = this.measured[index];
    }
    this.build(this.length, kept, this.guesses.slice(0, this.length));
  }

  /**
   * Changes how many rows there are, keeping what is known about the ones that remain. Rows that join
   * count as `guess` says.
   */
  resize(length: number, guess?: Guess): void {
    if (length === this.length) {
      return;
    }
    const before = this.length;
    if (length < before || length > this.measured.length) {
      const kept = new Float64Array(length);
      kept.set(this.measured.subarray(0, Math.min(length, before)));
      const guesses = new Float64Array(length);
      guesses.set(this.guesses.subarray(0, Math.min(length, before)));
      if (length > before) {
        guesses.set(this.guessesFor(before, length, guess), before);
      }
      this.length = length;
      this.build(length, kept, guesses, Math.max(length, 1) * (length > this.measured.length ? 2 : 1));
      return;
    }
    // Room is left over from an earlier growth: the new rows are added to the tree one by one.
    const added = this.guessesFor(before, length, guess);
    this.length = length;
    for (let index = before; index < length; index++) {
      this.measured[index] = 0;
      this.guesses[index] = added[index - before];
      this.add(index + 1, this.guesses[index]);
    }
  }

  /** The heights as measured, 0 for rows not measured yet. */
  snapshot(): Float64Array {
    return this.measured.slice(0, this.length);
  }

  /** The estimates of rows `from` up to `to`, each at least a pixel. */
  private guessesFor(from: number, to: number, guess?: Guess): Float64Array {
    const out = new Float64Array(Math.max(to - from, 0));
    for (let index = from; index < to; index++) {
      const value = guess ? guess(index) : this.estimate;
      out[index - from] = Number.isFinite(value) && value >= 1 ? value : this.estimate;
    }
    return out;
  }

  private build(length: number, values: Float64Array, guesses: Float64Array, capacity = length): void {
    this.measured = new Float64Array(capacity);
    this.measured.set(values.subarray(0, length));
    this.guesses = new Float64Array(capacity);
    this.guesses.set(guesses.subarray(0, length));
    this.sums = new Float64Array(capacity + 1);
    for (let index = 0; index < length; index++) {
      this.sums[index + 1] = this.height(index);
    }
    // Each node passes what it holds up to the one that covers it, which builds the tree in one pass.
    for (let at = 1; at <= capacity; at++) {
      const parent = at + (at & -at);
      if (parent <= capacity) {
        this.sums[parent] += this.sums[at];
      }
    }
  }

  private add(at: number, delta: number): void {
    for (; at < this.sums.length; at += at & -at) {
      this.sums[at] += delta;
    }
  }
}
