# svelte-gpui-editor

A text editor for svelte-gpui apps, moved here from Grove's native app (`grove/native`) so other
apps can use it too. `examples/editor` is a complete app with it.

- `editor-core` is the editor without a window: the rope buffer, selections, history, motions, vim
  mode and tree-sitter highlighting. Nothing in it knows about GPUI.
- `lsp-client` is a language server client: JSON-RPC over stdio, document sync, and the parts of
  completion, diagnostics and hover that need no window.
- `view` (the `svelte-gpui-editor` crate) draws a document with GPUI, as the app element `editor`.
- `src` is the UI side: the `Editor` component and `gpuiDocumentHost()`, which opens, saves and
  watches the documents the host keeps.

An app takes the crates as git dependencies of this repository, pinned to the same revision as
`svelte_gpui_host`, and registers the documents' commands and the element:

```rust
let documents = SharedDocuments::new(commands.emitter());
let commands = documents.register(commands);
let signals = Signals::default();
let elements = Elements::new().register("editor", move |host, cx| {
    let (documents, signals) = (documents.clone(), signals.clone());
    cx.new(|cx| Editor::new(documents, NoLanguageService::shared(), &signals, host, cx))
});
```

An app that runs language servers implements `LanguageService` over them and passes that instead
of `NoLanguageService`; Grove's `LspStore` does.

`cargo test --workspace` in this folder runs the crates' tests.
