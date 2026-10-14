// The renderer's side of `electron`, which Grove's preload script uses to build
// `window.workbench`: `ipcRenderer.invoke` and `.on` carried over the host's socket
// to the main process. Running the preload as it is means the app sees exactly the
// object it sees in Electron.

declare const host: { ipc(line: string): void };
declare const console: { error(...args: unknown[]): void };

type Listener = (event: unknown, ...args: unknown[]) => void;

let nextId = 1;
const pending = new Map<number, { channel: string; resolve(value: unknown): void; reject(error: Error): void }>();
const listeners = new Map<string, Set<Listener>>();

/** JSON with bytes made safe, as the server writes it. */
function encode(value: unknown): string {
  return JSON.stringify(value, (_key, item) => {
    if (item instanceof Uint8Array) {
      let binary = "";
      for (const byte of item) {
        binary += String.fromCharCode(byte);
      }
      return { $bytes: btoa(binary) };
    }
    return item;
  });
}

function decode(line: string): any {
  return JSON.parse(line, (_key, item) => {
    if (item && typeof item === "object" && typeof item.$bytes === "string") {
      const binary = atob(item.$bytes);
      const bytes = new Uint8Array(binary.length);
      for (let index = 0; index < binary.length; index++) {
        bytes[index] = binary.charCodeAt(index);
      }
      return bytes;
    }
    return item;
  });
}

/** Called by the host with each line the main process sends. */
(globalThis as any).__ipc = (line: string): void => {
  const message = decode(line);
  if (typeof message.id === "number") {
    const waiting = pending.get(message.id);
    if (waiting === undefined) {
      return;
    }
    pending.delete(message.id);
    if (message.error !== undefined) {
      console.error(`ipc ${waiting.channel} failed: ${message.error}`);
      waiting.reject(new Error(message.error));
      return;
    }
    waiting.resolve(message.result === null ? undefined : message.result);
    return;
  }
  if (typeof message.event === "string") {
    for (const listener of [...(listeners.get(message.event) ?? [])]) {
      listener({}, ...(message.args ?? []));
    }
  }
};

export const ipcRenderer = {
  invoke(channel: string, ...args: unknown[]): Promise<unknown> {
    const id = nextId++;
    return new Promise((resolve, reject) => {
      pending.set(id, { channel, resolve, reject });
      host.ipc(encode({ id, channel, args }));
    });
  },
  send(channel: string, ...args: unknown[]): void {
    host.ipc(encode({ channel, args }));
  },
  sendSync(): undefined {
    return undefined;
  },
  on(channel: string, listener: Listener): typeof ipcRenderer {
    let set = listeners.get(channel);
    if (set === undefined) {
      set = new Set();
      listeners.set(channel, set);
    }
    set.add(listener);
    return ipcRenderer;
  },
  once(channel: string, listener: Listener): typeof ipcRenderer {
    const wrapped: Listener = (event, ...args) => {
      ipcRenderer.removeListener(channel, wrapped);
      listener(event, ...args);
    };
    return ipcRenderer.on(channel, wrapped);
  },
  removeListener(channel: string, listener: Listener): typeof ipcRenderer {
    listeners.get(channel)?.delete(listener);
    return ipcRenderer;
  },
  removeAllListeners(channel?: string): typeof ipcRenderer {
    if (channel === undefined) {
      listeners.clear();
    } else {
      listeners.delete(channel);
    }
    return ipcRenderer;
  },
};

export const contextBridge = {
  exposeInMainWorld(name: string, api: unknown): void {
    (globalThis as any)[name] = api;
  },
};

export const webUtils = { getPathForFile: (): string => "" };

export const webFrame = {
  setZoomFactor(): void {},
  getZoomFactor: (): number => 1,
  setZoomLevel(): void {},
  getZoomLevel: (): number => 0,
};
