// The app's side of `host.io`: each call gets a number, the host runs it off the main
// thread, and what comes back through `__io` settles the promise or reaches the
// stream's handlers. See host/src/io/mod.rs for the other side.

/** What the host sent back: JSON, and bytes for binary payloads. */
export interface Reply {
  json: any;
  bytes?: ArrayBuffer;
}

interface Handlers {
  resolve?(reply: Reply): void;
  reject?(error: Error): void;
  event?(reply: Reply): void;
  close?(reply: Reply): void;
}

const calls = new Map<number, Handlers>();
let nextCall = 1;

function start(op: string, args: unknown, bytes: ArrayBuffer | undefined, handlers: Handlers): number {
  const call = nextCall++;
  calls.set(call, handlers);
  (globalThis as any).host.io(op, call, JSON.stringify(args ?? {}), bytes);
  return call;
}

/** Starts an operation and waits for its one answer. A failure rejects with an `Error`. */
export function request(op: string, args?: unknown, bytes?: ArrayBuffer): Promise<Reply> {
  return new Promise((resolve, reject) => {
    start(op, args, bytes, { resolve, reject });
  });
}

/**
 * Starts a stream, such as a socket, and returns its call number, which later requests
 * name to act on it. `event` hears what it produces until `close`; `reject` hears a
 * failure to start, after which nothing else comes.
 */
export function stream(
  op: string,
  args: unknown,
  handlers: { event(reply: Reply): void; close(reply: Reply): void; reject(error: Error): void },
  bytes?: ArrayBuffer,
): number {
  return start(op, args, bytes, handlers);
}

/** Bytes as the host takes them: an `ArrayBuffer` of exactly the viewed range. */
export function toBuffer(data: ArrayBuffer | ArrayBufferView): ArrayBuffer {
  if (data instanceof ArrayBuffer) {
    return data;
  }
  return data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer;
}

/** An `Error` for a failed call, with the host's `code` when it gave one. */
function failure(json: any): Error {
  const error = new Error(String(json?.message ?? "failed")) as Error & { code?: string };
  if (typeof json?.code === "string") {
    error.code = json.code;
  }
  return error;
}

/** Called by the host with what a call produced. */
export function receive(call: number, settle: string, json: string, bytes: ArrayBuffer | undefined): void {
  const handlers = calls.get(call);
  if (handlers === undefined) {
    return;
  }
  const reply: Reply = { json: json === "" ? null : JSON.parse(json), bytes };
  switch (settle) {
    case "resolve":
      calls.delete(call);
      handlers.resolve?.(reply);
      return;
    case "reject":
      calls.delete(call);
      handlers.reject?.(failure(reply.json));
      return;
    case "event":
      handlers.event?.(reply);
      return;
    case "close":
      calls.delete(call);
      handlers.close?.(reply);
      return;
  }
}
