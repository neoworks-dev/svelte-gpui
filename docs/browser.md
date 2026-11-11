# Targets: the native window and the browser

One app builds for two targets from the same source: the native window the host opens with
GPUI, and a browser tab. The app is written exactly as it is for the window, with the
same primitives, props, style objects and events, and the same `@neoworks/svelte-gpui`
imports. Only drawing differs. In a browser the primitives are DOM elements, and the
host, which keeps doing everything that reaches Rust, runs without a window.

```
bun build.mjs --web             # the app, bundled for the browser
svelte-gpui --web dist/web      # the host, serving it and answering its calls
```

The host prints an address to open, such as `http://127.0.0.1:41937/#token=…`.

## What is the same

- Every component of `@neoworks/svelte-gpui/ui`, with the same props and events: the types are
  shared (`runtime/ui/props.ts`), so they cannot drift.
- Every style property in [Styles](styles.md), turned into CSS by one function, `toCss`.
  Hover, active and focus styles, transitions, `zoom`, `truncate`, `maxLines` and the rest work.
- `invoke`, `listen`, `files`, `fetch` and `WebSocket` reach the same Rust the native
  window's calls do. A command your crate registers answers a page as it answers the window, and
  `fetch` has no cross-origin rules, because the host makes the request.
- `clipboard`, `window`, `frameStats()` and `element()` are exported with the same names and types.
  What each does in a page is under [What differs](#what-differs).

## Building

`buildWeb` is exported by `build.mjs`, next to `build`:

```js
import { buildWeb } from "@neoworks/svelte-gpui/build";

await buildWeb({
  entry: "/abs/path/src/main.ts",
  outdir: "/abs/path/dist/web",
  title: "My app",
});
```

```ts
buildWeb({
  entry: string,      // absolute path of the entry, the one `build` takes
  outdir: string,     // where index.html and the rest are written
  alias?: Record<string, string>,   // esbuild's alias, to name packages the app imports
  watch?: boolean,    // rebuild whenever a source file changes
  title?: string,     // the page's title until the app sets one
  // and esbuild's define, loader, plugins, logLevel, sourcemap and minify
}): Promise<{ outdir, html, js, css, stop }>
```

It writes:

| File | What |
|---|---|
| `index.html` | A page that loads the script as a module, and the stylesheet when there is one. |
| `app.js` | The app and the DOM primitives, minified unless `watch` is on. |
| `app.css` | The `<style>` blocks of your components, when there are any. |
| `fonts/` | Geist and JetBrains Mono, the host's own faces, which `"sans"` and `"mono"` name. |

`css` in the result is `null` without a stylesheet, and `stop` ends the watching.

An app outside this repository builds with it too: it needs no copy of the package in its
`node_modules`, and no `svelte` of its own if it has none. `alias` names what it imports that
esbuild cannot find, such as another workspace package.

Images and other files the app loads by a relative path must sit next to `index.html`, as
they sit next to the bundle natively. The demo's build copies its `assets/` there.

A bundler of your own gets the same through package.json: under the `browser` condition,
`@neoworks/svelte-gpui` and `@neoworks/svelte-gpui/ui` resolve to the DOM versions
(`runtime/web/`), and without it to the native ones. The composed components (`List`, `Popover`,
`Menu`, …) are one implementation for both, built on `#primitives`, which package.json also
points at the right primitives.

Type checking: `tsconfig.web.json` here checks the same source against the DOM versions with
`customConditions: ["browser"]`. Do the same in your own config to check both targets.

## Running

```
svelte-gpui --web DIR [--port N] [--allow-files PATH]…
```

serves `DIR` and the page's calls on `127.0.0.1`, on a free port unless `--port` is given, and
prints the address. `--allow-files` narrows file access, as for the window. The host's `--bundle`
flags do not apply: there is no window.

An app that has its own Rust crate serves the same way, with the commands it registers for the window:

```rust
let commands = commands::all();
let server = svelte_gpui::serve(svelte_gpui::ServeOptions::new("dist/web").commands(commands))?;
println!("open {}", server.url());
server.wait();
```

`serve` returns a `Server` and runs on threads of its own, which stop when it is dropped. `run_web`
is `serve` followed by `wait`, as the command line does. `Server::url()` is the address to open.

## How it works

The page connects to the host with a WebSocket at `/io`, and sends it the calls it would make to
`host.io` in the window: `invoke`, `fs.*`, `fetch`, `ws.*`. The host runs them with the same
code, on threads, and answers over the same socket, where `runtime/io/channel.ts` settles the
promises exactly as it does in QuickJS. `fetch` and `WebSocket` in the page are replaced with the
host's, which run through the same connection. A page opened without the host's address, from a
static server or a file, keeps the browser's own, and `invoke` and `files` fail with the code
`NoHost` and a message that says how to open it.

Every message is binary: four bytes with the length of a JSON header, the header, and the call's bytes.
The page sends `{ op, call, args }`. The host sends `{ call, settle, json }`, where `settle` is
`resolve`, `reject`, `event` or `close`. `host/src/serve/` has the host's side.

A server has one session at a time. A reload, or a second tab, takes it over, and the first is
closed: a command's events go to one page, not split between two.

## Who may talk to the host

The host can read and write your files and run your Rust, so a page must not be able to talk to
it unless you opened it.

- It listens on `127.0.0.1` only.
- It makes a random 256-bit token each time it starts and gives it to the page in the fragment of
  the address it prints. A browser never sends a fragment to anybody, so the token reaches only the
  page you opened. The page keeps it for the tab, takes it out of the address bar, and sends it back
  in the WebSocket's address.
- The WebSocket is refused without the token, compared in constant time, and without the server's
  own origin in `Origin`. A page on another site cannot set that header, and cannot learn the token.
- Every request, for a file or the WebSocket, must name the server's own address in `Host`, which
  stops a page on another site whose name points at this machine from reading it.
- The server only reads files from `DIR`, never outside it, links included, and only answers
  `GET` and `HEAD`.

## What differs

An app that runs on both needs to know these.

| | Native window | Browser |
|---|---|---|
| Which target | `target === "native"` | `target === "browser"`, both from `@neoworks/svelte-gpui` |
| Files dropped on a `View` | `event.paths`, absolute paths the host reads | `event.paths` is empty. `event.files` holds the `File`s on `ondrop`, to read with `text()` and `arrayBuffer()`. A browser tells the page no path. On `ondragenter` and `ondragover` only `count` is known. |
| `files.pickFiles()`, `pickSavePath()` | The system's dialogs, shown by the window | The desktop's own file chooser, shown by the host without a window, with real paths as natively. `filters` work here only. See [File dialogs](#file-dialogs). |
| `files.paths()`, `files.*` on paths | The machine the host runs on | The same, since it is the same host. |
| `clipboard` | The system's, through the host | The browser's: it needs the tab focused and the user's permission. `readImage()` hands the image to the host to write into the temporary directory, and answers with that path, or `null` if the app's file limit does not allow that directory. |
| `window.setTitle`, `options.window.title` | The title bar | `document.title` |
| `options.window` size | Opens the window at that size | Ignored. `minWidth` and `minHeight` are the least size the page lays out at, scrolling below that. |
| `window.minimize()`, `maximize()`, … | Ask the window manager | Do nothing. `window.maximized` is always `false`. Fullscreen works. |
| `window.close()` | Closes the window | Closes the tab, if the browser lets a page. `onCloseRequested` hooks the browser's own leave prompt, which an answer of `false` triggers. It can only ask at once, not wait for a promise. |
| `window.scale`, `size`, `focused` | Reactive | Reactive, from the tab |
| `element(name)` | Draws a GPUI view your crate registered | A box with a notice, and its methods reject. GPUI views are native only. |
| `frameStats()` | Frames drawn, idle windows draw few | Frames the browser drew, near the display's rate, and `buildMs: 0` |
| `Image` `src` | A file next to the bundle, or a URL | A URL, resolved against the page, so files sit next to `index.html` |
| `Icon` | Tinted vector | The drawing as a mask over the colour, so every SVG is tinted whatever it names |
| `<style>` blocks, `class` | Ignored | Work, since there is a DOM, but an app that wants to run on both should not use them |
| Text | Cannot be selected | Can be, and copied |
| Tab key | No navigation | The browser's focus order: buttons, fields and `focusable` views |
| Scrollbars | None | Thin, dark ones, on a `Scroll` |
| `Presence` | `fade`, `fly`, vertical `slide` | Every Svelte transition |
| Hover | The mouse | The mouse and pen, not touch |
| Several pages | One window | One session: a second page takes over |

A few places differ in how they are drawn, and the apps notice them only at the edges:

- An element with no pointer handler lets the pointer through to what is behind it, as in the window.
  `Text` is the exception, and takes it so that it can be selected.
- An `Overlay`'s element lives in a layer above the app, with a hidden marker where you wrote it. Keys
  and presses still reach the elements around the marker, as they do in the window. What it does not
  do is inherit text colour and font from its parent: give the overlay a `style` that sets them, as the
  default panel does.
- Of two overlays, the one that appeared later is on top, and a higher `layer` is above a lower one.
- A child with `flex` or `grow` may be smaller than its content, as in the window; any other keeps
  CSS's rule that it is not, unless it sets `minWidth: 0` or `minHeight: 0`.
- `Scroll`'s `anchor` is done by the library rather than by the browser's `overflow-anchor`, which
  not every browser has.

## Writing one app for both

Dropped files are the one thing to handle both ways. Anything else that only one target can do goes behind
`target`, which is `"native"` or `"browser"`:

```svelte
<script lang="ts">
  import { files } from "@neoworks/svelte-gpui";
  import { View, Text, type FileDragEvent } from "@neoworks/svelte-gpui/ui";

  async function drop(event: FileDragEvent): Promise<void> {
    // Native: paths the host reads. Browser: the files themselves.
    const texts = event.files
      ? await Promise.all(event.files.map((file) => file.text()))
      : await Promise.all(event.paths.map((path) => files.readText(path)));
    open(texts);
  }
</script>

<View ondrop={drop}><Text>Drop files here</Text></View>

<Button onpress={() => files.pickFiles({ multiple: true })}><Text>Choose files…</Text></Button>
```

The dialogs work on both, with the same paths. Only a drop differs.

`examples/demo` builds for both: `bun examples/demo/build.mjs` for the window, and with `--web`
for the browser.

## File dialogs

`files.pickFiles()` and `files.pickSavePath()` work in a browser as they do in the window: they open a dialog
and resolve with real paths on the machine the host runs on, or with `null` if the user cancels, and what is
chosen may be read from then on, whatever the app's file limit says. The window shows them with GPUI. The headless
host has no window, so it asks the desktop to show its own file chooser: on Linux the `FileChooser` of
xdg-desktop-portal, over D-Bus, which the portal's backend (GTK, KDE, …) draws. The call runs on a thread and
answers like any other, so nothing waits for the user.

- `pickFiles({ multiple?, directories?, prompt?, filters? })`: `directories` chooses folders, `multiple` more than
  one, `prompt` is the button's text.
- `pickSavePath({ directory?, name?, filters? })`: `directory` is where it starts, `name` the file name it
  suggests.
- `filters` is `[{ name?, extensions: ["png", ".jpg", "*.gif"] }]`, which groups the files the dialog offers. The
  window's dialog does not take filters, so they only apply here, and an app that needs one on both checks the
  extension of what it is given.

The dialog is the desktop's, so it appears where the desktop shows dialogs, which is on the machine that runs the
host. That is the machine of a browser tab served from `127.0.0.1`. Without a portal, or with no session bus (a
service, a container), the call rejects with an `Error` that says there is no file chooser, with the code `Other`.
Other systems than Linux have none yet, and reject the same way.

## Development

`buildWeb({ watch: true })` rebuilds when a source file changes. Reload the page to see it. The page keeps
its token for the tab, so a reload reconnects. A host started again makes a new token, which the new
address carries.
