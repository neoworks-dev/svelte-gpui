# Network and files

An app reaches the outside world through three APIs:

- **`fetch`**, the web's own.
- **`WebSocket`**, the web's own.
- **`files`**, from `@neoworks/svelte-gpui`.

The host runs each request on a background thread, and the result settles the promise
or fires the event as soon as it arrives, so the window never waits on the network or
the disk.

`examples/demo` has a Network section and a Files section that use all three.

## fetch

`fetch` works as in a browser, so code and libraries written for the web work unchanged:

```ts
const response = await fetch("https://api.github.com/repos/zed-industries/zed", {
  headers: { accept: "application/vnd.github+json" },
});
if (!response.ok) {
  throw new Error(`GitHub said ${response.status}`);
}
const repository = await response.json();
```

| Part | Supported |
|---|---|
| Input | A URL string, a `URL`, or a `Request`. |
| `method`, `headers` | Any. Headers can be an object, a list of pairs, or a `Headers`. |
| `body` | A string, `ArrayBuffer`, typed array, `DataView` or `URLSearchParams`. Strings get `text/plain;charset=UTF-8` and `URLSearchParams` gets `application/x-www-form-urlencoded`, unless you set a content type. |
| `redirect` | `"follow"` (the default, up to 20 hops), `"manual"` or `"error"`. |
| `signal` | An `AbortSignal`, including `AbortSignal.timeout(ms)` and `AbortSignal.any([...])`. Aborting rejects with an `AbortError`. |
| `Response` | `ok`, `status`, `statusText`, `headers`, `url`, `redirected`, `text()`, `json()`, `arrayBuffer()`, `bytes()`, `clone()`, `bodyUsed`. |

As in browsers, a 404 or 500 **resolves** with that status. Only a failure to reach
the server (DNS, a refused connection, TLS) rejects, with a `TypeError`.

`Headers`, `Request`, `Response`, `AbortController`, `AbortSignal`, `TextEncoder`,
`TextDecoder` (UTF-8) and `DOMException` are globals too.

Not supported:
- Bodies are read whole before the promise resolves, so there is no `response.body`
  stream and no upload progress.
- There is no `Blob` or `FormData`.
- Cookies are not kept between requests.
- An aborted request still finishes on its thread; the result is dropped.

## WebSocket

```ts
const socket = new WebSocket("wss://example.com/live");
socket.binaryType = "arraybuffer";
socket.onopen = () => socket.send(JSON.stringify({ subscribe: "prices" }));
socket.onmessage = (event) => update(JSON.parse(event.data));
socket.onclose = (event) => console.log("closed", event.code, event.reason);
```

Each socket has its own thread, which sleeps until a message arrives or the app sends
one. An idle socket costs nothing, and a message goes out at once.

| Part | Supported |
|---|---|
| `new WebSocket(url, protocols?)` | `ws://`, `wss://`, and `http(s)://`, which mean the same. |
| Events | `onopen`, `onmessage`, `onerror` and `onclose`, plus `addEventListener` and `removeEventListener`. |
| `send(data)` | A string, `ArrayBuffer`, typed array or `DataView`. It throws `InvalidStateError` before `open`. |
| `close(code?, reason?)` | Waits up to 5 seconds for the server's reply. |
| `MessageEvent.data` | A string, or an `ArrayBuffer` for binary messages. |
| `CloseEvent` | `code`, `reason`, `wasClean`. |
| Properties | `readyState` and its constants, `protocol`, `url`, `binaryType`. |

`binaryType` can only be `"arraybuffer"`, because there is no `Blob`. A connection that
fails fires `error` and then `close` with code 1006, as in browsers. `bufferedAmount`
is always 0.

## Files

```ts
import { files, FileError } from "@neoworks/svelte-gpui";

const { data } = await files.paths();
const settingsPath = `${data}/my-app/settings.json`;

await files.mkdir(`${data}/my-app`, { recursive: true });
await files.writeText(settingsPath, JSON.stringify(settings));

try {
  settings = JSON.parse(await files.readText(settingsPath));
} catch (error) {
  if (!(error instanceof FileError && error.code === "NotFound")) throw error;
}
```

Every call returns a promise and runs on a thread.

| Function | Result |
|---|---|
| `readText(path)` | `string`. Rejects with `InvalidData` if the file is not UTF-8. |
| `readBytes(path)` | `Uint8Array` |
| `writeText(path, text)`, `writeBytes(path, data)` | Creates or replaces the file. |
| `appendText(path, text)` | Creates the file if needed. |
| `exists(path)` | `boolean` |
| `stat(path)` | `{ kind, size, modified, created? }`. `kind` is `"file"`, `"directory"` or `"symlink"`, and times are ms since the epoch. A final symlink is not followed. |
| `list(directory)` | Entries sorted by name, each `{ name, path, kind, size, modified, created? }`. |
| `mkdir(path, { recursive? })` | |
| `remove(path, { recursive? })` | A directory needs `recursive` unless it is empty. A symlink removes the link, not its target. |
| `rename(from, to)`, `copy(from, to)` | `copy` copies directories recursively. `rename` does not work across file systems. |
| `watch(path, callback, { recursive?, onerror? })` | Calls `callback({ kind, path })` with `kind` `"create"`, `"modify"` or `"remove"`, as changes happen. Returns a function that stops the watch. |
| `paths()` | `{ home, config, data, cache, temp, bundle, cwd }`: the system's directories (XDG on Linux), the bundle's directory, and the working directory. |
| `pickFiles({ multiple?, directories?, prompt? })` | The system's open dialog. Resolves with `string[]`, or `null` if cancelled. |
| `pickSavePath({ directory?, name? })` | The system's save dialog. Resolves with a `string`, or `null`. |

Paths:
- `~` at the start of a path means the home directory.
- A relative path resolves against the directory the process started in. An app cannot
  rely on what that is, so build absolute paths from `paths()`.

Failures reject with a `FileError`:
- `message` names the operation, the path and the system's reason.
- `code` is one of `NotFound`, `PermissionDenied`, `NotAllowed`, `Exists`, `NotEmpty`,
  `NotADirectory`, `IsADirectory`, `InvalidData`, `InvalidArgument` or `Other`.

Watches use the system's change notifications (inotify on Linux), so they report at
once and cost nothing while idle. A rename shows as a `remove` and a `create`. A watch
that cannot start (for instance on a missing path) calls `onerror`, or logs if there is
none.

### Limiting file access

By default an app may read and write anything the user can, like any native program.
An app that only needs some places can say so when it starts, and the host then
refuses everything else:

```ts
start(App, {}, {
  files: { allow: ["$data/my-app", "$config/my-app", "~/Documents/My App"] },
});
```

- **What an entry grants:** each entry allows itself and everything below it.
- **Placeholders:** `$home`, `$config`, `$data`, `$cache`, `$temp`, `$bundle` and `$cwd`
  at the start of an entry stand for the directories from `files.paths()`, and `~` for
  home.
- **Nothing:** `allow: []` allows no files at all.
- **Dialogs:** whatever the user picks in `pickFiles` or `pickSavePath` is allowed from
  then on, even outside the limit. A picked directory allows everything below it.
- **Set once:** the limit is set before the app mounts and cannot be changed or widened
  later, so code that runs afterwards (including a dependency) cannot lift it.
- **How it is checked:** on the real path, after following symlinks and resolving `..`,
  so neither can escape. Both sides of `rename` and `copy` are checked.
- **What a refusal looks like:** a refused call rejects with code `NotAllowed`. A refused
  watch calls `onerror`.

Whoever runs the app can narrow it further with the host flag `--allow-files PATH`,
which can be repeated. When both the app and the flag set limits, a path must be inside
both, so the flag can only take access away. `paths()` and the dialogs always work.

## Under the hood

The three APIs share one path:

1. The app calls `host.io(op, call, args, bytes)`. `runtime/io/channel.ts` turns this
   into a promise, or into a stream of events for sockets and watches.
2. The host's `Io` (`host/src/io/`) starts the work on a thread: `fetch.rs`,
   `websocket.rs` or `files.rs`. Dialogs go to the window instead, because they need
   GPUI.
3. The thread sends its result down a channel. The window wakes as soon as a result
   arrives, hands it to the app (`__io`), and draws what changed.

Bytes travel as `ArrayBuffer`s, and everything else as JSON.
