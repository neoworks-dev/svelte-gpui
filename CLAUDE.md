# svelte-gpui — working notes

Svelte 5 compiled and run in QuickJS, drawn by a Rust GPUI host. See README.md.

- The JS runtime (`runtime/`) mirrors Svelte's DOM mutations as operations. The
  Rust host (`host/`) keeps a copy of the tree and paints it. Operation codes in
  `runtime/host/bridge.ts` and `host/src/tree.rs` must stay in step.
- `firstChild` and `nextSibling` must be prototype accessors in `nodes.ts`:
  Svelte's `init_operations()` pulls the getters off `Node.prototype` once.
- Svelte delegates `click` and other common events to a listener on the mount
  root; `host/events.ts` walks a synthetic event up the tree for it.
- The host build reuses the GPUI artifacts of the grove `gpui-spike` worktree via
  `CARGO_TARGET_DIR`; `host/vendor/xattr` patches a `libc` incompatibility.
- Tests for the host are screenshots on a headless Hyprland output
  (`hyprctl output create headless`), with `--click-text` standing in for a click.
