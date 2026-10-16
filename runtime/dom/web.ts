// The two web classes Svelte's own modules need when they load: `svelte/reactivity`
// declares `class SvelteURLSearchParams extends URLSearchParams` and a reactive URL
// at import time, and `svelte/motion` imports it. A bare engine has neither.
//
// The network globals (`fetch`, `WebSocket`, `TextEncoder`, ...) are here too: the host
// does the work, and the classes in `runtime/io/` give it the web's shape.

import { AbortController, AbortSignal, DOMException, Headers, Request, Response, fetch } from "../io/fetch";
import { TextDecoder, TextEncoder } from "../io/text";
import { CloseEvent, MessageEvent, WebSocket } from "../io/websocket";

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


define("URL", HostURL);
define("URLSearchParams", HostURLSearchParams);

define("TextEncoder", TextEncoder);
define("TextDecoder", TextDecoder);
define("DOMException", DOMException);
define("AbortController", AbortController);
define("AbortSignal", AbortSignal);
define("Headers", Headers);
define("Request", Request);
define("Response", Response);
define("fetch", fetch);
define("WebSocket", WebSocket);
define("MessageEvent", MessageEvent);
define("CloseEvent", CloseEvent);
