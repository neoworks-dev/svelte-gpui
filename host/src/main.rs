//! svelte-gpui: runs a compiled Svelte app in QuickJS and draws what it builds
//! with GPUI. The Svelte side keeps its own tree of primitives and sends only what
//! changes; this side keeps a copy, styles each primitive from the object the app
//! gave it, and paints it.

mod assets;
mod canvas;
mod diagnostics;
mod field;
mod images;
mod io;
mod render;
mod runtime;
mod style;
mod tree;
mod workbench;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

use diagnostics::{heap_megabytes, memory_megabytes};
use images::Images;
use runtime::App as Svelte;
use workbench::Workbench;

fn flag(name: &str) -> Option<String> {
    let arguments: Vec<String> = std::env::args().collect();
    let position = arguments.iter().position(|argument| argument == name)?;
    arguments.get(position + 1).cloned()
}

fn flags(name: &str) -> Vec<String> {
    let arguments: Vec<String> = std::env::args().collect();
    let mut found = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        if argument == name {
            if let Some(value) = arguments.get(index + 1) {
                found.push(value.clone());
            }
        }
    }
    found
}

fn number_flag(name: &str, default: u64) -> u64 {
    flag(name).and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn main() {
    let Some(bundle_path) = flag("--bundle") else {
        eprintln!("usage: svelte-gpui --bundle app.js [--width N --height N]");
        std::process::exit(2);
    };
    if !std::path::Path::new(&bundle_path).exists() {
        panic!("cannot read {bundle_path}");
    }

    let before = memory_megabytes();
    println!("malloc at start: {:.1} MB in use", heap_megabytes().0);
    let started = Instant::now();
    let svelte = Svelte::start(&bundle_path);
    println!("malloc after the bundle ran: {:.1} MB in use", heap_megabytes().0);
    println!(
        "mounted in {:.0} ms, {} nodes; memory {:.1} MB RSS before, {:.1} after",
        started.elapsed().as_secs_f64() * 1000.0,
        svelte.tree.borrow().nodes.len(),
        before.0,
        memory_megabytes().0
    );
    // Image paths are relative to the bundle, unless `--assets` names another directory.
    let assets_dir = match flag("--assets") {
        Some(path) => PathBuf::from(path),
        None => std::path::Path::new(&bundle_path).parent().map(PathBuf::from).unwrap_or_default(),
    };
    let images = Images::new(assets_dir);
    // Scripted steps for when nobody can click: `click:TEXT`, `focus:TEXT`, `scroll:TEXT:PIXELS`.
    let mut steps = flags("--do");
    for text in flags("--click-text") {
        steps.push(format!("click:{text}"));
    }
    let exit_after = flag("--exit-after-ms").and_then(|value| value.parse::<u64>().ok());
    let report = std::env::args().any(|argument| argument == "--report");

    Application::new().with_assets(assets::Assets).run(move |cx: &mut App| {
        cx.text_system().add_fonts(assets::fonts()).expect("fonts load");
        println!("malloc after the fonts: {:.1} MB in use", heap_megabytes().0);
        let bounds = Bounds::centered(
            None,
            size(px(number_flag("--width", 668) as f32), px(number_flag("--height", 821) as f32)),
            cx,
        );
        let window = cx
            .open_window(
                WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
                |window, cx| cx.new(|cx| Workbench::new(svelte, images, window, cx)),
            )
            .unwrap();
        cx.activate(true);

        // Network, socket and file calls finish on other threads; each result goes to
        // the app as soon as it arrives.
        let receiver = window.update(cx, |workbench, _, _| workbench.svelte.io.take_receiver()).ok().flatten();
        if let Some(mut receiver) = receiver {
            cx.spawn(async move |cx| {
                use futures::StreamExt;
                while let Some(completion) = receiver.next().await {
                    let delivered = cx.update(|cx| {
                        window.update(cx, |workbench, _window, cx| workbench.deliver(completion, cx)).ok()
                    });
                    if !matches!(delivered, Ok(Some(()))) {
                        return;
                    }
                }
            })
            .detach();
        }

        // The app's clock runs on display frames and timers (see `Workbench::schedule`).
        // This slow beat only picks up what finishes off the main thread, like images.
        cx.spawn(async move |cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(100)).await;
                let outcome =
                    cx.update(|cx| window.update(cx, |workbench, window, cx| workbench.tick(false, window, cx)).ok());
                if outcome.is_err() {
                    return;
                }
            }
        })
        .detach();

        // Scripted clicks, one after another, for when nobody can click.
        if !steps.is_empty() {
            cx.spawn(async move |cx| {
                for step in steps {
                    cx.background_executor().timer(Duration::from_millis(900)).await;
                    cx.update(|cx| {
                        window.update(cx, |workbench, window, cx| workbench.run_step(&step, window, cx)).ok();
                    })
                    .ok();
                }
            })
            .detach();
        }

        if let Some(milliseconds) = exit_after {
            cx.spawn(async move |cx| {
                cx.background_executor().timer(Duration::from_millis(milliseconds)).await;
                cx.update(|cx| {
                    window.update(cx, |workbench, _window, _cx| workbench.print_summary(report)).ok();
                })
                .ok();
                cx.update(|cx| cx.quit()).ok();
            })
            .detach();
        }
    });
}
