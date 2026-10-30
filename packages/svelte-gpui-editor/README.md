# svelte-gpui-editor

A text editor for svelte-gpui apps, moved here from Grove's native app (`grove/native`) so other
apps can use it too.

- `editor-core` is the editor without a window: the rope buffer, selections, history, motions, vim
  mode and tree-sitter highlighting. Nothing in it knows about GPUI.
- `lsp-client` is a language server client: JSON-RPC over stdio, document sync, and the parts of
  completion, diagnostics and hover that need no window.

An app takes them as git dependencies of this repository, pinned to the same revision as
`svelte_gpui_host`:

```toml
editor-core = { git = "https://github.com/neoworks-dev/svelte-gpui", rev = "…" }
```

`cargo test --workspace` in this folder runs both crates' tests.
