import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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

interface Running {
  child: ReturnType<typeof Bun.spawn>;
  port: number;
  token: string;
}

/** Starts the host serving `dir`, and reads the address it prints. */
async function startHost(dir: string, args: string[] = [], env: Record<string, string> = {}): Promise<Running> {
  const child = Bun.spawn([binary!, "--web", dir, ...args], { stdout: "pipe", stderr: "pipe", env: { ...process.env, ...env } });
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
  return { child, port: Number(found[1]), token: found[2] };
}

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
function connect(options: { token?: string; origin?: string | null; host?: Running } = {}) {
  const where = options.host ?? { port, token };
  const given = options.token ?? where.token;
  const headers: Record<string, string> = {};
  if (options.origin !== null) {
    headers.Origin = options.origin ?? `http://127.0.0.1:${where.port}`;
  }
  const socket = new WebSocket(`ws://127.0.0.1:${where.port}/io?token=${given}`, { headers } as never);
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
  // No session bus to reach, so that a dialog this host is asked for can show nothing on anybody's desktop.
  const running = await startHost(join(directory, "app"), [], { DBUS_SESSION_BUS_ADDRESS: "unix:path=/nonexistent" });
  child = running.child;
  port = running.port;
  token = running.token;
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

  test("a dialog the desktop cannot show says so instead of hanging", async () => {
    const page = connect();
    await page.opened;
    page.socket.send(call("fs.pickFiles", 11, {}));
    const refused = answer(await page.next());
    expect(refused.header).toMatchObject({ call: 11, settle: "reject" });
    expect(refused.header.json.message).toContain("file chooser");
    page.socket.send(call("fs.pickSavePath", 12, { name: "a.txt" }));
    expect(answer(await page.next()).header).toMatchObject({ call: 12, settle: "reject" });
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

// The dialogs, against a stand-in for the desktop's file chooser on a private session bus, which needs
// dbus-daemon and python with dbus and gi; without them these are skipped. The portal itself is the desktop's.
const canMockPortal =
  binary !== undefined &&
  Bun.which("dbus-daemon") !== null &&
  Bun.spawnSync(["python3", "-c", "import dbus, gi"], { stdout: "ignore", stderr: "ignore" }).exitCode === 0;
const withPortal = canMockPortal ? describe : describe.skip;

withPortal("the dialogs through the desktop's file chooser", () => {
  let scratch: string;
  let bus: ReturnType<typeof Bun.spawn>;
  let portal: ReturnType<typeof Bun.spawn>;
  let host: Running;
  let requests: string;

  /** What the stand-in portal was asked, one request per line. */
  const asked = (): { kind: string; title: string; options: any }[] =>
    existsSync(requests)
      ? readFileSync(requests, "utf8").trim().split("\n").filter(Boolean).map((line) => JSON.parse(line))
      : [];

  beforeAll(async () => {
    scratch = mkdtempSync(join(tmpdir(), "svelte-gpui-portal-"));
    requests = join(scratch, "requests.log");
    mkdirSync(join(scratch, "picked"));
    writeFileSync(join(scratch, "picked", "a.txt"), "chosen in a dialog");
    mkdirSync(join(scratch, "app"));
    writeFileSync(join(scratch, "app", "index.html"), "<p>x</p>");
    bus = Bun.spawn(["dbus-daemon", "--session", "--nofork", "--print-address=1"], { stdout: "pipe", stderr: "ignore" });
    const address = new TextDecoder().decode((await bus.stdout.getReader().read()).value).trim();
    const env = { DBUS_SESSION_BUS_ADDRESS: address, MOCK_PORTAL_LOG: requests, MOCK_PORTAL_ROOT: join(scratch, "picked") };
    portal = Bun.spawn(["python3", new URL("./fixtures/mock-portal.py", import.meta.url).pathname], {
      stdout: "pipe",
      stderr: "inherit",
      env: { ...process.env, ...env },
    });
    await portal.stdout.getReader().read();
    // File access is narrowed to a directory the dialogs' answers are not in, so reading them proves they were allowed.
    host = await startHost(join(scratch, "app"), ["--allow-files", join(scratch, "app")], env);
  });

  afterAll(() => {
    host?.child.kill();
    portal?.kill();
    bus?.kill();
    rmSync(scratch, { recursive: true, force: true });
  });

  test("a file dialog gives the paths the user chose, which the app may read from then on", async () => {
    const page = connect({ host });
    await page.opened;
    page.socket.send(call("fs.readText", 1, { path: join(scratch, "picked", "a.txt") }));
    expect(answer(await page.next()).header.json.code).toBe("NotAllowed");

    page.socket.send(call("fs.pickFiles", 2, { multiple: true, prompt: "Import" }));
    const chosen = answer(await page.next());
    expect(chosen.header).toMatchObject({ call: 2, settle: "resolve" });
    expect(chosen.header.json.paths).toEqual([join(scratch, "picked", "a.txt"), join(scratch, "picked", "b.txt")]);

    page.socket.send(call("fs.readText", 3, { path: join(scratch, "picked", "a.txt") }));
    expect(answer(await page.next()).header.json).toEqual({ text: "chosen in a dialog" });
    page.socket.close();
  });

  test("the options reach the chooser: files or folders, several, the button's text and the filters", async () => {
    const page = connect({ host });
    await page.opened;
    const filters = [{ name: "Images", extensions: ["png", ".jpg"] }, { extensions: ["*.gif"] }];
    page.socket.send(call("fs.pickFiles", 4, { multiple: true, prompt: "Import", filters }));
    await page.next();
    page.socket.send(call("fs.pickFiles", 5, { directories: true }));
    const folder = answer(await page.next());
    expect(folder.header.json.paths).toEqual([join(scratch, "picked", "folder")]);

    const [files, folders] = asked().slice(-2);
    expect(files).toMatchObject({ kind: "open", title: "Open File" });
    expect(files.options).toMatchObject({ multiple: true, directory: false, accept_label: "Import" });
    expect(files.options.filters).toEqual([
      ["Images", [[0, "*.png"], [0, "*.jpg"]]],
      ["*.gif", [[0, "*.gif"]]],
    ]);
    expect(folders).toMatchObject({ kind: "open", title: "Open Folder" });
    expect(folders.options).toMatchObject({ directory: true, multiple: false });
    page.socket.close();
  });

  test("a save dialog starts where the app said, suggests its name, and gives the path", async () => {
    const page = connect({ host });
    await page.opened;
    page.socket.send(call("fs.pickSavePath", 6, { directory: scratch, name: "notes.md", filters: [{ extensions: ["md"] }] }));
    const saved = answer(await page.next());
    expect(saved.header).toMatchObject({ call: 6, settle: "resolve" });
    expect(saved.header.json.path).toBe(join(scratch, "picked", "notes.md"));
    const request = asked().at(-1)!;
    expect(request).toMatchObject({ kind: "save", title: "Save File" });
    expect(request.options.current_name).toBe("notes.md");
    // The folder crosses D-Bus as a NUL-terminated byte string.
    expect(Buffer.from(request.options.current_folder.slice(0, -1)).toString()).toBe(scratch);
    page.socket.close();
  });

  test("a dialog the user cancels resolves with null, as it does in the window", async () => {
    const page = connect({ host });
    await page.opened;
    // The stand-in cancels a request whose button says so.
    page.socket.send(call("fs.pickFiles", 7, { prompt: "Cancel" }));
    expect(answer(await page.next()).header).toMatchObject({ call: 7, settle: "resolve", json: { paths: null } });
    page.socket.close();
  });

  test("several dialogs at once answer each its own call", async () => {
    const page = connect({ host });
    await page.opened;
    page.socket.send(call("fs.pickFiles", 8, { prompt: "Cancel" }));
    page.socket.send(call("fs.pickSavePath", 9, { name: "x.txt" }));
    const replies = [answer(await page.next()), answer(await page.next())].sort((a, b) => a.header.call - b.header.call);
    expect(replies[0].header).toMatchObject({ call: 8, json: { paths: null } });
    expect(replies[1].header.json.path).toBe(join(scratch, "picked", "x.txt"));
    page.socket.close();
  });
});
