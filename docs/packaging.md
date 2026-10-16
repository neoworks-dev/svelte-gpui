# Packaging

`scripts/package.mjs` turns an app into **one executable** with the bundle and its
files inside. It runs from anywhere, with no flags and nothing beside it.

```
bun scripts/package.mjs --app examples/demo
bun scripts/package.mjs --app examples/rust-commands
```

The result is in `dist/package/<app>/`:

| File | |
|---|---|
| `<executable>` | The app. Stripped, about 34 MB: most of it is GPUI. |
| `<id>.desktop` | The launcher entry. |
| `<id>.svg` or `<id>.png` | The icon, as `metadata.icon` gave it. |
| `<executable>-<version>-linux-<machine>.tar.gz` | The three files above in one directory, for sharing. `--no-tar` skips it. |

Other flags: `--out DIR` for somewhere other than `dist/package`, and `--no-strip` to keep
the symbols.

Linux only for now; see [Other platforms](#other-platforms).

## The app's metadata

An app describes itself in the `"svelte-gpui"` key of its `package.json`, or in a
`svelte-gpui.json` beside it (which wins if both exist):

```json
{
  "name": "svelte-gpui-demo",
  "version": "0.1.0",
  "svelte-gpui": {
    "name": "svelte-gpui demo",
    "id": "dev.neoworks.svelte-gpui.demo",
    "icon": "icon.svg",
    "description": "Everything svelte-gpui can do, in one app.",
    "bundle": "dist/demo.js",
    "build": "build.mjs"
  }
}
```

Paths are relative to the app's directory.

| Key | Meaning |
|---|---|
| `name` | Required. What people see: the launcher entry's name, and the window's title unless `title` or the app says otherwise. Defaults to `package.json`'s `name`. |
| `id` | Required. A reverse-domain identifier, such as `dev.example.my-app`. It is the window's application id (which is how the desktop matches the window to the `.desktop` file and its icon), the `.desktop` and icon file names, and the name of the cache directory the app's files unpack to. |
| `bundle` | Required. The script the build produces. Everything else in its directory (images, say) is packaged too, with the same relative paths, apart from `.qbc`, `.map`, `.tmp` and `.meta.json` files. |
| `version` | Defaults to `package.json`'s `version`, then `0.0.0`. |
| `build` | The script that builds the bundle, run with `bun` in the app's directory. Default `build.mjs`. `false` skips it. |
| `icon` | An SVG or PNG. |
| `title` | The window's title, if it differs from `name`. `start(App, {}, { window: { title } })` takes precedence. |
| `description` | The launcher entry's comment. |
| `categories` | The launcher entry's categories. Default `Utility;`. |
| `executable` | The executable's name. Default: `name` in lower case with dashes. |
| `crate` | Present when the app has its own Rust crate (see below). |

## What goes into the executable

1. **The bundle**, compiled to QuickJS bytecode without its source text or line tables
   (`svelte-gpui --compile-only app.js app.qbc` does the compiling). The app starts from
   memory with no parsing, and a stack trace has no line numbers. If the bundle does not
   compile, its source is embedded instead, and the packager says so.
2. **The files beside the bundle**, as one archive.
3. **The metadata**, so the window gets its title and application id.

The packager writes these to `target/package/<app>/embed/` and builds the executable with
them included (`include_bytes!`, through `svelte_gpui::embed!()`).

### Where the files go at run time

The bundle runs from the executable. **The files cannot**: GPUI reads images from files,
so on its first start the app unpacks them to
`$XDG_CACHE_HOME/<id>/files-<hash>/` (`~/.cache/<id>/...`), where `<hash>` identifies the
files' content. That costs about 40 ms once; every later start finds the directory ready
and does nothing. A new version with different files unpacks to a new directory and
removes the old ones. Unpacking goes to a temporary directory that is renamed into
place, so an interrupted start leaves nothing half-written.

That directory is the app's **bundle directory**:

- `Image` paths such as `"assets/photo.png"` resolve against it, as they resolve against
  the bundle's directory in development.
- `files.paths().bundle` and `$bundle` in a file limit are that directory, so
  `${bundle}/assets/photo.png` works in both cases.

An app with no files beside its bundle unpacks nothing, and its bundle directory is the
executable's.

## An app with no Rust of its own

`examples/demo` is one. The packager generates a crate in `target/package/<app>/crate`
whose whole program is

```rust
fn main() {
    svelte_gpui::run(svelte_gpui::Options::embedded(svelte_gpui::embed!()));
}
```

and builds it, against the host library by path and with the host's `Cargo.lock`.

## An app with its own Rust crate

An app that registers [commands](rust.md) is built from its own crate. Add `crate` to the
metadata, and a `package` feature to `Cargo.toml`:

```json
"crate": { "features": ["package"] }
```

```toml
[features]
package = []
```

and pick the options by feature in `main.rs`:

```rust
#[cfg(feature = "package")]
fn options() -> svelte_gpui::Options {
    svelte_gpui::Options::embedded(svelte_gpui::embed!())
}

#[cfg(not(feature = "package"))]
fn options() -> svelte_gpui::Options {
    svelte_gpui::Options::from_command_line().expect("usage: my-app --bundle dist/app.js")
}
```

The packager builds the crate (`crate.manifest` names another `Cargo.toml` than the one in
the app's directory) with `--features package` and `SVELTE_GPUI_EMBED_DIR` set to the
directory it wrote. `embed!()` reads its three files from there at compile time, which is
why it fails to compile without the variable: keep it behind the feature, so
`cargo run` in development needs nothing.

`Options::embedded` still reads the command line, for flags that suit testing
(`--exit-after-ms`, `--do`, `--allow-files`, `--width`, `--height`, `--app-id`), and `--bundle FILE`
runs that file instead of the embedded app. A packaged app therefore takes no flags, but
accepts them.

`examples/rust-commands` is set up this way.

## Installing

An executable runs wherever it is. To make it a launcher entry on a Linux desktop, copy
the three files to where the desktop looks:

```
install -Dm755 svelte-gpui-demo ~/.local/bin/svelte-gpui-demo
install -Dm644 dev.neoworks.svelte-gpui.demo.desktop ~/.local/share/applications/
install -Dm644 dev.neoworks.svelte-gpui.demo.svg ~/.local/share/icons/hicolor/scalable/apps/
```

The `.desktop` file's `Exec=` is the executable's bare name, so it is found on `PATH`
(`~/.local/bin` usually is). Change it to a full path to install elsewhere. The icon is
found by the `Icon=` name, which is the app's `id`.

## Building

The packager builds with cargo, so it follows `CARGO_TARGET_DIR`. Without it, it builds
into the checkout's own `target/`, which `scripts/host.sh --seed` first fills with the GPUI
artifacts it can reuse. The first package of an
app compiles the app's crate and the host library again (about ten seconds with GPUI
built); the packaged app does not need cargo or bun to run.

## Other platforms

Only Linux is done: the packager, the `.desktop` file and the cache directory (which
reads `XDG_CACHE_HOME` and `HOME`) are Linux's.

- **macOS** needs a `.app` directory: `Contents/MacOS/<executable>`, an `Info.plist`
  (`CFBundleIdentifier` from `id`, `CFBundleName`, `CFBundleShortVersionString`), the
  icon as an `.icns`, and signing and notarization to run on other machines. The
  executable is built for `aarch64-apple-darwin` and `x86_64-apple-darwin`, or both with
  `lipo`. The files can unpack to `~/Library/Caches/<id>`, or be read from
  `Contents/Resources`, which also removes the unpacking.
- **Windows** needs an `.exe` with its icon as a resource (an `.ico`, linked by a build
  script), an installer or a zip, and signing. The files can unpack to
  `%LOCALAPPDATA%\<id>`. A GUI-subsystem executable keeps a console from opening.

Neither is tested, and the host itself has only been run on Linux with Wayland.
