declare const console: { log(...args: unknown[]): void; error(...args: unknown[]): void };
// The web platform pieces an app reaches for that a bare engine does not have.
// Each is the smallest thing that behaves for what real code does with it: parse
// a URL, encode a string, clone an object, cancel a request. Anything that needs
// a network or a clock the host has not offered rejects, so the app sees a normal
// failure rather than a missing name.

const globals = globalThis as any;

function define(name: string, value: unknown): void {
  if (globals[name] === undefined) {
    globals[name] = value;
  }
}

class HostURLSearchParams {
  private readonly entries: [string, string][] = [];

  constructor(init: string | Record<string, string> | [string, string][] | HostURLSearchParams = "") {
    if (typeof init === "string") {
      const text = init.startsWith("?") ? init.slice(1) : init;
      for (const part of text.split("&")) {
        if (part === "") {
          continue;
        }
        const [key, ...rest] = part.split("=");
        this.entries.push([decode(key), decode(rest.join("="))]);
      }
      return;
    }
    if (init instanceof HostURLSearchParams) {
      this.entries.push(...init.entries);
      return;
    }
    const pairs = Array.isArray(init) ? init : Object.entries(init);
    for (const [key, value] of pairs) {
      this.entries.push([String(key), String(value)]);
    }
  }

  append(key: string, value: string): void {
    this.entries.push([key, String(value)]);
  }

  delete(key: string): void {
    for (let index = this.entries.length - 1; index >= 0; index--) {
      if (this.entries[index][0] === key) {
        this.entries.splice(index, 1);
      }
    }
  }

  get(key: string): string | null {
    return this.entries.find(([name]) => name === key)?.[1] ?? null;
  }

  getAll(key: string): string[] {
    return this.entries.filter(([name]) => name === key).map(([, value]) => value);
  }

  has(key: string): boolean {
    return this.entries.some(([name]) => name === key);
  }

  set(key: string, value: string): void {
    const first = this.entries.findIndex(([name]) => name === key);
    if (first === -1) {
      this.entries.push([key, String(value)]);
      return;
    }
    this.entries[first][1] = String(value);
    this.delete(key);
    this.entries.splice(first, 0, [key, String(value)]);
  }

  forEach(callback: (value: string, key: string) => void): void {
    for (const [key, value] of this.entries) {
      callback(value, key);
    }
  }

  keys(): IterableIterator<string> {
    return this.entries.map(([key]) => key)[Symbol.iterator]();
  }

  values(): IterableIterator<string> {
    return this.entries.map(([, value]) => value)[Symbol.iterator]();
  }

  [Symbol.iterator](): IterableIterator<[string, string]> {
    return this.entries[Symbol.iterator]();
  }

  toString(): string {
    return this.entries.map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`).join("&");
  }
}

function decode(text: string): string {
  try {
    return decodeURIComponent(text.replace(/\+/g, " "));
  } catch {
    return text;
  }
}

class HostURL {
  protocol = "";
  username = "";
  password = "";
  hostname = "";
  port = "";
  pathname = "/";
  hash = "";
  searchParams: HostURLSearchParams;

  constructor(input: string, base?: string | HostURL) {
    let text = String(input);
    const absolute = /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(text);
    if (!absolute) {
      // `new URL("./x", import.meta.url)` has no base in a bundle; files are relative to the root.
      const parent = new HostURL(base === undefined || base === "" ? "file:///" : String(base));
      text = text.startsWith("/")
        ? `${parent.origin}${text}`
        : `${parent.origin}${parent.pathname.replace(/[^/]*$/, "")}${text}`;
    }
    const match = /^([a-zA-Z][a-zA-Z0-9+.-]*:)(?:\/\/(?:([^:@/]*)(?::([^@/]*))?@)?([^:/?#]*)(?::(\d+))?)?([^?#]*)(\?[^#]*)?(#.*)?$/.exec(text);
    if (match === null) {
      throw new TypeError(`Invalid URL: ${text}`);
    }
    this.protocol = match[1];
    this.username = match[2] ?? "";
    this.password = match[3] ?? "";
    this.hostname = match[4] ?? "";
    this.port = match[5] ?? "";
    this.pathname = match[6] === "" ? "/" : match[6];
    this.searchParams = new HostURLSearchParams(match[7] ?? "");
    this.hash = match[8] ?? "";
  }

  get host(): string {
    return this.port === "" ? this.hostname : `${this.hostname}:${this.port}`;
  }

  get origin(): string {
    return `${this.protocol}//${this.host}`;
  }

  get search(): string {
    const text = this.searchParams.toString();
    return text === "" ? "" : `?${text}`;
  }

  get href(): string {
    return `${this.origin}${this.pathname}${this.search}${this.hash}`;
  }

  toString(): string {
    return this.href;
  }

  toJSON(): string {
    return this.href;
  }

  static canParse(input: string, base?: string): boolean {
    try {
      new HostURL(input, base);
      return true;
    } catch {
      return false;
    }
  }
}

class HostTextEncoder {
  readonly encoding = "utf-8";

  encode(text = ""): Uint8Array {
    const bytes: number[] = [];
    for (const character of text) {
      const point = character.codePointAt(0) as number;
      if (point < 0x80) {
        bytes.push(point);
      } else if (point < 0x800) {
        bytes.push(0xc0 | (point >> 6), 0x80 | (point & 0x3f));
      } else if (point < 0x10000) {
        bytes.push(0xe0 | (point >> 12), 0x80 | ((point >> 6) & 0x3f), 0x80 | (point & 0x3f));
      } else {
        bytes.push(0xf0 | (point >> 18), 0x80 | ((point >> 12) & 0x3f), 0x80 | ((point >> 6) & 0x3f), 0x80 | (point & 0x3f));
      }
    }
    return new Uint8Array(bytes);
  }
}

class HostTextDecoder {
  readonly encoding = "utf-8";

  decode(input?: ArrayBuffer | ArrayBufferView): string {
    if (input === undefined) {
      return "";
    }
    const bytes = input instanceof ArrayBuffer ? new Uint8Array(input) : new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
    let out = "";
    for (let index = 0; index < bytes.length; ) {
      const first = bytes[index++];
      let point = first;
      if (first >= 0xf0) {
        point = ((first & 0x07) << 18) | ((bytes[index++] & 0x3f) << 12) | ((bytes[index++] & 0x3f) << 6) | (bytes[index++] & 0x3f);
      } else if (first >= 0xe0) {
        point = ((first & 0x0f) << 12) | ((bytes[index++] & 0x3f) << 6) | (bytes[index++] & 0x3f);
      } else if (first >= 0xc0) {
        point = ((first & 0x1f) << 6) | (bytes[index++] & 0x3f);
      }
      out += String.fromCodePoint(point);
    }
    return out;
  }
}

const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function btoaImpl(text: string): string {
  let out = "";
  for (let index = 0; index < text.length; index += 3) {
    const a = text.charCodeAt(index);
    const b = text.charCodeAt(index + 1);
    const c = text.charCodeAt(index + 2);
    out += BASE64[a >> 2] + BASE64[((a & 3) << 4) | ((b || 0) >> 4)];
    out += Number.isNaN(b) ? "=" : BASE64[((b & 15) << 2) | ((c || 0) >> 6)];
    out += Number.isNaN(c) ? "=" : BASE64[c & 63];
  }
  return out;
}

function atobImpl(text: string): string {
  let out = "";
  const clean = text.replace(/[^A-Za-z0-9+/]/g, "");
  for (let index = 0; index < clean.length; index += 4) {
    const chunk = [0, 1, 2, 3].map((offset) => BASE64.indexOf(clean[index + offset] ?? "A"));
    out += String.fromCharCode((chunk[0] << 2) | (chunk[1] >> 4));
    if (index + 2 < clean.length) {
      out += String.fromCharCode(((chunk[1] & 15) << 4) | (chunk[2] >> 2));
    }
    if (index + 3 < clean.length) {
      out += String.fromCharCode(((chunk[2] & 3) << 6) | chunk[3]);
    }
  }
  return out;
}

function clone<T>(value: T): T {
  if (value === null || typeof value !== "object") {
    return value;
  }
  if (value instanceof Date) {
    return new Date(value.getTime()) as T;
  }
  if (value instanceof Map) {
    return new Map([...value].map(([key, entry]) => [clone(key), clone(entry)])) as T;
  }
  if (value instanceof Set) {
    return new Set([...value].map(clone)) as T;
  }
  if (Array.isArray(value)) {
    return value.map(clone) as T;
  }
  const copy: Record<string, unknown> = {};
  for (const [key, entry] of Object.entries(value)) {
    copy[key] = clone(entry);
  }
  return copy as T;
}

class HostAbortSignal {
  aborted = false;
  reason: unknown = undefined;
  onabort: (() => void) | null = null;
  private readonly listeners = new Set<() => void>();

  addEventListener(_type: string, listener: () => void): void {
    this.listeners.add(listener);
  }

  removeEventListener(_type: string, listener: () => void): void {
    this.listeners.delete(listener);
  }

  throwIfAborted(): void {
    if (this.aborted) {
      throw this.reason;
    }
  }

  fire(reason: unknown): void {
    if (this.aborted) {
      return;
    }
    this.aborted = true;
    this.reason = reason ?? new Error("AbortError");
    this.onabort?.();
    for (const listener of [...this.listeners]) {
      listener();
    }
  }

  static abort(reason?: unknown): HostAbortSignal {
    const signal = new HostAbortSignal();
    signal.fire(reason);
    return signal;
  }

  static timeout(): HostAbortSignal {
    return new HostAbortSignal();
  }
}

class HostAbortController {
  readonly signal = new HostAbortSignal();

  abort(reason?: unknown): void {
    this.signal.fire(reason);
  }
}

define("URL", HostURL);
define("URLSearchParams", HostURLSearchParams);
define("TextEncoder", HostTextEncoder);
define("TextDecoder", HostTextDecoder);
define("atob", atobImpl);
define("btoa", btoaImpl);
define("structuredClone", clone);
define("AbortController", HostAbortController);
define("AbortSignal", HostAbortSignal);
define("fetch", () => Promise.reject(new TypeError("fetch is not available")));
define("requestIdleCallback", (callback: (deadline: unknown) => void) =>
  globals.setTimeout(() => callback({ didTimeout: false, timeRemaining: () => 8 }), 1),
);
define("cancelIdleCallback", (handle: number) => globals.clearTimeout(handle));
define("crypto", {
  randomUUID: () =>
    "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (letter) => {
      const random = (Math.random() * 16) | 0;
      return (letter === "x" ? random : (random & 3) | 8).toString(16);
    }),
  getRandomValues: <T extends ArrayBufferView>(array: T): T => {
    const bytes = new Uint8Array(array.buffer, array.byteOffset, array.byteLength);
    for (let index = 0; index < bytes.length; index++) {
      bytes[index] = (Math.random() * 256) | 0;
    }
    return array;
  },
});

/** `EventTarget` for objects the app makes itself: a store, a worker stand-in, a cancel signal. */
class HostEventTarget {
  private readonly handlers = new Map<string, Set<(event: any) => void>>();

  addEventListener(type: string, handler: (event: any) => void): void {
    let set = this.handlers.get(type);
    if (set === undefined) {
      set = new Set();
      this.handlers.set(type, set);
    }
    set.add(handler);
  }

  removeEventListener(type: string, handler: (event: any) => void): void {
    this.handlers.get(type)?.delete(handler);
  }

  dispatchEvent(event: any): boolean {
    event.target = this;
    event.currentTarget = this;
    for (const handler of [...(this.handlers.get(event.type) ?? [])]) {
      handler.call(this, event);
    }
    return !event.defaultPrevented;
  }
}

define("EventTarget", HostEventTarget);

// Code written for Node or Electron's preload reads `process`; the pieces it reads are here.
define("process", {
  env: {} as Record<string, string>,
  platform: "linux",
  arch: "x64",
  versions: { node: "22.0.0" },
  contextIsolated: false,
  argv: [] as string[],
  cwd: () => "/",
  nextTick: (callback: (...args: unknown[]) => void, ...args: unknown[]) =>
    globals.queueMicrotask(() => callback(...args)),
  stdout: { write: (text: string) => console.log(text) },
  stderr: { write: (text: string) => console.error(text) },
});

/** `localStorage` and `sessionStorage`, kept for the life of the window. */
class MemoryStorage {
  private readonly items = new Map<string, string>();

  get length(): number {
    return this.items.size;
  }

  getItem(key: string): string | null {
    return this.items.get(String(key)) ?? null;
  }

  setItem(key: string, value: unknown): void {
    this.items.set(String(key), String(value));
  }

  removeItem(key: string): void {
    this.items.delete(String(key));
  }

  clear(): void {
    this.items.clear();
  }

  key(index: number): string | null {
    return [...this.items.keys()][index] ?? null;
  }
}

define("localStorage", new MemoryStorage());
define("sessionStorage", new MemoryStorage());

/** A `Worker` that never runs anything: sandboxed plugins need a real one, and for now there is none. */
class HostWorker {
  onmessage: ((event: unknown) => void) | null = null;
  onerror: ((event: unknown) => void) | null = null;
  postMessage(): void {}
  terminate(): void {}
  addEventListener(): void {}
  removeEventListener(): void {}
}

define("Worker", HostWorker);
