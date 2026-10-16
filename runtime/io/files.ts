// Files, through the host. Every call runs on a thread there and answers with a promise;
// the access limit and the paths' rules are in host/src/io/files.rs.
//
// A path may start with `~` for the home directory. A relative path is resolved against
// the directory the process was started in, which an app cannot rely on; build absolute
// paths from `files.paths()` instead.
//
// Nothing is limited unless the app declares a limit with `start(App, props, { files: {
// allow } })`. Whatever the user picks in a dialog stays allowed regardless.

import { request, stream, toBuffer } from "./channel";

declare const console: { error(...args: unknown[]): void };

/** What a failed file call says went wrong, in `FileError.code`. */
export type FileErrorCode =
  | "NotFound"
  | "PermissionDenied"
  | "NotAllowed"
  | "Exists"
  | "NotEmpty"
  | "NotADirectory"
  | "IsADirectory"
  | "InvalidData"
  | "InvalidArgument"
  | "Other";

/** The error a failed file call rejects with. `message` names the path and the system's reason. */
export class FileError extends Error {
  readonly code: FileErrorCode;

  constructor(code: FileErrorCode, message: string) {
    super(message);
    this.name = "FileError";
    this.code = code;
  }
}

export type FileKind = "file" | "directory" | "symlink";

/** What is known about something on disk. Times are milliseconds since the epoch. */
export interface FileStat {
  kind: FileKind;
  size: number;
  modified: number;
  /** Absent where the system does not record when a file was created. */
  created?: number;
}

/** One entry of a directory listing. `path` is the directory's path with `name` added. */
export interface FileEntry extends FileStat {
  name: string;
  path: string;
}

/** A change a watch saw. A rename shows as a `remove` and a `create`. */
export interface WatchEvent {
  kind: "create" | "modify" | "remove";
  path: string;
}

export interface WatchOptions {
  /** Also watch everything below the directory. */
  recursive?: boolean;
  /** Called when the watch cannot start, for instance for a path that is not allowed. Defaults to `console.error`. */
  onerror?(error: FileError): void;
}

/** Where things live on this machine. All are absolute. */
export interface FilePaths {
  home: string;
  config: string;
  data: string;
  cache: string;
  temp: string;
  /** The directory of the running bundle. */
  bundle: string;
  /** The directory the process was started in. */
  cwd: string;
}

export interface PickFilesOptions {
  /** Let the user choose more than one. */
  multiple?: boolean;
  /** Choose directories instead of files. */
  directories?: boolean;
  /** Text for the dialog's confirm button. */
  prompt?: string;
}

export interface PickSavePathOptions {
  /** The directory the dialog starts in. */
  directory?: string;
  /** The file name it suggests. */
  name?: string;
}

/**
 * The limit an app puts on its own file access. Each entry grants itself and everything
 * below it. `~` and the placeholders `$home`, `$config`, `$data`, `$cache`, `$temp`,
 * `$bundle` and `$cwd` at the start of an entry stand for the directories of `files.paths()`.
 * An empty list allows nothing but what the user picks in a dialog.
 */
export interface FilesLimit {
  allow: string[];
}

/** Turns the host's failure, which carries a `code`, into a `FileError`. */
function failure(error: unknown): Error {
  const code = (error as { code?: unknown } | null)?.code;
  if (!(error instanceof Error) || typeof code !== "string") {
    return error instanceof Error ? error : new Error(String(error));
  }
  return new FileError(code as FileErrorCode, error.message);
}

async function call(op: string, args: unknown, bytes?: ArrayBuffer | ArrayBufferView) {
  try {
    return await request(op, args, bytes === undefined ? undefined : toBuffer(bytes));
  } catch (error) {
    throw failure(error);
  }
}

/**
 * Sets the limit. The host takes it once, before the app mounts (see `start`), and refuses
 * a second one, so code that runs later cannot widen it.
 */
export function limitFiles(limit: FilesLimit): void {
  call("fs.limit", { allow: limit.allow }).catch((error) => console.error(error.message));
}

export const files = {
  /** The text of a file, which has to be UTF-8. */
  async readText(path: string): Promise<string> {
    return (await call("fs.readText", { path })).json.text;
  },

  async readBytes(path: string): Promise<Uint8Array> {
    const reply = await call("fs.readBytes", { path });
    return new Uint8Array(reply.bytes ?? new ArrayBuffer(0));
  },

  /** Replaces the file's content, or creates the file. Its directory has to exist. */
  async writeText(path: string, text: string): Promise<void> {
    await call("fs.writeText", { path, text });
  },

  async writeBytes(path: string, data: ArrayBuffer | ArrayBufferView): Promise<void> {
    await call("fs.writeBytes", { path }, data);
  },

  /** Adds to the end of the file, creating it if it is not there. */
  async appendText(path: string, text: string): Promise<void> {
    await call("fs.appendText", { path, text });
  },

  async exists(path: string): Promise<boolean> {
    return (await call("fs.exists", { path })).json.exists;
  },

  /** Describes the path itself: a symbolic link is reported as `"symlink"`, not as what it points to. */
  async stat(path: string): Promise<FileStat> {
    return (await call("fs.stat", { path })).json;
  },

  /** The entries of a directory, by name. */
  async list(directory: string): Promise<FileEntry[]> {
    return (await call("fs.list", { path: directory })).json.entries;
  },

  /** Makes a directory. With `recursive`, its parents too, and no error if it exists. */
  async mkdir(path: string, options: { recursive?: boolean } = {}): Promise<void> {
    await call("fs.mkdir", { path, recursive: options.recursive === true });
  },

  /** Removes a file, a link, or a directory; a directory with something in it needs `recursive`. */
  async remove(path: string, options: { recursive?: boolean } = {}): Promise<void> {
    await call("fs.remove", { path, recursive: options.recursive === true });
  },

  /** Moves a file or directory. It cannot move between file systems. */
  async rename(from: string, to: string): Promise<void> {
    await call("fs.rename", { from, to });
  },

  /** Copies a file, or a directory with what is in it (links inside it are left out). */
  async copy(from: string, to: string): Promise<void> {
    await call("fs.copy", { from, to });
  },

  /**
   * Calls `callback` for each change under `path` and returns the function that stops
   * watching. The host looks again a few times a second, so changes arrive with a short
   * delay, and one made right after `watch` returns may be missed.
   */
  watch(path: string, callback: (event: WatchEvent) => void, options: WatchOptions = {}): () => void {
    const onerror = options.onerror ?? ((error: Error) => console.error(error.message));
    const watch = stream(
      "fs.watch",
      { path, recursive: options.recursive === true },
      {
        event: (reply) => callback(reply.json),
        close: () => {},
        reject: (error) => onerror(failure(error) as FileError),
      },
    );
    let stopped = false;
    return () => {
      if (!stopped) {
        stopped = true;
        call("fs.unwatch", { watch }).catch(() => {});
      }
    };
  },

  /** Where the system keeps things, and the app's own directory. See `FilePaths`. */
  async paths(): Promise<FilePaths> {
    return (await call("fs.paths", {})).json;
  },

  /** Opens the system's dialog. Resolves with the chosen paths, or `null` if the user cancelled. They are allowed from then on, wherever they are. */
  async pickFiles(options: PickFilesOptions = {}): Promise<string[] | null> {
    return (await call("fs.pickFiles", options)).json.paths;
  },

  /** Opens the system's save dialog. Resolves with the chosen path, or `null` if the user cancelled. It is allowed from then on. */
  async pickSavePath(options: PickSavePathOptions = {}): Promise<string | null> {
    return (await call("fs.pickSavePath", options)).json.path;
  },
};
