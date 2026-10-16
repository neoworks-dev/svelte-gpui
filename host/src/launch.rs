//! Starting an app: what the host needs to know ([`Options`]), and [`run`], which opens
//! the window and runs until it closes.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

use crate::assets;
use crate::diagnostics::{heap_megabytes, memory_megabytes};
use crate::images::Images;
use crate::runtime::App as Svelte;
use crate::workbench::Workbench;

/// Everything about how to run an app.
#[derive(Clone, Debug)]
pub struct Options {
    /// The built app: one script.
    pub bundle: PathBuf,
    /// Where relative image paths are resolved; the bundle's directory by default.
    pub assets: Option<PathBuf>,
    /// The window's size in logical pixels.
    pub width: f32,
    pub height: f32,
    /// Paths file access is narrowed to, on top of the app's own limit.
    pub allow_files: Vec<String>,
    /// Scripted input steps, for testing (see docs/testing.md).
    pub steps: Vec<String>,
    /// Quit after this long and print what the run cost.
    pub exit_after_ms: Option<u64>,
    /// With `exit_after_ms`, also print an outline of the element tree.
    pub report: bool,
}

impl Options {
    /// Options for a bundle, with everything else as the defaults.
    pub fn new(bundle: impl Into<PathBuf>) -> Self {
        Options {
            bundle: bundle.into(),
            assets: None,
            width: 668.0,
            height: 821.0,
            allow_files: Vec::new(),
            steps: Vec::new(),
            exit_after_ms: None,
            report: false,
        }
    }

    /// Reads the host's command line: `--bundle`, `--width`, `--height`, `--assets`,
    /// `--allow-files`, `--do`, `--exit-after-ms` and `--report`. `None` without a bundle.
    pub fn from_command_line() -> Option<Self> {
        let arguments: Vec<String> = std::env::args().collect();
        let all = |name: &str| -> Vec<String> {
            let positions = arguments.iter().enumerate().filter(|(_, argument)| *argument == name);
            positions.filter_map(|(index, _)| arguments.get(index + 1).cloned()).collect()
        };
        let one = |name: &str| all(name).into_iter().next();
        let number = |name: &str| one(name).and_then(|value| value.parse::<f32>().ok());
        let mut options = Options::new(one("--bundle")?);
        options.assets = one("--assets").map(PathBuf::from);
        options.width = number("--width").unwrap_or(options.width);
        options.height = number("--height").unwrap_or(options.height);
        options.allow_files = all("--allow-files");
        options.steps = all("--do");
        options.steps.extend(all("--click-text").into_iter().map(|text| format!("click:{text}")));
        options.exit_after_ms = one("--exit-after-ms").and_then(|value| value.parse().ok());
        options.report = arguments.iter().any(|argument| argument == "--report");
        Some(options)
    }
}

/// Runs the app until its window closes.
pub fn run(options: Options) {
    let bundle_path = options.bundle.to_string_lossy().to_string();
    if !options.bundle.exists() {
        panic!("cannot read {bundle_path}");
    }

    let before = memory_megabytes();
    println!("malloc at start: {:.1} MB in use", heap_megabytes().0);
    let started = Instant::now();
    let svelte = Svelte::start(&bundle_path, &options.allow_files);
    println!("malloc after the bundle ran: {:.1} MB in use", heap_megabytes().0);
    println!(
        "mounted in {:.0} ms, {} nodes; memory {:.1} MB RSS before, {:.1} after",
        started.elapsed().as_secs_f64() * 1000.0,
        svelte.tree.borrow().nodes.len(),
        before.0,
        memory_megabytes().0
    );
    // Image paths are relative to the bundle, unless `assets` names another directory.
    let assets_dir = match options.assets.clone() {
        Some(path) => path,
        None => options.bundle.parent().map(PathBuf::from).unwrap_or_default(),
    };
    let images = Images::new(assets_dir);
    let steps = options.steps.clone();
    let exit_after = options.exit_after_ms;
    let report = options.report;

    Application::new().with_assets(assets::Assets).run(move |cx: &mut App| {
        cx.text_system().add_fonts(assets::fonts()).expect("fonts load");
        println!("malloc after the fonts: {:.1} MB in use", heap_megabytes().0);
        let bounds = Bounds::centered(None, size(px(options.width), px(options.height)), cx);
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
