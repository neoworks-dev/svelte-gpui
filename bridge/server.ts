// Grove's main process, run without Electron: it registers its handlers against
// the stand-in `electron` module, and this serves them over a unix socket. One
// JSON object per line. A request is `{id, channel, args}` and is answered with
// `{id, result}` or `{id, error}`; what the main process pushes to its window
// arrives as `{event, args}`. Bytes travel as `{"$bytes": "<base64>"}`.

import { createServer, type Socket } from "node:net";
import { rmSync } from "node:fs";
import { app, broadcast, invokeHandlers, sendListeners, subscribers } from "electron";
import { registerIpc, shutdown } from "@grove/src/main/ipc";

const socketPath = process.argv[2];
if (socketPath === undefined) {
  console.error("usage: server.js SOCKET");
  process.exit(2);
}

/** JSON with bytes made safe. */
function encode(value: unknown): string {
  return JSON.stringify(value, (_key, item) => {
    if (item instanceof Uint8Array) {
      return { $bytes: Buffer.from(item).toString("base64") };
    }
    if (item && item.type === "Buffer" && Array.isArray(item.data)) {
      return { $bytes: Buffer.from(item.data).toString("base64") };
    }
    return item;
  });
}

function decode(line: string): any {
  return JSON.parse(line, (_key, item) => {
    if (item && typeof item === "object" && typeof item.$bytes === "string") {
      return new Uint8Array(Buffer.from(item.$bytes, "base64"));
    }
    return item;
  });
}

const sender = { id: 1 };

async function handle(socket: Socket, message: any): Promise<void> {
  if (typeof message.id !== "number") {
    for (const listener of sendListeners.get(message.channel) ?? []) {
      listener({ sender }, ...(message.args ?? []));
    }
    return;
  }
  const handler = invokeHandlers.get(message.channel);
  if (handler === undefined) {
    socket.write(encode({ id: message.id, error: `no handler for ${message.channel}` }) + "\n");
    return;
  }
  try {
    const result = await handler({ sender }, ...(message.args ?? []));
    socket.write(encode({ id: message.id, result: result === undefined ? null : result }) + "\n");
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error);
    socket.write(encode({ id: message.id, error: text }) + "\n");
  }
}

async function main(): Promise<void> {
  await registerIpc();
  rmSync(socketPath, { force: true });
  const server = createServer((socket) => {
    const push = (channel: string, args: unknown[]): void => {
      socket.write(encode({ event: channel, args }) + "\n");
    };
    subscribers.add(push);
    socket.on("close", () => subscribers.delete(push));
    socket.on("error", () => subscribers.delete(push));
    let buffer = "";
    socket.on("data", (chunk) => {
      buffer += chunk.toString("utf8");
      let newline = buffer.indexOf("\n");
      while (newline !== -1) {
        const line = buffer.slice(0, newline);
        buffer = buffer.slice(newline + 1);
        if (line.trim() !== "") {
          void handle(socket, decode(line));
        }
        newline = buffer.indexOf("\n");
      }
    });
  });
  server.listen(socketPath, () => {
    console.log(`grove main process listening on ${socketPath}`);
  });
  process.on("SIGTERM", async () => {
    await shutdown().catch(() => {});
    process.exit(0);
  });
}

void main().catch((error) => {
  console.error("the main process failed to start:", error);
  process.exit(1);
});
void app;
void broadcast;
