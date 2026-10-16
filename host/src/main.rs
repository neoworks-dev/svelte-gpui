//! The `svelte-gpui` command: runs a bundle, `svelte-gpui --bundle app.js`.

fn main() {
    let Some(options) = svelte_gpui::Options::from_command_line() else {
        eprintln!("usage: svelte-gpui --bundle app.js [--width N --height N]");
        std::process::exit(2);
    };
    svelte_gpui::run(options);
}
