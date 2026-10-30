//! The editor package in an app of its own. The host keeps the documents (`SharedDocuments`
//! registers their commands), and the UI shows one with `Editor` from `@neoworks/svelte-gpui-editor`.
//! It runs no language servers, so it passes `NoLanguageService`.
//!
//! ```text
//! bun examples/editor/build.mjs
//! cargo run --release -- --bundle examples/editor/dist/editor.js
//! ```

use svelte_gpui::gpui::AppContext as _;
use svelte_gpui::{Commands, Elements, Options};
use svelte_gpui_editor::{Editor, NoLanguageService, SharedDocuments, Signals};

fn main() {
    let commands = Commands::new();
    let documents = SharedDocuments::new(commands.emitter());
    let commands = documents.register(commands);
    let signals = Signals::default();
    let elements = Elements::new().register("editor", move |host, cx| {
        let documents = documents.clone();
        let signals = signals.clone();
        cx.new(|cx| Editor::new(documents, NoLanguageService::shared(), &signals, host, cx))
    });
    let options = Options::from_command_line().unwrap_or_else(|| {
        eprintln!("usage: editor-example --bundle examples/editor/dist/editor.js");
        std::process::exit(2);
    });
    svelte_gpui::run(options.commands(commands).elements(elements));
}
