import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { decodeReply, encodeCall } from "../../runtime/web/wire";

// The headless host (`svelte-gpui --web DIR`): what it serves, who may talk to it, and that a call
// from the page reaches the same Rust the window's calls do. It needs the built host, which
// scripts/host.sh makes; without one these tests are skipped.

const binary = [process.env.SVELTE_GPUI_HOST, new URL("../../target/release/svelte-gpui", import.meta.url).pathname].find(
  (path): path is string => path !== undefined && existsSync(path),
);
const hosted = binary === undefined ? describe.skip : describe;

let directory: string;
let child: ReturnType<typeof Bun.spawn>;
let origin: string;
let token: string;
let port: number;

/** A call as the page sends it, by the page's own code. */
function call(op: string, id: number, args: unknown = {}, bytes?: Uint8Array): ArrayBuffer {
  return encodeCall(op, id, JSON.stringify(args), bytes === undefined ? undefined : (bytes.slice().buffer as ArrayBuffer));
}

/** What the server writes back for a request written as it is given, without a client tidying it. */
async function raw(request: string): Promise<string> {
  let text = "";
  await new Promise<void>((resolve) => {
    Bun.connect({
      hostname: "127.0.0.1",
      port,
      socket: {
        open: (socket) => void socket.write(request),
        data: (_socket, data) => void (text += data.toString()),
        close: () => resolve(),
        error: () => resolve(),
      },
    }).catch(() => resolve());
  });
  return text;
}

/** What the host answered, as the page reads it. */
function answer(data: ArrayBuffer): { header: { call: number; settle: string; json: any }; bytes: Uint8Array } {
  const reply = decodeReply(data);
  return { header: reply, bytes: new Uint8Array(reply.bytes ?? new ArrayBuffer(0)) };
}

/** Opens the page's WebSocket, and says how it ended up: `"open"`, or `"refused"` with the HTTP status when known. */
function connect(options: { token?: string; origin?: string | null } = {}) {
  const given = options.token ?? token;
  const headers: Record<string, string> = {};
  if (options.origin !== null) {
    headers.Origin = options.origin ?? origin;
  }
  const socket = new WebSocket(`ws://127.0.0.1:${port}/io?token=${given}`, { headers } as never);
  socket.binaryType = "arraybuffer";
  const opened = new Promise<"open" | "refused">((resolve) => {
    socket.addEventListener("open", () => resolve("open"));
    socket.addEventListener("error", () => resolve("refused"));
    socket.addEventListener("close", () => resolve("refused"));
  });
  const replies: ArrayBuffer[] = [];
  const waiting: ((data: ArrayBuffer) => void)[] = [];
  socket.addEventListener("message", (event) => {
    const next = waiting.shift();
    if (next) {
      next(event.data as ArrayBuffer);
    } else {
      replies.push(event.data as ArrayBuffer);
    }
  });
  const next = (): Promise<ArrayBuffer> =>
    new Promise((resolve) => {
      const ready = replies.shift();
      if (ready) {
        resolve(ready);
      } else {
        waiting.push(resolve);
      }
    });
  return { socket, opened, next };
}

beforeAll(async () => {
  if (binary === undefined) {
    return;
  }
  directory = mkdtempSync(join(tmpdir(), "svelte-gpui-serve-"));
  mkdirSync(join(directory, "app"));
  writeFileSync(join(directory, "app", "index.html"), "<p>hello</p>");
  writeFileSync(join(directory, "app", "app.js"), "console.log(1)");
  writeFileSync(join(directory, "secret.txt"), "not served");
  child = Bun.spawn([binary, "--web", join(directory, "app")], { stdout: "pipe", stderr: "pipe" });
  const reader = child.stdout.getReader();
  let said = "";
  while (!/#token=[0-9a-f]+/.test(said)) {
    const { value, done } = await reader.read();
    if (done) {
      throw new Error(`the host said nothing: ${said}`);
    }
    said += new TextDecoder().decode(value);
  }
  const found = /http:\/\/127\.0\.0\.1:(\d+)\/#token=([0-9a-f]+)/.exec(said)!;
  port = Number(found[1]);
  token = found[2];
  origin = `http://127.0.0.1:${port}`;
});

afterAll(() => {
  child?.kill();
  if (directory) {
    rmSync(directory, { recursive: true, force: true });
  }
});

hosted("the headless host", () => {
  test("serves the built app, with its types, and the page at /", async () => {
    const page = await fetch(`${origin}/`);
    expect(page.status).toBe(200);
    expect(page.headers.get("content-type")).toContain("text/html");
    expect(await page.text()).toBe("<p>hello</p>");
    const script = await fetch(`${origin}/app.js`);
    expect(script.headers.get("content-type")).toContain("javascript");
    expect(script.headers.get("cache-control")).toBe("no-store");
  });

  test("serves nothing outside the directory", async () => {
    // A client that normalises `..` away would not test anything, so the request is written by hand.
    for (const path of ["/../secret.txt", "/%2e%2e/secret.txt", "/sub/../../secret.txt", "/..%2fsecret.txt"]) {
      const reply = await raw(`GET ${path} HTTP/1.1\r\nHost: 127.0.0.1:${port}\r\nConnection: close\r\n\r\n`);
      expect(reply, path).toStartWith("HTTP/1.1 404");
      expect(reply, path).not.toContain("not served");
    }
  });

  test("refuses a request for any other host, which is what a rebinding page sends", async () => {
    const response = await fetch(`${origin}/`, { headers: { Host: "evil.example" } });
    expect(response.status).toBe(403);
  });

  test("only reads", async () => {
    expect((await fetch(`${origin}/`, { method: "POST", body: "x" })).status).toBe(405);
    expect((await fetch(`${origin}/missing`)).status).toBe(404);
  });

  test("a WebSocket needs the token", async () => {
    expect(await connect({ token: "0".repeat(64) }).opened).toBe("refused");
    expect(await connect({ token: "" }).opened).toBe("refused");
  });

  test("a WebSocket needs the server's own origin", async () => {
    expect(await connect({ origin: "http://evil.example" }).opened).toBe("refused");
    expect(await connect({ origin: `http://127.0.0.1:${port + 1}` }).opened).toBe("refused");
    expect(await connect({ origin: null }).opened).toBe("refused");
  });

  test("an app's calls reach the same files the window's do", async () => {
    const page = connect();
    expect(await page.opened).toBe("open");

    page.socket.send(call("fs.paths", 1));
    const paths = answer(await page.next());
    expect(paths.header).toMatchObject({ call: 1, settle: "resolve" });
    expect(typeof paths.header.json.home).toBe("string");

    const file = join(directory, "written.txt");
    page.socket.send(call("fs.writeBytes", 2, { path: file }, new TextEncoder().encode("from the page")));
    expect(answer(await page.next()).header).toMatchObject({ call: 2, settle: "resolve" });
    page.socket.send(call("fs.readBytes", 3, { path: file }));
    const read = answer(await page.next());
    expect(read.header.call).toBe(3);
    expect(new TextDecoder().decode(read.bytes)).toBe("from the page");

    page.socket.send(call("fs.readText", 4, { path: join(directory, "missing.txt") }));
    const missing = answer(await page.next());
    expect(missing.header).toMatchObject({ call: 4, settle: "reject" });
    expect(missing.header.json.code).toBe("NotFound");
    page.socket.close();
  });

  test("a call the host does not know is refused, as in the window", async () => {
    const page = connect();
    await page.opened;
    page.socket.send(call("nothing.here", 9));
    expect(answer(await page.next()).header).toMatchObject({ call: 9, settle: "reject" });
    page.socket.send(call("invoke", 10, { name: "greet", args: {} }));
    const unknown = answer(await page.next());
    expect(unknown.header.json.code).toBe("UnknownCommand");
    page.socket.close();
  });

  test("a dialog, which needs a window, says so instead of hanging", async () => {
    const page = connect();
    await page.opened;
    page.socket.send(call("fs.pickFiles", 11, {}));
    const refused = answer(await page.next());
    expect(refused.header).toMatchObject({ call: 11, settle: "reject" });
    expect(refused.header.json.message).toContain("needs a window");
    page.socket.close();
  });

  test("a message that is not a call is ignored, and the connection goes on", async () => {
    const page = connect();
    await page.opened;
    page.socket.send(new Uint8Array([1, 2, 3]));
    page.socket.send(call("fs.paths", 12));
    expect(answer(await page.next()).header.call).toBe(12);
    page.socket.close();
  });

  test("a second page takes the session, and the first is closed", async () => {
    const first = connect();
    await first.opened;
    const closed = new Promise<void>((resolve) => first.socket.addEventListener("close", () => resolve()));
    const second = connect();
    await second.opened;
    await closed;
    second.socket.send(call("fs.paths", 13));
    expect(answer(await second.next()).header.call).toBe(13);
    second.socket.close();
  });
});
