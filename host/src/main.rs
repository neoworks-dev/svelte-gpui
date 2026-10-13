//! svelte-gpui: runs a compiled Svelte app in QuickJS and draws what it builds
//! with GPUI. The Svelte side keeps its own tree and sends only what changes;
//! this side keeps a copy, styles it from a Tailwind stylesheet, and paints it.

mod assets;
mod css;
mod render;
mod runtime;
mod style;
mod tree;

use std::time::{Duration, Instant};

use gpui::{
    App, Application, Bounds, Context, IntoElement, Render, Window, WindowBounds, WindowOptions,
    prelude::*, px, size,
};

use css::Stylesheet;
use render::Painter;
use runtime::App as Svelte;
use style::Resolver;
use tree::{Kind, NodeId, ROOT, Tree};

/// Resident and proportional memory of this process in megabytes, from /proc.
fn memory_megabytes() -> (f64, f64) {
    let read = |path: &str, key: &str| -> f64 {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix(key) {
                let kilobytes: f64 = rest.trim().trim_end_matches("kB").trim().parse().unwrap_or(0.0);
                return kilobytes / 1024.0;
            }
        }
        0.0
    };
    (
        read("/proc/self/status", "VmRSS:"),
        read("/proc/self/smaps_rollup", "Pss:"),
    )
}

#[derive(Default)]
struct Stats {
    events: u32,
    event_seconds: f64,
    renders: u32,
    render_seconds: f64,
    worst_render_seconds: f64,
}

struct Workbench {
    svelte: Svelte,
    resolver: Resolver,
    started: Instant,
    seen_version: u64,
    stats: Stats,
}

impl Workbench {
    /// Reports a click on a node to the app, then redraws.
    fn click(&mut self, id: NodeId, cx: &mut Context<Self>) {
        let started = Instant::now();
        let before = self.svelte.counters.borrow().operations;
        self.svelte.event(id, "click", "{}");
        if std::env::var("SVELTE_GPUI_TRACE").is_ok() {
            let after = self.svelte.counters.borrow().operations;
            println!("click on node {id}: {} operations, {:.2} ms", after - before, started.elapsed().as_secs_f64() * 1000.0);
        }
        self.stats.events += 1;
        self.stats.event_seconds += started.elapsed().as_secs_f64();
        cx.notify();
    }

    /// Advances the app's clock and redraws if anything changed.
    fn tick(&mut self, cx: &mut Context<Self>) {
        let milliseconds = self.started.elapsed().as_secs_f64() * 1000.0;
        self.svelte.pump(milliseconds);
        let version = self.svelte.tree.borrow().version;
        if version != self.seen_version {
            self.seen_version = version;
            cx.notify();
        }
    }

    /// The deepest element whose text contains `needle`, for scripted clicks.
    fn find_by_text(&self, needle: &str) -> Option<NodeId> {
        let tree = self.svelte.tree.borrow();
        let mut current = ROOT;
        if !tree.text_content(current).contains(needle) {
            return None;
        }
        loop {
            let node = tree.nodes.get(&current)?;
            let next = node.children.iter().find(|child| {
                matches!(tree.nodes.get(child).map(|n| &n.kind), Some(Kind::Element { .. }))
                    && tree.text_content(**child).contains(needle)
            });
            match next {
                Some(child) => current = *child,
                None => return Some(current),
            }
        }
    }
}

impl Render for Workbench {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let started = Instant::now();
        let tree = self.svelte.tree.clone();
        let tree = tree.borrow();
        let painter = Painter {
            tree: &tree,
            resolver: &self.resolver,
        };
        let document = painter.document(cx);
        let elapsed = started.elapsed().as_secs_f64();
        self.stats.renders += 1;
        self.stats.render_seconds += elapsed;
        self.stats.worst_render_seconds = self.stats.worst_render_seconds.max(elapsed);
        document
    }
}

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

fn print_report(resolver: &Resolver, tree: &Tree) {
    println!("nodes in the host tree: {}", tree.nodes.len());
    let skipped = resolver.sheet.skipped_selectors();
    println!("selectors left out: {}", skipped.len());
    for selector in skipped.iter().take(25) {
        println!("  {selector}");
    }
    let unsupported = resolver.unsupported();
    println!("declarations not honoured: {}", unsupported.len());
    for declaration in unsupported.iter().take(40) {
        println!("  {declaration}");
    }
}

fn main() {
    let Some(bundle_path) = flag("--bundle") else {
        eprintln!("usage: svelte-gpui --bundle app.js [--css app.css] [--width N --height N]");
        std::process::exit(2);
    };
    let bundle = std::fs::read_to_string(&bundle_path)
        .unwrap_or_else(|error| panic!("cannot read {bundle_path}: {error}"));
    let stylesheet = match flag("--css") {
        Some(path) => std::fs::read_to_string(&path).unwrap_or_default(),
        None => String::new(),
    };

    let before = memory_megabytes();
    let started = Instant::now();
    let svelte = Svelte::start(&bundle);
    println!(
        "mounted in {:.0} ms, {} nodes; memory {:.1} MB RSS before, {:.1} after",
        started.elapsed().as_secs_f64() * 1000.0,
        svelte.tree.borrow().nodes.len(),
        before.0,
        memory_megabytes().0
    );
    let resolver = Resolver::new(Stylesheet::parse(&stylesheet));
    let clicks = flags("--click-text");
    let exit_after = flag("--exit-after-ms").and_then(|value| value.parse::<u64>().ok());
    let report = std::env::args().any(|argument| argument == "--report");

    Application::new().with_assets(assets::Assets).run(move |cx: &mut App| {
        cx.text_system().add_fonts(assets::fonts()).expect("fonts load");
        let bounds = Bounds::centered(
            None,
            size(px(number_flag("--width", 668) as f32), px(number_flag("--height", 821) as f32)),
            cx,
        );
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_| Workbench {
                        svelte,
                        resolver,
                        started: Instant::now(),
                        seen_version: 0,
                        stats: Stats::default(),
                    })
                },
            )
            .unwrap();
        cx.activate(true);

        // The clock: timers and animation frames inside the app run off this.
        cx.spawn(async move |cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(33)).await;
                if cx.update(|cx| window.update(cx, |workbench, _window, cx| workbench.tick(cx)).ok()).is_err() {
                    return;
                }
            }
        })
        .detach();

        // Scripted clicks, one after another, for when nobody can click.
        if !clicks.is_empty() {
            cx.spawn(async move |cx| {
                for needle in clicks {
                    cx.background_executor().timer(Duration::from_millis(900)).await;
                    cx.update(|cx| {
                        window
                            .update(cx, |workbench, _window, cx| match workbench.find_by_text(&needle) {
                                Some(id) => workbench.click(id, cx),
                                None => eprintln!("no element says {needle:?}"),
                            })
                            .ok();
                    })
                    .ok();
                }
            })
            .detach();
        }

        if let Some(milliseconds) = exit_after {
            cx.spawn(async move |cx| {
                cx.background_executor().timer(Duration::from_millis(milliseconds)).await;
                let (resident, proportional) = memory_megabytes();
                println!("memory: {resident:.1} MB RSS, {proportional:.1} MB PSS");
                cx.update(|cx| {
                    window
                        .update(cx, |workbench, _window, _cx| {
                            let stats = &workbench.stats;
                            let counters = workbench.svelte.counters.borrow();
                            println!(
                                "{} events, {:.2} ms each; {} batches, {} operations",
                                stats.events,
                                stats.event_seconds / stats.events.max(1) as f64 * 1000.0,
                                counters.batches,
                                counters.operations
                            );
                            println!(
                                "{} renders, {:.2} ms each building elements, worst {:.2} ms",
                                stats.renders,
                                stats.render_seconds / stats.renders.max(1) as f64 * 1000.0,
                                stats.worst_render_seconds * 1000.0
                            );
                            if report {
                                print_report(&workbench.resolver, &workbench.svelte.tree.borrow());
                            }
                        })
                        .ok();
                })
                .ok();
                cx.update(|cx| cx.quit()).ok();
            })
            .detach();
        }
    });
}
