//! The `svelte-gpui` command: runs a bundle, `svelte-gpui --bundle app.js`.
//!
//! `svelte-gpui --compile-only app.js app.js.qbc` compiles a bundle to bytecode and exits,
//! which is what packaging does before it embeds the bundle.

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if let Some(position) = arguments.iter().position(|argument| argument == "--compile-only") {
        compile_only(&arguments[position + 1..]);
        return;
    }
    let Some(options) = svelte_gpui::Options::from_command_line() else {
        eprintln!("usage: svelte-gpui --bundle app.js [--width N --height N]");
        std::process::exit(2);
    };
    svelte_gpui::run(options);
}

fn compile_only(paths: &[String]) {
    let [source, output] = paths else {
        eprintln!("usage: svelte-gpui --compile-only app.js output.qbc");
        std::process::exit(2);
    };
    if let Err(message) = svelte_gpui::compile(source.as_ref(), output.as_ref()) {
        eprintln!("svelte-gpui: {message}");
        std::process::exit(1);
    }
}
