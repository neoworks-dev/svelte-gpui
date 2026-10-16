// Order matters: the DOM globals have to exist before Svelte's client runtime
// is evaluated, and this is the module every entry point imports first.
import { pump, wake } from "./dom/globals";

import { flushSync, mount } from "svelte";
import { SElement } from "./dom/nodes";
import { adoptRoot, flush, install } from "./host/bridge";
import { dispatch } from "./host/events";
import { receive } from "./io/channel";
import { limitFiles, type FilesLimit } from "./io/files";

export type { HostEvent } from "./host/events";
export { files, FileError } from "./io/files";
export type {
  FileEntry,
  FileErrorCode,
  FileKind,
  FilePaths,
  FileStat,
  FilesLimit,
  PickFilesOptions,
  PickSavePathOptions,
  WatchEvent,
  WatchOptions,
} from "./io/files";

/** What `start` takes beyond the component and its props. */
export interface StartOptions {
  /** Limits the files the app may reach. Without it, everything is allowed. */
  files?: FilesLimit;
}

/**
 * Mounts a component into the host's root, and exposes the entry points the host
 * calls back into: `__event` for input, `__io` for background calls finishing,
 * `__window` for the window's size, and `__pump` for the clock, which answers when it next has work (see `wake`).
 */
export function start(Component: unknown, props: Record<string, unknown> = {}, options: StartOptions = {}): void {
  // The limit goes first, so nothing the app does while mounting can run without it.
  if (options.files !== undefined) {
    limitFiles(options.files);
  }
  install();
  const root = new SElement("#root");
  adoptRoot(root);

  const settle = (): void => {
    flushSync();
    flush();
  };

  mount(Component as any, { target: root as any, props });
  settle();

  const globals = globalThis as any;
  globals.__settle = settle;
  globals.__event = (id: number, type: string, detail: string): string => {
    const outcome = dispatch(id, type, JSON.parse(detail));
    settle();
    return JSON.stringify(outcome);
  };
  globals.__io = receive;
  globals.__window = (width: number, height: number): void => {
    globals.innerWidth = width;
    globals.innerHeight = height;
    dispatch(0, "resize", {});
    settle();
  };
  globals.__pump = (clock: number, frame: boolean): number => {
    if (pump(clock, frame)) {
      settle();
    }
    return wake();
  };
}

/**
 * Frames the window drew in the last second, and the average time building their
 * elements took. While something moves, frames follow the display's refresh rate;
 * GPUI only draws when something changed, so an idle window reports few.
 */
export function frameStats(): { fps: number; buildMs: number } {
  const [fps, buildMs] = (globalThis as any).host.frameStats() as [number, number];
  return { fps, buildMs };
}

/** The system clipboard, through the host. */
export const clipboard = {
  write(text: string): Promise<void> {
    return (globalThis as any).navigator.clipboard.writeText(text);
  },
  read(): Promise<string> {
    return (globalThis as any).navigator.clipboard.readText();
  },
};
