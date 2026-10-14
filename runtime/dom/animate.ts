// `element.animate()`, which is how Svelte plays `transition:`, `in:` and `out:`.
// Svelte samples its easing into one keyframe per frame, so playing an animation
// is stepping through that list against the clock and writing the inline style
// the keyframe names. Nothing is composited: the host just sees style changes.

import { SElement } from "./nodes";

type Keyframe = Record<string, string | number | null | undefined>;

interface Options {
  duration?: number;
  delay?: number;
  fill?: string;
  easing?: string;
}

type Clock = () => number;

let clock: Clock = () => 0;

/** Tells animations where to read the time from. */
export function useClock(next: Clock): void {
  clock = next;
}

const running = new Set<HostAnimation>();

/** `translateY` becomes `translate-y`, as `style.setProperty` wants it. */
function kebab(name: string): string {
  if (name.startsWith("--")) {
    return name;
  }
  return name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
}

/** Svelte passes keyframes as a list of frames; the object form is a list per property. */
function frames(input: Keyframe[] | Record<string, unknown[]>): Keyframe[] {
  if (Array.isArray(input)) {
    return input;
  }
  const names = Object.keys(input);
  const length = Math.max(0, ...names.map((name) => (input[name] as unknown[]).length));
  const out: Keyframe[] = [];
  for (let index = 0; index < length; index++) {
    const frame: Keyframe = {};
    for (const name of names) {
      frame[name] = (input[name] as unknown[])[index] as string;
    }
    out.push(frame);
  }
  return out;
}

class HostAnimation {
  onfinish: (() => void) | null = null;
  oncancel: (() => void) | null = null;
  effect: unknown = {};
  playState: "running" | "paused" | "finished" | "idle" = "running";
  finished: Promise<HostAnimation>;

  private readonly keyframes: Keyframe[];
  private readonly duration: number;
  private readonly delay: number;
  private readonly fill: boolean;
  private startedAt: number;
  private pausedAt = 0;
  /** What the element's inline style held before the animation touched it. */
  private readonly base = new Map<string, string>();
  private resolveFinished: (animation: HostAnimation) => void = () => {};

  constructor(
    private readonly element: SElement,
    keyframes: Keyframe[] | Record<string, unknown[]>,
    options: Options | number,
  ) {
    const settings: Options = typeof options === "number" ? { duration: options } : options;
    this.keyframes = frames(keyframes);
    this.duration = settings.duration ?? 0;
    this.delay = settings.delay ?? 0;
    this.fill = settings.fill === "forwards" || settings.fill === "both";
    this.startedAt = clock();
    this.finished = new Promise((resolve) => {
      this.resolveFinished = resolve;
    });
    running.add(this);
  }

  get currentTime(): number {
    if (this.playState === "paused") {
      return this.pausedAt;
    }
    return clock() - this.startedAt;
  }

  set currentTime(value: number) {
    this.startedAt = clock() - value;
  }

  pause(): void {
    if (this.playState !== "running") {
      return;
    }
    this.pausedAt = this.currentTime;
    this.playState = "paused";
  }

  play(): void {
    if (this.playState === "paused") {
      this.startedAt = clock() - this.pausedAt;
      this.playState = "running";
    }
  }

  /** Jumps to the end: the last keyframe is applied and `onfinish` runs. */
  finish(): void {
    this.apply(this.keyframes.length - 1);
    this.complete();
  }

  /** Stops, and puts back whatever the inline style held before. */
  cancel(): void {
    running.delete(this);
    for (const [name, value] of this.base) {
      this.element.style.setProperty(name, value === "" ? null : value);
    }
    this.base.clear();
    if (this.playState !== "idle") {
      this.playState = "idle";
      this.oncancel?.();
    }
  }

  /** Moves the animation to the clock's time. Called once per host tick. */
  step(): void {
    if (this.playState !== "running") {
      return;
    }
    const elapsed = this.currentTime - this.delay;
    if (elapsed < 0) {
      return;
    }
    if (this.duration <= 0 || elapsed >= this.duration) {
      this.finish();
      return;
    }
    const position = (elapsed / this.duration) * (this.keyframes.length - 1);
    this.apply(Math.floor(position));
  }

  private apply(index: number): void {
    const frame = this.keyframes[Math.max(0, Math.min(index, this.keyframes.length - 1))];
    if (frame === undefined) {
      return;
    }
    for (const [name, value] of Object.entries(frame)) {
      if (value === null || value === undefined || name === "offset" || name === "easing") {
        continue;
      }
      const property = kebab(name);
      if (!this.base.has(property)) {
        this.base.set(property, this.element.style.getPropertyValue(property));
      }
      this.element.style.setProperty(property, String(value));
    }
  }

  private complete(): void {
    running.delete(this);
    this.playState = "finished";
    if (!this.fill) {
      for (const [name, value] of this.base) {
        this.element.style.setProperty(name, value === "" ? null : value);
      }
      this.base.clear();
    }
    this.onfinish?.();
    this.resolveFinished(this);
  }
}

/** Advances every running animation; says whether there were any, so the caller flushes. */
export function stepAnimations(): boolean {
  const had = running.size > 0;
  for (const animation of [...running]) {
    animation.step();
  }
  return had;
}

/** Whether an animation is still going, so the host keeps ticking quickly. */
export function animating(): boolean {
  return running.size > 0;
}

export function installAnimate(): void {
  (SElement.prototype as any).animate = function (
    this: SElement,
    keyframes: Keyframe[] | Record<string, unknown[]>,
    options: Options | number,
  ): HostAnimation {
    return new HostAnimation(this, keyframes, options);
  };
}
