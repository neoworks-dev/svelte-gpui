// Calling the app's own Rust code. `invoke` runs a command the app's crate registered
// (host/src/io/commands.rs) and `listen` hears the events that code pushes. Both ride the
// same `host.io` channel as `fetch` and `files`.

import { request, stream, toBuffer } from "./channel";

declare const console: { error(...args: unknown[]): void };

/**
 * The commands an app's Rust side offers, for type checking. Declare yours by merging
 * into this interface; `invoke` then checks names, arguments and results:
 *
 * ```ts
 * declare module "@neoworks/svelte-gpui" {
 *   interface CommandMap {
 *     greet: { args: { name: string }; result: string };
 *   }
 * }
 * ```
 */
export interface CommandMap {}

/** The events an app's Rust side emits, for type checking: name to payload type. Merge into it like `CommandMap`. */
export interface EventMap {}

/** What a command answered with: its JSON, and the bytes it sent along, empty if none. */
export interface InvokeResult<T = unknown> {
  value: T;
  bytes: Uint8Array;
}

/** Bytes as `invoke` takes them. */
export type InvokeBytes = ArrayBuffer | ArrayBufferView;

/**
 * Calls a Rust command and resolves with what it returned. The arguments are sent as
 * JSON, and `bytes` goes along as binary. A command that fails rejects with an `Error`
 * whose `message` is the command's, and whose `code` is set if the command gave one.
 * Calling a name the app did not register rejects with the code `UnknownCommand`. The
 * command runs on a thread, so the window never waits for it.
 *
 * Bytes a command sends back are dropped here; `invokeBytes` returns them.
 */
export function invoke<K extends keyof CommandMap>(
  name: K,
  args: CommandMap[K] extends { args: infer A } ? A : never,
  bytes?: InvokeBytes,
): Promise<CommandMap[K] extends { result: infer R } ? R : never>;
export function invoke<T = unknown>(name: string, args?: unknown, bytes?: InvokeBytes): Promise<T>;
export async function invoke(name: string, args?: unknown, bytes?: InvokeBytes): Promise<unknown> {
  return (await call(name, args, bytes)).json;
}

/** Like `invoke`, and also returns the bytes the command sent back. */
export async function invokeBytes<T = unknown>(
  name: string,
  args?: unknown,
  bytes?: InvokeBytes,
): Promise<InvokeResult<T>> {
  const reply = await call(name, args, bytes);
  return { value: reply.json as T, bytes: new Uint8Array(reply.bytes ?? new ArrayBuffer(0)) };
}

function call(name: string, args: unknown, bytes: InvokeBytes | undefined) {
  return request("invoke", { name, args: args ?? null }, bytes === undefined ? undefined : toBuffer(bytes));
}

type Callback = (payload: any, bytes?: ArrayBuffer) => void;

/** The host's stream for one event name, and the callbacks that share it. */
interface Subscription {
  stream: number;
  callbacks: Set<{ callback: Callback }>;
}

const subscriptions = new Map<string, Subscription>();

/**
 * Calls `callback` with each event Rust emits under `name`, and returns a function that
 * stops listening. `bytes` is set when the event carried binary data. A name needs no
 * registration: nothing arrives until Rust emits it. All the callbacks for one name share a
 * single stream from the host, and a name nobody listens to costs nothing.
 */
export function listen<K extends keyof EventMap>(name: K, callback: (payload: EventMap[K], bytes?: ArrayBuffer) => void): () => void;
export function listen<T = unknown>(name: string, callback: (payload: T, bytes?: ArrayBuffer) => void): () => void;
export function listen(name: string, callback: Callback): () => void {
  const entry = { callback };
  let subscription = subscriptions.get(name);
  if (subscription === undefined) {
    subscription = open(name);
    subscriptions.set(name, subscription);
  }
  subscription.callbacks.add(entry);
  const joined = subscription;
  return () => {
    if (!joined.callbacks.delete(entry) || joined.callbacks.size > 0) {
      return;
    }
    if (subscriptions.get(name) === joined) {
      subscriptions.delete(name);
    }
    // The stream closes when the host has taken the listener away.
    request("events.unlisten", { stream: joined.stream }).catch(() => {});
  };
}

function open(name: string): Subscription {
  const callbacks = new Set<{ callback: Callback }>();
  const handle = stream(
    "events.listen",
    { name },
    {
      event: (reply) => {
        // A callback may unlisten itself, or throw; neither stops the others.
        for (const { callback } of [...callbacks]) {
          try {
            callback(reply.json, reply.bytes);
          } catch (error) {
            console.error(`a listener for "${name}" threw: ${error instanceof Error ? error.message : error}`);
          }
        }
      },
      close: () => {},
      reject: (error) => console.error(`cannot listen for "${name}": ${error.message}`),
    },
  );
  return { stream: handle, callbacks };
}
