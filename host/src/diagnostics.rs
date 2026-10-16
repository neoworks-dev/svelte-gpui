//! What the host reports about itself: memory, timings, and an outline of the tree.

use crate::render;
use crate::tree::{Kind, NodeId, ROOT, Tree};
use crate::workbench::Workbench;

/// Resident and proportional memory of this process in megabytes, from /proc.
pub fn memory_megabytes() -> (f64, f64) {
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
    (read("/proc/self/status", "VmRSS:"), read("/proc/self/smaps_rollup", "Pss:"))
}

#[derive(Default)]
pub struct Stats {
    pub events: u32,
    pub event_seconds: f64,
    pub renders: u32,
    pub render_seconds: f64,
    pub worst_render_seconds: f64,
}

/// How many nodes sit under `id`, itself included.
fn subtree_size(tree: &Tree, id: NodeId) -> usize {
    let Some(node) = tree.nodes.get(&id) else {
        return 0;
    };
    1 + node.children.iter().map(|child| subtree_size(tree, *child)).sum::<usize>()
}

/// The outline of the tree down to `depth`, with what each branch weighs, for finding what is slow.
fn print_outline(tree: &Tree, id: NodeId, depth: usize, limit: usize) {
    let Some(node) = tree.nodes.get(&id) else {
        return;
    };
    let Kind::Element { tag } = &node.kind else {
        return;
    };
    let size = subtree_size(tree, id);
    if size < 40 {
        return;
    }
    let look = node.attributes.get("look").map(String::as_str).unwrap_or("");
    println!("{}{} {} {}", "  ".repeat(depth), size, tag, &look[..look.len().min(90)]);
    if depth < limit {
        for child in &node.children {
            print_outline(tree, *child, depth + 1, limit);
        }
    }
}

pub fn print_report(tree: &Tree) {
    let builds = render::VIEW_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    let nanos = render::VIEW_NANOS.load(std::sync::atomic::Ordering::Relaxed);
    println!("views: {builds} built, {:.2} ms each", nanos as f64 / builds.max(1) as f64 / 1e6);
    println!("nodes in the host tree: {}", tree.nodes.len());
    if std::env::var("SVELTE_GPUI_OUTLINE").is_ok() {
        print_outline(tree, ROOT, 0, 14);
    }
    let mut tags: std::collections::BTreeMap<&str, usize> = Default::default();
    for node in tree.nodes.values() {
        if let Kind::Element { tag } = &node.kind {
            *tags.entry(tag.as_str()).or_default() += 1;
        }
    }
    println!("tags: {}", tags.iter().map(|(tag, count)| format!("{tag}={count}")).collect::<Vec<_>>().join(" "));
}

/// What the allocator holds: bytes in use by the program, and bytes it keeps free.
pub fn heap_megabytes() -> (f64, f64) {
    // SAFETY: mallinfo2 only reads the allocator's own counters.
    let info = unsafe { libc::mallinfo2() };
    (info.uordblks as f64 / 1048576.0, info.fordblks as f64 / 1048576.0)
}

impl Workbench {
    /// What the run cost, printed when `--exit-after-ms` ends it; `--report` adds an
    /// outline of the tree.
    pub fn print_summary(&self, report: bool) {
        let (resident, proportional) = memory_megabytes();
        println!("memory: {resident:.1} MB RSS, {proportional:.1} MB PSS");
        let (used, free) = heap_megabytes();
        println!("malloc: {used:.1} MB in use, {free:.1} MB free inside the heap");
        let stats = &self.stats;
        let counters = self.svelte.counters.borrow();
        println!("{}", self.svelte.memory_breakdown());
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
            print_report(&self.svelte.tree.borrow());
        }
    }
}
