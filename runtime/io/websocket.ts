// `WebSocket`, backed by a thread in the host (see host/src/io/websocket.rs). The
// socket is one open stream: the call number of `ws.open` names it, and `ws.send` and
// `ws.close` are separate requests that carry that number.
//
// `binaryType` is `"arraybuffer"`, the default here, and `"blob"` is refused because
// there is no `Blob`. `bufferedAmount` is always 0: a message goes to the host's
// thread as soon as `send` is called.

import { stream, request, toBuffer, type Reply } from "./channel";
import { DOMException } from "./fetch";
import { NetEvent, NetEventTarget } from "./events";
import { TextEncoder } from "./text";

const CONNECTING = 0;
const OPEN = 1;
const CLOSING = 2;
const CLOSED = 3;

/** The longest reason a close frame can carry, in UTF-8 bytes. */
const MAX_REASON_BYTES = 123;

export class MessageEvent extends NetEvent {
  readonly data: string | ArrayBuffer;
  readonly origin: string;
  readonly lastEventId = "";
  readonly source = null;
  readonly ports: unknown[] = [];

  constructor(type: string, init: { data?: string | ArrayBuffer; origin?: string } = {}) {
    super(type);
    this.data = init.data ?? "";
    this.origin = init.origin ?? "";
  }
}

export class CloseEvent extends NetEvent {
  readonly code: number;
  readonly reason: string;
  readonly wasClean: boolean;

  constructor(type: string, init: { code?: number; reason?: string; wasClean?: boolean } = {}) {
    super(type);
    this.code = init.code ?? 0;
    this.reason = init.reason ?? "";
    this.wasClean = init.wasClean ?? false;
  }
}

type Handler<Event> = ((this: WebSocket, event: Event) => void) | null;

export class WebSocket extends NetEventTarget {
  static readonly CONNECTING = CONNECTING;
  static readonly OPEN = OPEN;
  static readonly CLOSING = CLOSING;
  static readonly CLOSED = CLOSED;
  readonly CONNECTING = CONNECTING;
  readonly OPEN = OPEN;
  readonly CLOSING = CLOSING;
  readonly CLOSED = CLOSED;

  readonly url: string;
  readyState = CONNECTING;
  protocol = "";
  extensions = "";
  bufferedAmount = 0;

  onopen: Handler<NetEvent> = null;
  onmessage: Handler<MessageEvent> = null;
  onerror: Handler<NetEvent> = null;
  onclose: Handler<CloseEvent> = null;

  private kind: "arraybuffer" = "arraybuffer";
  private socket: number;

  constructor(url: string | { href: string }, protocols?: string | string[]) {
    super();
    this.url = socketUrl(url);
    const list = protocols === undefined ? [] : typeof protocols === "string" ? [protocols] : [...protocols];
    if (new Set(list).size !== list.length) {
      throw new DOMException("The subprotocols must all be unique", "SyntaxError");
    }
    this.socket = stream(
      "ws.open",
      { url: this.url, protocols: list },
      {
        event: (reply) => this.happened(reply),
        close: (reply) => this.closed(reply.json),
        // The host could not even start the socket.
        reject: (error) => {
          this.happened({ json: { type: "error", message: error.message } });
          this.closed({ code: 1006, reason: "", wasClean: false });
        },
      },
    );
  }

  get binaryType(): "arraybuffer" | "blob" {
    return this.kind;
  }

  set binaryType(value: "arraybuffer" | "blob") {
    if (value === "blob") {
      throw new DOMException("binaryType 'blob' is not supported: there is no Blob; use 'arraybuffer'", "NotSupportedError");
    }
  }

  send(data: string | ArrayBuffer | ArrayBufferView): void {
    if (this.readyState === CONNECTING) {
      throw new DOMException("Still in CONNECTING state", "InvalidStateError");
    }
    if (this.readyState !== OPEN) {
      return;
    }
    if (data instanceof ArrayBuffer || ArrayBuffer.isView(data)) {
      this.ask("ws.send", { socket: this.socket }, toBuffer(data));
    } else {
      this.ask("ws.send", { socket: this.socket, text: String(data) });
    }
  }

  close(code?: number, reason?: string): void {
    if (code !== undefined && code !== 1000 && !(code >= 3000 && code <= 4999)) {
      throw new DOMException(`The code must be either 1000, or between 3000 and 4999. ${code} is neither.`, "InvalidAccessError");
    }
    if (reason !== undefined && new TextEncoder().encode(reason).length > MAX_REASON_BYTES) {
      throw new DOMException("The message must not be greater than 123 bytes.", "SyntaxError");
    }
    if (this.readyState === CLOSING || this.readyState === CLOSED) {
      return;
    }
    this.readyState = CLOSING;
    this.ask("ws.close", { socket: this.socket, code, reason });
  }

  /** Sends a request to the socket's thread. Nothing waits for it: failures show up as events. */
  private ask(op: string, args: unknown, bytes?: ArrayBuffer): void {
    request(op, args, bytes).catch(() => {});
  }

  private happened(reply: Pick<Reply, "json" | "bytes">): void {
    const { json } = reply;
    switch (json.type) {
      case "open":
        if (this.readyState !== CONNECTING) {
          return;
        }
        this.readyState = OPEN;
        this.protocol = json.protocol ?? "";
        this.fire("onopen", new NetEvent("open"));
        return;
      case "message":
        if (this.readyState !== OPEN) {
          return;
        }
        this.fire("onmessage", new MessageEvent("message", { data: json.binary ? reply.bytes : json.data }));
        return;
      case "error": {
        const event = new NetEvent("error");
        (event as any).message = json.message;
        this.fire("onerror", event);
        return;
      }
    }
  }

  private closed(detail: { code: number; reason: string; wasClean: boolean }): void {
    this.readyState = CLOSED;
    this.fire("onclose", new CloseEvent("close", detail));
  }

  /** Calls the event's `on…` property, then the listeners. */
  private fire(handler: "onopen" | "onmessage" | "onerror" | "onclose", event: NetEvent): void {
    try {
      (this[handler] as ((event: any) => void) | null)?.call(this, event);
    } catch (error) {
      (globalThis as any).console?.error("uncaught in WebSocket handler:", error);
    }
    this.dispatchEvent(event);
  }
}

/** The address as a `ws:` or `wss:` URL; `http:` and `https:` are accepted and mapped. */
function socketUrl(input: string | { href: string }): string {
  const text = String(input).trim().replace(/#.*$/, "");
  const match = /^([a-zA-Z][a-zA-Z0-9+.-]*):\/\//.exec(text);
  const scheme = match?.[1].toLowerCase();
  if (scheme === "ws" || scheme === "wss") {
    return text;
  }
  if (scheme === "http" || scheme === "https") {
    return `ws${text.slice(4)}`;
  }
  throw new DOMException(`The URL '${text}' is invalid: it must use the ws or wss scheme.`, "SyntaxError");
}
