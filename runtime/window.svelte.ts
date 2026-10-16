// The app's window: what it is like, and what the app may ask of it. Requests go to the
// host, which carries them out on its next tick; what the window is like comes back through
// `receive`, which the host calls when anything changed.

declare const console: { error(...args: unknown[]): void };

/** How the window opens, in `start(App, props, { window })`. The host's `--width` and `--height` override the size. */
export interface WindowOptions {
  /** The window's title. */
  title?: string;
  /** The content's size in logical pixels. The default is 668 by 821. */
  width?: number;
  height?: number;
  /** The smallest size the user can resize the window to. It can only be set when the window opens. */
  minWidth?: number;
  minHeight?: number;
}

/**
 * Decides whether the window may close. Return `false`, or a promise that resolves to
 * `false`, to keep it open; anything else lets it close.
 */
export type CloseHandler = () => boolean | void | Promise<boolean | void>;

interface HostWindow {
  window(operation: string, argumentsJson: string): void;
  windowOptions(optionsJson: string): void;
}

const host = (): HostWindow => (globalThis as any).host;

let size = $state.raw({ width: 0, height: 0 });
let scale = $state(1);
let focused = $state(true);
let maximized = $state(false);
let fullscreen = $state(false);

const closeHandlers = new Set<CloseHandler>();
let asking = false;

function request(operation: string, args: Record<string, unknown> = {}): void {
  host().window(operation, JSON.stringify(args));
}

/** Hands the host the options the app opens its window with. Called by `start`. */
export function declareWindow(options: WindowOptions): void {
  host().windowOptions(JSON.stringify(options));
}

/** Takes what the host reports about the window. Returns whether the size changed. */
export function receive(
  width: number,
  height: number,
  newScale: number,
  isFocused: boolean,
  isMaximized: boolean,
  isFullscreen: boolean,
): boolean {
  const resized = width !== size.width || height !== size.height;
  if (resized) {
    size = { width, height };
  }
  scale = newScale;
  focused = isFocused;
  maximized = isMaximized;
  fullscreen = isFullscreen;
  return resized;
}

function isThenable(value: unknown): value is PromiseLike<unknown> {
  return typeof (value as { then?: unknown } | null | undefined)?.then === "function";
}

/**
 * Asks the close handlers. Answers whether the window may close now. When a handler answers
 * later, this answers `false` and closes the window itself once every handler has said yes.
 */
export function askToClose(): boolean {
  if (asking) {
    return false;
  }
  const later: PromiseLike<unknown>[] = [];
  for (const handler of [...closeHandlers]) {
    let answer;
    try {
      answer = handler();
    } catch (error) {
      console.error("a close handler threw:", error);
      continue;
    }
    if (answer === false) {
      return false;
    }
    if (isThenable(answer)) {
      later.push(answer);
    }
  }
  if (later.length === 0) {
    return true;
  }
  asking = true;
  Promise.all(later).then(
    (answers) => {
      asking = false;
      if (answers.every((answer) => answer !== false)) {
        request("destroy");
      }
    },
    (error) => {
      asking = false;
      console.error("a close handler failed:", error);
    },
  );
  return false;
}

/**
 * The app's window. `size`, `scale`, `focused`, `maximized` and `fullscreen` are reactive,
 * so a component or `$effect` that reads one runs again when it changes. They hold their
 * defaults until the window has drawn its first frame.
 *
 * Requests take effect on the next frame, and the window manager has the last word: it may
 * ignore `maximize` or `minimize`. There is no `setSize`, `setPosition` or `center`: GPUI
 * 0.2.2 on Wayland cannot resize a mapped window (its `resize` only changes GPUI's idea of
 * the size, and the window draws nothing until the compositor sends its own), and
 * Wayland gives a client no say over where its window goes.
 */
export const window = {
  /** The content's size in logical pixels. */
  get size(): { readonly width: number; readonly height: number } {
    return size;
  },
  /** Device pixels per logical pixel. */
  get scale(): number {
    return scale;
  },
  /** Whether the window has the keyboard focus. */
  get focused(): boolean {
    return focused;
  },
  get maximized(): boolean {
    return maximized;
  },
  get fullscreen(): boolean {
    return fullscreen;
  },

  /** Sets the title the window manager shows. */
  setTitle(title: string): void {
    request("title", { title });
  },
  minimize(): void {
    request("minimize");
  },
  maximize(): void {
    request("maximize");
  },
  unmaximize(): void {
    request("unmaximize");
  },
  toggleMaximize(): void {
    request("toggleMaximize");
  },
  setFullscreen(on: boolean): void {
    request("fullscreen", { on });
  },
  toggleFullscreen(): void {
    request("toggleFullscreen");
  },

  /**
   * Closes the window, the way the close button does: the handlers from `onCloseRequested`
   * are asked first, and any of them can keep it open.
   */
  close(): void {
    if (askToClose()) {
      request("destroy");
    }
  },
  /** Closes the window without asking the handlers, which ends the app. */
  destroy(): void {
    request("destroy");
  },
  /**
   * Calls `handler` whenever the window is asked to close, by the close button, the window
   * manager or `close()`. Return `false`, or a promise resolving to `false`, to keep it
   * open; with a promise the window stays open meanwhile, and closes once it resolves to
   * anything else. Returns a function that removes the handler.
   */
  onCloseRequested(handler: CloseHandler): () => void {
    closeHandlers.add(handler);
    return () => {
      closeHandlers.delete(handler);
    };
  },
};
