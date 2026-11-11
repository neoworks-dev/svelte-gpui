// The page's line to the host. In a browser the same Rust host runs without a window and the page
// talks to it over a WebSocket, so every call that reaches Rust in the native window, `invoke`,
// `files`, `fetch` and `WebSocket` among them, reaches it here too. This is `host.io` for the page:
// `runtime/io/channel.ts` sends through it exactly as it does in QuickJS, and what the host answers
// goes to `receive` there. The wire is described in host/src/serve/mod.rs.
//
// The page learns the address's token from the fragment of the URL the host printed
// (`#token=…`), which a browser sends nobody, and keeps it for the tab.

import { failAll, receive } from "../io/channel";
import { fetch as hostFetch } from "../io/fetch";
import { WebSocket as HostWebSocket } from "../io/websocket";
import { decodeReply, encodeCall } from "./wire";

/** The browser's own, before the page's are replaced by the host's. */
const BrowserWebSocket = globalThis.WebSocket;
const BrowserRequest = globalThis.Request;

const TOKEN = "svelte-gpui:token";

const NO_HOST =
  "there is no svelte-gpui host behind this page. Run `svelte-gpui --web DIR` (or the app's own binary) and open " +
  "the address it prints, which carries the token the page needs to talk to it.";

interface Connection {
  socket: WebSocket;
  opened: Promise<void>;
}

let connection: Connection | null = null;

/** The token the page was opened with, kept for the tab so that a reload still has it. */
function tokenFromPage(): string | null {
  try {
    const given = /[#&]token=([0-9a-f]+)/.exec(location.hash)?.[1];
    if (given !== undefined) {
      sessionStorage.setItem(TOKEN, given);
      // The fragment need not stay in the address bar, or in a bookmark.
      history.replaceState(null, "", location.pathname + location.search);
      return given;
    }
    return sessionStorage.getItem(TOKEN);
  } catch {
    return null;
  }
}

function open(token: string): Connection {
  const scheme = location.protocol === "https:" ? "wss" : "ws";
  const socket = new BrowserWebSocket(`${scheme}://${location.host}/io?token=${token}`);
  socket.binaryType = "arraybuffer";
  const opened = new Promise<void>((resolve, reject) => {
    socket.addEventListener("open", () => resolve(), { once: true });
    socket.addEventListener(
      "error",
      () =>
        reject(
          new Error(
            "could not connect to the svelte-gpui host. It is not running, or the token this page has is not its own: " +
              "open the address the host printed, which carries the current one.",
          ),
        ),
      { once: true },
    );
  });
  socket.addEventListener("message", (event) => {
    const reply = decodeReply(event.data as ArrayBuffer);
    receive(reply.call, reply.settle, JSON.stringify(reply.json), reply.bytes);
  });
  socket.addEventListener("close", () => {
    failAll("the connection to the svelte-gpui host closed", "HostClosed");
    if (connection?.socket === socket) {
      connection = null;
    }
  });
  return { socket, opened };
}

/** Sends a call, once the connection is open. The host's answer arrives through `receive`. */
function io(op: string, call: number, argsJson: string, bytes: ArrayBuffer | undefined): void {
  if (connection === null) {
    // Answered later than it was asked, as a real answer would be.
    queueMicrotask(() => receive(call, "reject", JSON.stringify({ message: NO_HOST, code: "NoHost" }), undefined));
    return;
  }
  const { socket, opened } = connection;
  const message = encodeCall(op, call, argsJson, bytes);
  if (socket.readyState === socket.OPEN) {
    socket.send(message);
    return;
  }
  opened.then(
    () => socket.send(message),
    (error: Error) => receive(call, "reject", JSON.stringify({ message: error.message, code: "HostUnreachable" }), undefined),
  );
}

/**
 * A `fetch` that goes through the host, which has no cross-origin rules, as in the native window. It takes
 * the page's own `Request` too, which the host's does not know.
 */
async function fetchThroughHost(input: Parameters<typeof hostFetch>[0] | Request, init?: Parameters<typeof hostFetch>[1]) {
  if (BrowserRequest !== undefined && input instanceof BrowserRequest) {
    const noBody = input.method === "GET" || input.method === "HEAD";
    init = {
      method: input.method,
      headers: Object.fromEntries(input.headers.entries()),
      body: noBody ? undefined : new Uint8Array(await input.arrayBuffer()),
      signal: input.signal as never,
      ...init,
    };
    input = input.url;
  }
  return hostFetch(input as string, init);
}

/**
 * Connects to the host the page was served by, and makes `fetch` and `WebSocket` go through it, as they do in
 * the native window. A page that was not opened through the host, which has no token, keeps the browser's own
 * and fails `invoke` and `files` with an error that says so. Returns whether there is a host.
 */
export function connectToHost(): boolean {
  const token = tokenFromPage();
  const globals = globalThis as any;
  globals.host = { io };
  if (token === null) {
    return false;
  }
  connection = open(token);
  // The first failure to connect reaches the console once instead of at every call.
  connection.opened.catch((error: Error) => console.error(error.message));
  globals.fetch = fetchThroughHost;
  globals.WebSocket = HostWebSocket;
  return true;
}
