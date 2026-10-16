# svelte-gpui — working notes

Native apps from Svelte 5 components, run in QuickJS and drawn by a Rust GPUI host.
See README.md. svelte-gpui is its own project; Grove is one app that may use it.

- Apps are built from primitives (`runtime/ui/*.svelte`), each rendering one host
  tag (`g-view`, `g-text`, …) that `host/src/render/` draws. There is no HTML and
  no CSS: a tag the host does not know is not drawn. Host tags are prefixed because
  Svelte puts `<text>` and `<image>` in the SVG namespace.
- Styles are typed objects (`runtime/ui/style.ts`) sent as JSON in the `look`
  attribute; `host/src/style/parse.rs` reads the same property names. Keep the two in step.
- Operation codes live only in `protocol.json`; `host/build.rs` generates the Rust
  constants and `runtime/host/bridge.ts` imports the file.
- Host event names (`press`, `edit`, `key`, `down`, …) avoid the ones Svelte
  delegates (`click`, `input`, `keydown`, …), so handlers sit on their own element.
  Components pass handlers through an attribute spread so an absent one adds no
  listener: a listener is what gives an element its GPUI handler.
- `firstChild` and `nextSibling` must be prototype accessors in `nodes.ts`:
  Svelte's `init_operations()` pulls the getters off `Node.prototype` once.
- The host build reuses the GPUI artifacts of the grove `gpui-spike` worktree via
  `CARGO_TARGET_DIR`; `host/vendor/xattr` patches a `libc` incompatibility.
- Test on a headless Hyprland output with `scripts/shot.sh`, which creates the
  output and always names the window it moves. Scripted steps run 0.9 s apart, so
  wait long enough before the screenshot.
- The app's clock ticks on display frames while anything animates and on timers
  otherwise (`host/src/workbench/clock.rs`). Never pace it with a fixed interval: that
  caps motion below the display's refresh rate.
- The host's Rust is not formatted crate-wide; format only files you touch, with
  `rustfmt --edition 2024 --config max_width=120,use_small_heuristics=Max,skip_children=true`.
- `bun run typecheck` runs tsc and svelte-check.
- Network and files go through one path: `host.io(op, call, args, bytes)` from
  `runtime/io/channel.ts` to `host/src/io/`, where each area (`fetch.rs`,
  `websocket.rs`, `files.rs`) runs work on threads and answers through a `Sink`.
  Results wake the window at once (`Workbench::deliver`); never poll for them. Work
  that needs GPUI (file dialogs) is a `MainJob`, run in `workbench/io.rs`.
- User docs live in `docs/`, and the agent skill in `.claude/skills/svelte-gpui/`.
  When a primitive, style property, event or step changes, update both.
