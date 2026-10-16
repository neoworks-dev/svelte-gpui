//! Starting an app: what the host needs to know ([`Options`]), and [`run`], which opens
//! the window and runs until it closes.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use gpui::{App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, prelude::*, px, size};

use crate::assets;
use crate::diagnostics::{heap_megabytes, memory_megabytes};
use crate::embedded::Embedded;
use crate::errors;
use crate::images::Images;
use crate::io::Commands;
use crate::runtime::{App as Svelte, Script};
use crate::watch;
use crate::window_control::WindowSettings;
use crate::workbench::{ErrorOutput, Workbench};

/// The window's size when neither a flag nor the app says.
const DEFAULT_WIDTH: f32 = 668.0;
const DEFAULT_HEIGHT: f32 = 821.0;

/// Everything about how to run an app.
#[derive(Clone, Debug)]
pub struct Options {
    /// The built app: one script.
    pub bundle: PathBuf,
    /// Where relative image paths are resolved; the bundle's directory by default.
    pub assets: Option<PathBuf>,
    /// The window's size in logical pixels, when set. It overrides the size the app asks
    /// for with `start(..., { window })`; without either, the window is 668 by 821.
    pub width: Option<f32>,
    pub height: Option<f32>,
    /// The window's app id, which the desktop matches window rules against. It overrides a
    /// packaged app's own id; tests set it so the window can be kept out of the way.
    pub app_id: Option<String>,
    /// Paths file access is narrowed to, on top of the app's own limit.
    pub allow_files: Vec<String>,
    /// Scripted input steps, for testing (see docs/testing.md).
    pub steps: Vec<String>,
    /// Quit after this long and print what the run cost.
    pub exit_after_ms: Option<u64>,
    /// With `exit_after_ms`, also print an outline of the element tree.
    pub report: bool,
    /// Watch the bundle, and start it again in the same window whenever a build writes it.
    pub watch: bool,
    /// Development: errors show in an overlay over the app, with stacks mapped back to its
    /// sources, and the bundle runs from its text so they have line numbers. Otherwise they
    /// are logged to a file in the cache directory and the window keeps going. On with `--dev`
    /// and with `--watch`.
    pub dev: bool,
    /// An app packed into the executable. When set, it runs instead of `bundle`, and `watch`
    /// does nothing. See [`Options::embedded`].
    pub embedded: Option<Embedded>,
    /// What the app can call in Rust, and the events Rust can push to it (docs/rust.md).
    pub commands: Commands,
}

impl Options {
    /// Options for a bundle, with everything else as the defaults.
    pub fn new(bundle: impl Into<PathBuf>) -> Self {
        Options {
            bundle: bundle.into(),
            assets: None,
            width: None,
            height: None,
            app_id: None,
            allow_files: Vec::new(),
            steps: Vec::new(),
            exit_after_ms: None,
            report: false,
            watch: false,
            dev: false,
            embedded: None,
            commands: Commands::default(),
        }
    }

    /// Options for an app packed into the executable (see [`embed!`](crate::embed)). The
    /// command line is still read, for the flags that suit testing, such as `--exit-after-ms`
    /// and `--do`; `--bundle` runs that file instead of the packed app.
    pub fn embedded(embedded: Embedded) -> Self {
        Self::read_command_line(Some(embedded)).expect("an embedded app needs no bundle flag")
    }

    /// These options with `commands` as what the app can call in Rust.
    pub fn commands(mut self, commands: Commands) -> Self {
        self.commands = commands;
        self
    }

    /// Reads the host's command line: `--bundle`, `--width`, `--height`, `--app-id`, `--assets`,
    /// `--allow-files`, `--do`, `--exit-after-ms`, `--report`, `--watch` and `--dev`. `None` without a bundle.
    pub fn from_command_line() -> Option<Self> {
        Self::read_command_line(None)
    }

    fn read_command_line(embedded: Option<Embedded>) -> Option<Self> {
        let arguments: Vec<String> = std::env::args().collect();
        let all = |name: &str| -> Vec<String> {
            let positions = arguments.iter().enumerate().filter(|(_, argument)| *argument == name);
            positions.filter_map(|(index, _)| arguments.get(index + 1).cloned()).collect()
        };
        let one = |name: &str| all(name).into_iter().next();
        let number = |name: &str| one(name).and_then(|value| value.parse::<f32>().ok());
        let mut options = match (one("--bundle"), embedded) {
            (Some(bundle), _) => Options::new(bundle),
            (None, Some(embedded)) => Options { embedded: Some(embedded), ..Options::new("") },
            (None, None) => return None,
        };
        options.assets = one("--assets").map(PathBuf::from);
        options.width = number("--width");
        options.height = number("--height");
        options.app_id = one("--app-id");
        options.allow_files = all("--allow-files");
        options.steps = all("--do");
        options.steps.extend(all("--click-text").into_iter().map(|text| format!("click:{text}")));
        options.exit_after_ms = one("--exit-after-ms").and_then(|value| value.parse().ok());
        options.report = arguments.iter().any(|argument| argument == "--report");
        options.watch = arguments.iter().any(|argument| argument == "--watch");
        options.dev = options.watch || arguments.iter().any(|argument| argument == "--dev");
        Some(options)
    }
}

/// Runs the app until its window closes.
pub fn run(options: Options) {
    let source = Source::find(&options);

    let before = memory_megabytes();
    println!("malloc at start: {:.1} MB in use", heap_megabytes().0);
    let started = Instant::now();
    let svelte = Svelte::start_with(&source.bundle_path, source.script, &options.allow_files, options.commands.clone());
    println!("malloc after the bundle ran: {:.1} MB in use", heap_megabytes().0);
    println!(
        "mounted in {:.0} ms, {} nodes; memory {:.1} MB RSS before, {:.1} after",
        started.elapsed().as_secs_f64() * 1000.0,
        svelte.tree.borrow().nodes.len(),
        before.0,
        memory_megabytes().0
    );
    let images = Images::new(source.assets_dir);
    let steps = options.steps.clone();
    let exit_after = options.exit_after_ms;
    let report = options.report;
    let settings = svelte.window_settings.borrow().clone().unwrap_or_default();
    let error_output = match options.dev {
        true => ErrorOutput::Overlay,
        false => ErrorOutput::Log(errors::log_path(&app_name(&options))),
    };

    Application::new().with_assets(assets::Assets).run(move |cx: &mut App| {
        cx.text_system().add_fonts(assets::fonts()).expect("fonts load");
        println!("malloc after the fonts: {:.1} MB in use", heap_megabytes().0);
        let opening = open_options(&options, settings, cx);
        let title = opening.titlebar.as_ref().and_then(|titlebar| titlebar.title.clone());
        let window = cx.open_window(opening, |window, cx| cx.new(|cx| Workbench::new(svelte, images, window, cx)));
        let window = window.unwrap();
        window.update(cx, |workbench, _, _| workbench.send_errors_to(error_output)).ok();
        // Wayland takes the title from the window, not from the options it opens with.
        if let Some(title) = title {
            window.update(cx, |_, window, _| window.set_window_title(&title)).ok();
        }
        cx.activate(true);
        if options.watch && options.embedded.is_none() {
            watch::reload_on_change(window, options.bundle.clone(), options.allow_files.clone(), cx);
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

/// Where an app's script and files are.
struct Source {
    /// A file in the bundle's directory, which is what `files.paths().bundle` reports.
    bundle_path: String,
    /// Where relative image paths are resolved.
    assets_dir: PathBuf,
    script: Script,
}

impl Source {
    /// Finds the app, unpacking an embedded one. Stops the process if there is none.
    fn find(options: &Options) -> Source {
        if let Some(embedded) = &options.embedded {
            let unpacked = embedded.unpack().unwrap_or_else(|error| panic!("{error}"));
            return Source {
                bundle_path: unpacked.directory.join("bundle.js").to_string_lossy().to_string(),
                assets_dir: options.assets.clone().unwrap_or(unpacked.directory),
                script: unpacked.script,
            };
        }
        let bundle_path = options.bundle.to_string_lossy().to_string();
        if !options.bundle.exists() {
            panic!("cannot read {bundle_path}");
        }
        // Image paths are relative to the bundle, unless `assets` names another directory.
        let assets_dir = match options.assets.clone() {
            Some(path) => path,
            None => options.bundle.parent().map(PathBuf::from).unwrap_or_default(),
        };
        let script =
            if options.dev { Script::Development(bundle_path.clone()) } else { Script::File(bundle_path.clone()) };
        Source { script, bundle_path, assets_dir }
    }
}

/// What the app is called in the cache directory: the id of a packaged app, or the bundle's name.
fn app_name(options: &Options) -> String {
    let packaged = options.embedded.and_then(|embedded| embedded.metadata().ok()).map(|metadata| metadata.id);
    packaged.unwrap_or_else(|| {
        let stem = options.bundle.file_stem().map(|stem| stem.to_string_lossy().to_string());
        stem.unwrap_or_else(|| "app".to_string())
    })
}

/// How the window opens: the host's flags first, then what the app asked for, then the defaults.
fn open_options(options: &Options, settings: WindowSettings, cx: &mut App) -> WindowOptions {
    let width = options.width.or(settings.width).unwrap_or(DEFAULT_WIDTH);
    let height = options.height.or(settings.height).unwrap_or(DEFAULT_HEIGHT);
    let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
    let mut opening = WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() };
    if let Some(title) = settings.title {
        opening.titlebar = Some(TitlebarOptions { title: Some(title.into()), ..Default::default() });
    }
    if let Some(metadata) = options.embedded.and_then(|embedded| embedded.metadata().ok()) {
        // The id lets the desktop match the window to the app's `.desktop` file and icon.
        opening.app_id = Some(metadata.id);
        if opening.titlebar.as_ref().and_then(|titlebar| titlebar.title.as_ref()).is_none() {
            let title = metadata.title.unwrap_or(metadata.name);
            opening.titlebar = Some(TitlebarOptions { title: Some(title.into()), ..Default::default() });
        }
    }
    if let Some(app_id) = &options.app_id {
        opening.app_id = Some(app_id.clone());
    }
    if settings.min_width.is_some() || settings.min_height.is_some() {
        let minimum = size(px(settings.min_width.unwrap_or(0.0)), px(settings.min_height.unwrap_or(0.0)));
        opening.window_min_size = Some(minimum);
    }
    opening
}
