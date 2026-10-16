// `fetch` and the classes around it: `Headers`, `Request`, `Response`, `AbortController`,
// `AbortSignal` and `DOMException`. The host makes the request on a thread (see
// host/src/io/fetch.rs).
//
// Bodies are buffered: the host reads a response to its end before the promise
// resolves, and `Response.body` (a stream) does not exist, only the readers `text()`,
// `json()`, `arrayBuffer()` and `bytes()`. There is no `Blob` or `FormData` either.

import { request, toBuffer } from "./channel";
import { NetEvent, NetEventTarget } from "./events";
import { TextDecoder, TextEncoder } from "./text";

/** A string or a `URL`: anything that prints as the address. */
export type UrlInput = string | { href: string };

declare function setTimeout(callback: () => void, milliseconds: number): number;

const LEGACY_CODES: Record<string, number> = {
  IndexSizeError: 1,
  InvalidStateError: 11,
  SyntaxError: 12,
  NotSupportedError: 9,
  AbortError: 20,
  TimeoutError: 23,
  DataCloneError: 25,
};

/** Stands in when the engine has no `DOMException` of its own. */
class HostDOMException extends Error {
  readonly code: number;

  constructor(message = "", name = "Error") {
    super(message);
    Object.defineProperty(this, "name", { value: name, configurable: true, writable: true });
    this.code = LEGACY_CODES[name] ?? 0;
  }
}

/** The error `fetch` rejects with when aborted, and `WebSocket.send` throws while connecting. */
export const DOMException: typeof HostDOMException = (globalThis as any).DOMException ?? HostDOMException;

/** Raises an `AbortSignal` when `abort()` is called. */
export class AbortSignal extends NetEventTarget {
  aborted = false;
  reason: unknown = undefined;
  onabort: ((event: NetEvent) => void) | null = null;

  throwIfAborted(): void {
    if (this.aborted) {
      throw this.reason;
    }
  }

  /** @internal Called by `AbortController`; the signal cannot be aborted by anyone else. */
  _abort(reason: unknown): void {
    if (this.aborted) {
      return;
    }
    this.aborted = true;
    this.reason = reason === undefined ? new DOMException("This operation was aborted", "AbortError") : reason;
    const event = new NetEvent("abort");
    this.onabort?.call(this, event);
    this.dispatchEvent(event);
  }

  static abort(reason?: unknown): AbortSignal {
    const signal = new AbortSignal();
    signal._abort(reason);
    return signal;
  }

  static timeout(milliseconds: number): AbortSignal {
    const signal = new AbortSignal();
    setTimeout(() => signal._abort(new DOMException("The operation timed out", "TimeoutError")), milliseconds);
    return signal;
  }

  static any(signals: AbortSignal[]): AbortSignal {
    const combined = new AbortSignal();
    for (const signal of signals) {
      if (signal.aborted) {
        combined._abort(signal.reason);
        break;
      }
      signal.addEventListener("abort", () => combined._abort(signal.reason), { once: true });
    }
    return combined;
  }
}

export class AbortController {
  readonly signal = new AbortSignal();

  abort(reason?: unknown): void {
    this.signal._abort(reason);
  }
}

const TOKEN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;

function headerName(name: unknown): string {
  const text = String(name);
  if (!TOKEN.test(text)) {
    throw new TypeError(`Invalid header name: ${text}`);
  }
  return text.toLowerCase();
}

function headerValue(value: unknown): string {
  return String(value).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");
}

export type HeadersInit = Headers | [string, string][] | Record<string, string>;

/** HTTP headers: names are case-insensitive, and a repeated name joins its values with a comma. */
export class Headers {
  /** Values by lower-cased name, in insertion order of the names. */
  private readonly byName = new Map<string, string[]>();

  constructor(init?: HeadersInit | null) {
    if (init === undefined || init === null) {
      return;
    }
    if (init instanceof Headers) {
      init.forEach((value, name) => this.append(name, value));
    } else if (Array.isArray(init) || typeof (init as any)[Symbol.iterator] === "function") {
      for (const pair of init as Iterable<[string, string]>) {
        if (pair.length !== 2) {
          throw new TypeError("Each header must be a [name, value] pair");
        }
        this.append(pair[0], pair[1]);
      }
    } else {
      for (const [name, value] of Object.entries(init)) {
        this.append(name, value);
      }
    }
  }

  append(name: string, value: string): void {
    const key = headerName(name);
    const list = this.byName.get(key) ?? [];
    list.push(headerValue(value));
    this.byName.set(key, list);
  }

  delete(name: string): void {
    this.byName.delete(headerName(name));
  }

  get(name: string): string | null {
    const list = this.byName.get(headerName(name));
    return list === undefined ? null : list.join(", ");
  }

  getSetCookie(): string[] {
    return [...(this.byName.get("set-cookie") ?? [])];
  }

  has(name: string): boolean {
    return this.byName.has(headerName(name));
  }

  set(name: string, value: string): void {
    this.byName.set(headerName(name), [headerValue(value)]);
  }

  /** Name and value pairs, sorted by name, the way browsers list them. `set-cookie` stays one per cookie. */
  entries(): IterableIterator<[string, string]> {
    const pairs: [string, string][] = [];
    for (const name of [...this.byName.keys()].sort()) {
      const list = this.byName.get(name)!;
      if (name === "set-cookie") {
        pairs.push(...list.map((value): [string, string] => [name, value]));
      } else {
        pairs.push([name, list.join(", ")]);
      }
    }
    return pairs[Symbol.iterator]();
  }

  keys(): IterableIterator<string> {
    return [...this.entries()].map(([name]) => name)[Symbol.iterator]();
  }

  values(): IterableIterator<string> {
    return [...this.entries()].map(([, value]) => value)[Symbol.iterator]();
  }

  forEach(callback: (value: string, name: string, headers: Headers) => void, thisArg?: unknown): void {
    for (const [name, value] of this.entries()) {
      callback.call(thisArg, value, name, this);
    }
  }

  [Symbol.iterator](): IterableIterator<[string, string]> {
    return this.entries();
  }
}

export type BodyInit = string | ArrayBuffer | ArrayBufferView | { toString(): string } | null;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** A body as bytes, and the content type that goes with it when the caller gave none. */
function bodyBytes(body: unknown): { bytes: Uint8Array; type: string | null } | null {
  if (body === null || body === undefined) {
    return null;
  }
  if (typeof body === "string") {
    return { bytes: encoder.encode(body), type: "text/plain;charset=UTF-8" };
  }
  if (body instanceof ArrayBuffer) {
    return { bytes: new Uint8Array(body.slice(0)), type: null };
  }
  if (ArrayBuffer.isView(body)) {
    return { bytes: new Uint8Array(toBuffer(body).slice(0)), type: null };
  }
  const globals = globalThis as any;
  if (globals.URLSearchParams !== undefined && body instanceof globals.URLSearchParams) {
    return { bytes: encoder.encode(body.toString()), type: "application/x-www-form-urlencoded;charset=UTF-8" };
  }
  if (typeof body === "object" && ("stream" in body || "getAll" in body)) {
    throw new TypeError("Blob and FormData bodies are not supported");
  }
  return { bytes: encoder.encode(String(body)), type: "text/plain;charset=UTF-8" };
}

/** What `Request` and `Response` share: a body that can be read once. */
class Body {
  protected content: Uint8Array | null;
  private used = false;

  constructor(content: Uint8Array | null) {
    this.content = content;
  }

  get bodyUsed(): boolean {
    return this.used;
  }

  /** Takes the body, which can only happen once. */
  protected consume(): Uint8Array {
    if (this.used) {
      throw new TypeError("Body has already been consumed");
    }
    this.used = true;
    return this.content ?? new Uint8Array(0);
  }

  /** @internal The bytes without using the body up, for `clone()`. */
  protected peek(): Uint8Array | null {
    if (this.used) {
      throw new TypeError("Cannot clone: body has already been consumed");
    }
    return this.content === null ? null : this.content.slice();
  }

  async bytes(): Promise<Uint8Array> {
    return this.consume();
  }

  async arrayBuffer(): Promise<ArrayBuffer> {
    const bytes = this.consume();
    const whole = bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength;
    return (whole ? bytes.buffer : bytes.slice().buffer) as ArrayBuffer;
  }

  async text(): Promise<string> {
    return decoder.decode(this.consume());
  }

  async json(): Promise<any> {
    return JSON.parse(await this.text());
  }
}

export interface ResponseInit {
  status?: number;
  statusText?: string;
  headers?: HeadersInit;
}

export class Response extends Body {
  readonly headers: Headers;
  readonly status: number;
  readonly statusText: string;
  readonly url: string = "";
  readonly redirected: boolean = false;
  readonly type: string = "default";

  constructor(body?: BodyInit, init: ResponseInit = {}) {
    const converted = bodyBytes(body);
    super(converted?.bytes ?? null);
    this.status = init.status ?? 200;
    if (this.status < 200 || this.status > 599) {
      throw new RangeError(`The status provided (${this.status}) is outside the range [200, 599].`);
    }
    this.statusText = init.statusText ?? "";
    this.headers = new Headers(init.headers);
    if (converted?.type != null && !this.headers.has("content-type")) {
      this.headers.set("content-type", converted.type);
    }
  }

  get ok(): boolean {
    return this.status >= 200 && this.status <= 299;
  }

  clone(): Response {
    const copy = new Response(null, { status: this.status, statusText: this.statusText, headers: this.headers });
    copy.content = this.peek();
    (copy as any).url = this.url;
    (copy as any).redirected = this.redirected;
    (copy as any).type = this.type;
    return copy;
  }

  static error(): Response {
    const response = new Response(null, { status: 200 });
    (response as any).status = 0;
    (response as any).type = "error";
    return response;
  }

  static json(data: unknown, init: ResponseInit = {}): Response {
    const response = new Response(JSON.stringify(data), init);
    if (!new Headers(init.headers).has("content-type")) {
      response.headers.set("content-type", "application/json");
    }
    return response;
  }
}

export interface RequestInit {
  method?: string;
  headers?: HeadersInit;
  body?: BodyInit;
  signal?: AbortSignal | null;
  redirect?: "follow" | "manual" | "error";
}

/** Methods the standard wants upper-cased; any other method is sent as given. */
const NORMALIZED_METHODS = new Set(["DELETE", "GET", "HEAD", "OPTIONS", "POST", "PUT"]);

export class Request extends Body {
  readonly url: string;
  readonly method: string;
  readonly headers: Headers;
  readonly signal: AbortSignal;
  readonly redirect: "follow" | "manual" | "error";

  constructor(input: UrlInput | Request, init: RequestInit = {}) {
    const source = input instanceof Request ? input : null;
    const converted = init.body !== undefined ? bodyBytes(init.body) : null;
    super(converted?.bytes ?? source?.peekForCopy() ?? null);
    this.url = source !== null ? source.url : requestUrl(input as UrlInput);
    const method = (init.method ?? source?.method ?? "GET").trim();
    const upper = method.toUpperCase();
    this.method = NORMALIZED_METHODS.has(upper) ? upper : method;
    if (!TOKEN.test(this.method)) {
      throw new TypeError(`Invalid method: ${method}`);
    }
    this.headers = new Headers(init.headers ?? source?.headers);
    this.redirect = init.redirect ?? source?.redirect ?? "follow";
    this.signal = init.signal ?? source?.signal ?? new AbortSignal();
    if (this.content !== null && (this.method === "GET" || this.method === "HEAD")) {
      throw new TypeError("Request with GET/HEAD method cannot have a body");
    }
    if (converted?.type != null && !this.headers.has("content-type")) {
      this.headers.set("content-type", converted.type);
    }
  }

  /** @internal */
  peekForCopy(): Uint8Array | null {
    return this.peek();
  }

  clone(): Request {
    return new Request(this);
  }

  /** @internal The body, taken for sending. */
  takeBody(): Uint8Array | null {
    const body = this.content;
    this.consume();
    return body;
  }
}

function requestUrl(input: UrlInput): string {
  const text = String(input).trim().replace(/#.*$/, "");
  if (!/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(text)) {
    throw new TypeError(`Failed to parse URL from ${text}: there is no base to resolve it against`);
  }
  return text;
}

interface FetchHead {
  url: string;
  redirected: boolean;
  status: number;
  statusText: string;
  headers: [string, string][];
}

/** Makes an HTTP request on a host thread. Network failures reject with a `TypeError`. */
export async function fetch(input: UrlInput | Request, init?: RequestInit): Promise<Response> {
  const made = new Request(input, init);
  made.signal.throwIfAborted();
  if (input instanceof Request) {
    input.takeBody();
  }
  const body = made.takeBody();
  const headers: [string, string][] = [...made.headers].map(([name, value]) => [name, value]);
  const call = request(
    "fetch",
    { url: made.url, method: made.method, headers, redirect: made.redirect },
    body === null ? undefined : toBuffer(body),
  );
  // An abort rejects at once; the host finishes the transfer and the answer is dropped.
  const reply = await new Promise<Awaited<typeof call>>((resolve, reject) => {
    const onAbort = () => reject(made.signal.reason);
    made.signal.addEventListener("abort", onAbort, { once: true });
    call.then(
      (value) => {
        made.signal.removeEventListener("abort", onAbort);
        resolve(value);
      },
      (error: Error) => {
        made.signal.removeEventListener("abort", onAbort);
        reject(new TypeError(`Failed to fetch: ${error.message}`));
      },
    );
  });
  const head = reply.json as FetchHead;
  if (made.redirect === "error" && head.status >= 300 && head.status < 400) {
    throw new TypeError(`Failed to fetch ${made.url}: a redirect was received and redirects are not followed`);
  }
  return responseFrom(head, reply.bytes);
}

function responseFrom(head: FetchHead, bytes: ArrayBuffer | undefined): Response {
  const response = new Response(bytes ?? null, { statusText: head.statusText, headers: head.headers });
  Object.assign(response, { status: head.status, url: head.url, redirected: head.redirected, type: "basic" });
  return response;
}
