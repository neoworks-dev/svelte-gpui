// `@neoworks/svelte-gpui` for the browser, which package.json names under the `browser` condition. The same
// names and types as `runtime/index.ts`, so one app builds for both targets: `start` mounts the app in the
// page, and `window` is the tab. `files`, `invoke` and `listen` are the native implementations as they are:
// they send their calls to the host over the page's connection to it (see `./host.ts`), where the same Rust
// answers them, so a command or a file behaves as it does in the native window.

import { flushSync, mount } from "svelte";
import { files, limitFiles } from "../io/files";
import { rootElement } from "./ui/base";
import { connectToHost } from "./host";
import { declareWindow } from "./window.svelte";

export type { HostEvent } from "../host/events";
export { window } from "./window.svelte";
export type { CloseHandler, WindowOptions } from "./window.svelte";
export { element } from "./element";
export type { ElementMethods, ElementProps } from "../ui/element-of";
export { files, FileError } from "../io/files";
export { invoke, invokeBytes, listen } from "../io/invoke";
export type { CommandMap, EventMap, InvokeBytes, InvokeResult } from "../io/invoke";
export type {
  FileEntry,
  FileErrorCode,
  FileFilter,
  FileKind,
  FilePaths,
  FileStat,
  FilesLimit,
  PickFilesOptions,
  PickSavePathOptions,
  WatchEvent,
  WatchOptions,
} from "../io/files";
export type { StartOptions } from "../index";

import type { StartOptions } from "../index";

/** Which target the app is running on, for the few things that differ: `"native"` in the host's window, `"browser"` here. */
export const target: "native" | "browser" = "browser";

/**
 * Mounts a component into the page, which it fills, and connects the page to the host it was served by.
 * `options.window.title` is the page's title, and its `minWidth` and `minHeight` are the least the page
 * lays out at, scrolling below that. `options.files` limits file access, as natively, in the host.
 */
export function start(Component: unknown, props: Record<string, unknown> = {}, options: StartOptions = {}): void {
  const connected = connectToHost();
  // The limit goes first, so nothing the app does while mounting can run without it.
  if (options.files !== undefined && connected) {
    limitFiles(options.files);
  }
  const root = rootElement();
  if (options.window !== undefined) {
    declareWindow(options.window);
  }
  mount(Component as any, { target: root, props });
  flushSync();
}

let frames: { stamps: number[]; running: boolean } | undefined;

/**
 * Frames the page drew in the last second. A browser draws at the display's rate, so unlike the native window it
 * does not drop to nothing when idle. `buildMs` is always 0: the page's own work is not measured separately.
 */
export function frameStats(): { fps: number; buildMs: number } {
  if (frames === undefined) {
    const state = { stamps: [] as number[], running: true };
    frames = state;
    const tick = (now: number): void => {
      state.stamps.push(now);
      while (state.stamps.length > 0 && now - state.stamps[0] > 1000) {
        state.stamps.shift();
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }
  return { fps: frames.stamps.length, buildMs: 0 };
}

/** The clipboard of the browser the page is in. Reading and writing it needs the page to have focus, and the user's permission. */
export const clipboard = {
  write(text: string): Promise<void> {
    return navigator.clipboard.writeText(text);
  },
  read(): Promise<string> {
    return navigator.clipboard.readText();
  },
  /**
   * The image the clipboard holds, as a file the host wrote it to (the host's temporary directory), or null when it
   * holds none, or the host cannot write it. A browser gives a page the image itself, not a path, so it is handed to
   * the host to write, which is what makes it a file the app can open as it does natively.
   */
  async readImage(): Promise<{ path: string; mime: string } | null> {
    try {
      for (const item of await navigator.clipboard.read()) {
        const mime = item.types.find((type) => type.startsWith("image/"));
        if (mime === undefined) {
          continue;
        }
        const bytes = new Uint8Array(await (await item.getType(mime)).arrayBuffer());
        const { temp } = await files.paths();
        const path = `${temp}/svelte-gpui-clipboard-${Date.now()}.${mime.slice("image/".length).replace(/\+.*/, "")}`;
        await files.writeBytes(path, bytes);
        return { path, mime };
      }
    } catch {
      // Without permission, or a host that will not write there, there is no image to give.
    }
    return null;
  },
};
