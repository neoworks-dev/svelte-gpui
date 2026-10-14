// A stand-in for the parts of `electron` that Grove's main process uses, so the
// main process runs as a plain Node program with no window: the handlers it
// registers are served over a socket, and what it pushes to its window goes to
// whoever is connected. Only what the main process reaches for is here: `app`,
// `ipcMain`, `BrowserWindow`, `dialog`, `shell`, `protocol`, `safeStorage` and
// `clipboard`.

import { EventEmitter } from "node:events";
import { spawn, spawnSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";

type Handler = (event: unknown, ...args: unknown[]) => unknown;

/** Channels the app can `invoke`, and the listeners it can `send` to. */
export const invokeHandlers = new Map<string, Handler>();
export const sendListeners = new Map<string, Handler[]>();

/** Everything connected, told when the main process pushes an event. */
export const subscribers = new Set<(channel: string, args: unknown[]) => void>();

export function broadcast(channel: string, args: unknown[]): void {
  for (const subscriber of subscribers) {
    subscriber(channel, args);
  }
}

export const ipcMain = {
  handle(channel: string, handler: Handler): void {
    invokeHandlers.set(channel, handler);
  },
  handleOnce(channel: string, handler: Handler): void {
    invokeHandlers.set(channel, handler);
  },
  removeHandler(channel: string): void {
    invokeHandlers.delete(channel);
  },
  on(channel: string, listener: Handler): void {
    sendListeners.set(channel, [...(sendListeners.get(channel) ?? []), listener]);
  },
  once(channel: string, listener: Handler): void {
    ipcMain.on(channel, listener);
  },
  removeListener(channel: string, listener: Handler): void {
    sendListeners.set(
      channel,
      (sendListeners.get(channel) ?? []).filter((candidate) => candidate !== listener),
    );
  },
  removeAllListeners(channel?: string): void {
    if (channel === undefined) {
      sendListeners.clear();
      return;
    }
    sendListeners.delete(channel);
  },
};

class WebContents extends EventEmitter {
  id = 1;

  send(channel: string, ...args: unknown[]): void {
    broadcast(channel, args);
  }

  isDestroyed(): boolean {
    return false;
  }

  executeJavaScript(): Promise<unknown> {
    return Promise.resolve(undefined);
  }

  setWindowOpenHandler(): void {}
  setZoomFactor(): void {}
  getZoomFactor(): number {
    return 1;
  }
  openDevTools(): void {}
  isFocused(): boolean {
    return true;
  }
}

const webContents = new WebContents();
// The window that is never shown: the main process looks for one to talk to.
setImmediate(() => void new BrowserWindow());

export class BrowserWindow extends EventEmitter {
  webContents = webContents;
  static windows: BrowserWindow[] = [];

  constructor() {
    super();
    BrowserWindow.windows.push(this);
  }

  static getAllWindows(): BrowserWindow[] {
    return BrowserWindow.windows;
  }

  static fromWebContents(): BrowserWindow | null {
    return BrowserWindow.windows[0] ?? null;
  }

  static fromId(): BrowserWindow | null {
    return BrowserWindow.windows[0] ?? null;
  }

  static getFocusedWindow(): BrowserWindow | null {
    return BrowserWindow.windows[0] ?? null;
  }

  isDestroyed(): boolean {
    return false;
  }
  isMinimized(): boolean {
    return false;
  }
  isFullScreen(): boolean {
    return false;
  }
  isFocused(): boolean {
    return true;
  }
  isVisible(): boolean {
    return true;
  }
  show(): void {}
  focus(): void {}
  restore(): void {}
  minimize(): void {}
  close(): void {}
  setFullScreen(): void {}
  removeMenu(): void {}
  loadURL(): Promise<void> {
    return Promise.resolve();
  }
  loadFile(): Promise<void> {
    return Promise.resolve();
  }
  getBounds() {
    return { x: 0, y: 0, width: 1440, height: 900 };
  }
}

const appName = "grove";
const profile =
  process.env.GROVE_GPUI_PROFILE ?? join(homedir(), ".config", "grove-gpui");
mkdirSync(profile, { recursive: true });

class App extends EventEmitter {
  isPackaged = false;

  getPath(name: string): string {
    switch (name) {
      case "userData":
      case "appData":
      case "sessionData":
      case "logs":
        return profile;
      case "temp":
        return tmpdir();
      case "home":
        return homedir();
      case "downloads":
        return join(homedir(), "Downloads");
      case "documents":
        return join(homedir(), "Documents");
      default:
        return profile;
    }
  }

  getAppPath(): string {
    return process.env.GROVE_ROOT ?? process.cwd();
  }

  getVersion(): string {
    return "1.0.0";
  }

  getName(): string {
    return appName;
  }

  setName(): void {}
  setAppUserModelId(): void {}
  whenReady(): Promise<void> {
    return Promise.resolve();
  }
  requestSingleInstanceLock(): boolean {
    return true;
  }
  quit(): void {
    process.exit(0);
  }
  exit(code = 0): void {
    process.exit(code);
  }
  isReady(): boolean {
    return true;
  }
  commandLine = { appendSwitch(): void {} };
}

export const app = new App();

export const shell = {
  openExternal(url: string): Promise<void> {
    spawn("xdg-open", [url], { detached: true, stdio: "ignore" }).unref();
    return Promise.resolve();
  },
  openPath(path: string): Promise<string> {
    spawn("xdg-open", [path], { detached: true, stdio: "ignore" }).unref();
    return Promise.resolve("");
  },
  showItemInFolder(): void {},
};

// There is no window to ask in, so a dialog answers as if it were dismissed. The
// shell can answer these itself once it has dialogs of its own.
export const dialog = {
  showMessageBox: async () => ({ response: 0, checkboxChecked: false }),
  showOpenDialog: async () => ({ canceled: true, filePaths: [] as string[] }),
  showSaveDialog: async () => ({ canceled: true, filePath: undefined }),
  showErrorBox: (): void => {},
};

export const protocol = {
  registerSchemesAsPrivileged(): void {},
  handle(): void {},
  unhandle(): void {},
  registerFileProtocol(): void {},
};

// Without a keychain the main process falls back to what it does when encryption
// is unavailable.
export const safeStorage = {
  isEncryptionAvailable: (): boolean => false,
  encryptString: (text: string): Buffer => Buffer.from(text),
  decryptString: (data: Buffer): string => data.toString(),
};

let memoryClipboard = "";

export const clipboard = {
  readText(): string {
    const read = spawnSync("wl-paste", ["--no-newline"], { encoding: "utf8" });
    if (read.status === 0) {
      return read.stdout;
    }
    return memoryClipboard;
  },
  writeText(text: string): void {
    memoryClipboard = text;
    const write = spawn("wl-copy", [], { stdio: ["pipe", "ignore", "ignore"] });
    write.on("error", () => {});
    write.stdin.on("error", () => {});
    write.stdin.end(text);
  },
};

export type IpcMainInvokeEvent = { sender: WebContents };
