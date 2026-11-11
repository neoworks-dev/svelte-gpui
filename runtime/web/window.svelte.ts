// The app's window in a browser: the tab. Its size, scale and focus are the page's, kept as state the same
// way the native window's are; the title is the document's; fullscreen is the page's. What a page cannot do,
// minimize or maximize itself, it does not pretend to: those requests do nothing, as a window manager is
// free to ignore them natively too. Closing asks the handlers from `onCloseRequested` first.

import { CloseHandlers, type CloseHandler } from "../close-handlers";
import type { window as NativeWindow, WindowOptions } from "../window.svelte";

export type { CloseHandler } from "../close-handlers";
export type { WindowOptions } from "../window.svelte";

const here = typeof globalThis.window === "undefined" ? undefined : globalThis.window;

let size = $state.raw({ width: here?.innerWidth ?? 0, height: here?.innerHeight ?? 0 });
let scale = $state(here?.devicePixelRatio ?? 1);
let focused = $state(typeof document === "undefined" ? true : document.hasFocus());
let fullscreen = $state(false);

const closeHandlers = new CloseHandlers();

if (here !== undefined) {
  here.addEventListener("resize", () => {
    size = { width: here.innerWidth, height: here.innerHeight };
    scale = here.devicePixelRatio;
  });
  here.addEventListener("focus", () => (focused = true));
  here.addEventListener("blur", () => (focused = false));
  document.addEventListener("fullscreenchange", () => (fullscreen = document.fullscreenElement !== null));
  // A page can only ask the browser to keep it, and only for a handler that answers at once.
  here.addEventListener("beforeunload", (event) => {
    if (!closeHandlers.ask(() => here.close())) {
      event.preventDefault();
      event.returnValue = "";
    }
  });
}

/** Applies the options the app opens its window with: the title, and the least size the page keeps. */
export function declareWindow(options: WindowOptions): void {
  if (options.title !== undefined) {
    document.title = options.title;
  }
  const root = document.getElementById("gpui-root");
  if (root !== null) {
    root.style.minWidth = options.minWidth === undefined ? "" : `${options.minWidth}px`;
    root.style.minHeight = options.minHeight === undefined ? "" : `${options.minHeight}px`;
  }
}

export const window: typeof NativeWindow = {
  /** The page's size in logical pixels. */
  get size() {
    return size;
  },
  /** Device pixels per logical pixel. */
  get scale() {
    return scale;
  },
  /** Whether the page has the keyboard focus. */
  get focused() {
    return focused;
  },
  /** A page cannot tell whether its window is maximized. */
  get maximized() {
    return false;
  },
  get fullscreen() {
    return fullscreen;
  },

  /** Sets the page's title. */
  setTitle(title: string): void {
    document.title = title;
  },
  minimize(): void {},
  maximize(): void {},
  unmaximize(): void {},
  toggleMaximize(): void {},
  setFullscreen(on: boolean): void {
    if (on) {
      document.documentElement.requestFullscreen?.().catch(() => {});
    } else if (document.fullscreenElement !== null) {
      document.exitFullscreen().catch(() => {});
    }
  },
  toggleFullscreen(): void {
    window.setFullscreen(document.fullscreenElement === null);
  },

  /** Closes the tab, if the browser lets the page: it only does for a page a script opened. */
  close(): void {
    if (closeHandlers.ask(() => here?.close())) {
      here?.close();
    }
  },
  destroy(): void {
    here?.close();
  },
  onCloseRequested(handler: CloseHandler): () => void {
    return closeHandlers.add(handler);
  },
};
