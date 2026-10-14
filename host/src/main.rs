//! svelte-gpui: runs a compiled Svelte app in QuickJS and draws what it builds
//! with GPUI. The Svelte side keeps its own tree and sends only what changes;
//! this side keeps a copy, styles it from a Tailwind stylesheet, and paints it.

mod assets;
mod css;
mod forms;
mod images;
mod render;
mod runtime;
mod select;
mod style;
mod tree;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    App, Application, Bounds, Context, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Render,
    Window, WindowBounds, WindowOptions, point, prelude::*, px, size,
};

use css::Stylesheet;
use images::{ImageState, Images};
use render::{Interaction, Painter};
use runtime::{App as Svelte, Outcome};
use style::Resolver;
use tree::{Command, Kind, NodeId, ROOT, Tree};

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
    interaction: Interaction,
    started: Instant,
    seen_version: u64,
    stats: Stats,
    /// The node that had focus at the last tick.
    focused: Option<NodeId>,
    last_scroll: HashMap<NodeId, (f32, f32)>,
    sent_sizes: HashMap<NodeId, (f32, f32)>,
    window_size: (f32, f32),
    /// The source each `img` has been told about, so `load` and `error` fire once.
    announced_images: HashMap<NodeId, String>,
    /// Requests for elements that did not exist yet when the app asked, with tries left.
    queued: Vec<(Command, u8)>,
}

impl Workbench {
    /// Reports an event on a node to the app, then redraws.
    fn emit(&mut self, id: NodeId, kind: &str, detail: String, cx: &mut Context<Self>) -> Outcome {
        let started = Instant::now();
        let before = self.svelte.counters.borrow().operations;
        let outcome = self.svelte.event(id, kind, &detail);
        self.write_requested_clipboard(cx);
        self.stats.events += 1;
        self.stats.event_seconds += started.elapsed().as_secs_f64();
        if std::env::var("SVELTE_GPUI_TRACE").is_ok() {
            let after = self.svelte.counters.borrow().operations;
            println!(
                "{kind} on node {id}: {} operations, {:.2} ms",
                after - before,
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        cx.notify();
        outcome
    }

    /// Copies what the app just asked to, while the input that caused it is still current:
    /// Wayland only accepts a clipboard write that follows a real input event.
    fn write_requested_clipboard(&mut self, cx: &mut Context<Self>) {
        let mut writes: Vec<String> = Vec::new();
        self.svelte.tree.borrow_mut().commands.retain(|command| match command {
            Command::WriteClipboard(text) => {
                writes.push(text.clone());
                false
            }
            _ => true,
        });
        for text in writes {
            self.write_clipboard(text, cx);
        }
    }

    /// The pointer entered or left an element that tracks it, which restyles it.
    fn hover_changed(&mut self, id: NodeId, hovered: bool, cx: &mut Context<Self>) {
        let mut set = self.interaction.hovered.borrow_mut();
        let changed = if hovered { set.insert(id) } else { set.remove(&id) };
        drop(set);
        if changed {
            cx.notify();
        }
    }

    /// A plain click, which is what scripted runs use.
    fn click(&mut self, id: NodeId, cx: &mut Context<Self>) {
        self.emit(id, "click", "{}".to_string(), cx);
    }

    /// The element that has keyboard focus, if it is one the app made.
    fn focused_node(&self, window: &Window) -> Option<NodeId> {
        let handles = self.interaction.focus.borrow();
        for (id, handle) in handles.iter() {
            if handle.is_focused(window) {
                return Some(*id);
            }
        }
        None
    }

    /// A key press: first to the app, up from whatever has focus, and then, unless the
    /// app cancelled it, to the field that has focus or the clipboard shortcut it spells.
    fn key(&mut self, event: &KeyDownEvent, _down: bool, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let target = self.focused_node(window).unwrap_or(0);
        self.handle_key(target, &event.keystroke, event.is_held, false, cx)
    }

    /// The part of a key press after the target is known; scripted steps come here too.
    /// Returns whether the key was used, so it is not also typed.
    fn handle_key(
        &mut self,
        target: NodeId,
        keystroke: &Keystroke,
        held: bool,
        insert_text: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let command = keystroke.modifiers.control || keystroke.modifiers.platform;
        let clipboard_event = match (command, keystroke.key.as_str()) {
            (true, "c") => Some("copy"),
            (true, "x") => Some("cut"),
            (true, "v") => Some("paste"),
            _ => None,
        };
        let detail = key_detail(keystroke, held);
        if self.emit(target, "keydown", detail, cx).prevented {
            return true;
        }
        if let Some(kind) = clipboard_event {
            self.clipboard_shortcut(kind, target, cx);
            return true;
        }
        if target != 0
            && (self.field_key(target, keystroke, insert_text, cx) || self.select_key(target, keystroke, cx))
        {
            cx.notify();
            return true;
        }
        false
    }

    /// `copy`, `cut` and `paste`: the app sees them first, and may take over.
    fn clipboard_shortcut(&mut self, kind: &str, target: NodeId, cx: &mut Context<Self>) {
        let clipboard = self.read_clipboard(cx);
        let selected = self.selected_text(target);
        let detail = serde_json::json!({ "clipboardText": clipboard }).to_string();
        let outcome = self.emit(target, kind, detail, cx);
        if outcome.prevented {
            if let Some(text) = outcome.clipboard {
                self.write_clipboard(text, cx);
            }
            return;
        }
        match (kind, selected) {
            ("copy", Some(text)) => self.write_clipboard(text, cx),
            ("cut", Some(text)) => {
                self.write_clipboard(text, cx);
                self.cut_selection(target, cx);
            }
            ("paste", _) => self.paste_into(target, &clipboard, cx),
            _ => {}
        }
    }

    fn key_up(&mut self, event: &KeyUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        let target = self.focused_node(window).unwrap_or(0);
        let detail = key_detail(&event.keystroke, false);
        self.emit(target, "keyup", detail, cx);
    }

    /// Advances the app's clock, carries requests out, reports what the window learned,
    /// and redraws if anything changed.
    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let milliseconds = self.svelte.started.elapsed().as_secs_f64() * 1000.0;
        let busy = self.svelte.pump(milliseconds);
        self.run_commands(window, cx);
        self.report_focus(window, cx);
        self.report_scroll(cx);
        self.report_sizes();
        self.report_images(cx);
        self.report_window(window);
        let version = self.svelte.tree.borrow().version;
        if version != self.seen_version {
            self.seen_version = version;
            cx.notify();
        }
        busy
    }

    /// Does what the app asked: focus an element, or scroll one.
    fn run_commands(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let fresh: Vec<Command> = std::mem::take(&mut self.svelte.tree.borrow_mut().commands);
        for command in fresh {
            self.queued.push((command, 20));
        }
        let queued = std::mem::take(&mut self.queued);
        for (command, tries) in queued {
            if !self.carry_out(&command, window, cx) && tries > 0 {
                self.queued.push((command, tries - 1));
            }
        }
    }

    /// Whether the command could be carried out yet; the element may not have been drawn.
    fn carry_out(&mut self, command: &Command, window: &mut Window, cx: &mut Context<Self>) -> bool {
        match command {
            Command::WriteClipboard(text) => {
                self.write_clipboard(text.clone(), cx);
                true
            }
            Command::Select(id, start, end) => {
                if !self.interaction.fields.borrow().contains_key(id) {
                    return false;
                }
                self.select_range(*id, *start, *end);
                true
            }
            Command::Focus(id) => {
                let handles = self.interaction.focus.borrow();
                let Some(handle) = handles.get(id) else {
                    return false;
                };
                window.focus(handle);
                true
            }
            Command::ScrollTo(id, left, top) => {
                let handles = self.interaction.scroll.borrow();
                let Some(handle) = handles.get(id) else {
                    return false;
                };
                handle.set_offset(point(px(-left), px(-top)));
                true
            }
        }
    }

    /// Sends `blur` and `focus` when focus moved since the last tick.
    fn report_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = self.focused_node(window);
        if now == self.focused {
            return;
        }
        let before = self.focused;
        self.focused = now;
        if let Some(id) = before {
            self.commit_change(id, cx);
            self.emit(id, "blur", "{\"bubbles\":false}".to_string(), cx);
            self.emit(id, "focusout", "{}".to_string(), cx);
        }
        if let Some(id) = now {
            self.field_focused(id);
            self.emit(id, "focus", "{\"bubbles\":false}".to_string(), cx);
            self.emit(id, "focusin", "{}".to_string(), cx);
        }
    }

    /// Sends `scroll` for the scrollable elements that listen and have moved.
    fn report_scroll(&mut self, cx: &mut Context<Self>) {
        let mut moved: Vec<(NodeId, f32, f32, f32, f32)> = Vec::new();
        {
            let handles = self.interaction.scroll.borrow();
            let tree = self.svelte.tree.borrow();
            for (id, handle) in handles.iter() {
                let Some(node) = tree.nodes.get(id) else {
                    continue;
                };
                if !node.listeners.contains("scroll") {
                    continue;
                }
                let offset = handle.offset();
                let current = (-f32::from(offset.x), -f32::from(offset.y));
                let previous = self.last_scroll.get(id).copied().unwrap_or((0.0, 0.0));
                if (current.0 - previous.0).abs() < 0.5 && (current.1 - previous.1).abs() < 0.5 {
                    continue;
                }
                let content = handle.max_offset();
                moved.push((*id, current.0, current.1, f32::from(content.width), f32::from(content.height)));
            }
        }
        for (id, left, top, extra_width, extra_height) in moved {
            self.last_scroll.insert(id, (left, top));
            let detail = format!(
                "{{\"bubbles\":false,\"scrollLeft\":{left},\"scrollTop\":{top},\"scrollWidth\":{extra_width},\"scrollHeight\":{extra_height}}}"
            );
            self.emit(id, "scroll", detail, cx);
        }
    }

    /// Tells the app the size of every element it asked about, when it changed.
    fn report_sizes(&mut self) {
        let measured: Vec<(NodeId, (f32, f32))> = self
            .interaction
            .sizes
            .borrow()
            .iter()
            .map(|(id, size)| (*id, *size))
            .collect();
        for (id, size) in measured {
            let previous = self.sent_sizes.get(&id).copied();
            let unchanged = matches!(previous, Some(old) if (old.0 - size.0).abs() < 0.5 && (old.1 - size.1).abs() < 0.5);
            if unchanged || !self.svelte.tree.borrow().nodes.contains_key(&id) {
                continue;
            }
            self.sent_sizes.insert(id, size);
            self.svelte.size(id, size.0, size.1);
        }
    }

    /// Fires `load` or `error` on each image once its source has settled.
    fn report_images(&mut self, cx: &mut Context<Self>) {
        let mut outcomes: Vec<(NodeId, String, bool)> = Vec::new();
        {
            let tree = self.svelte.tree.borrow();
            for (id, node) in tree.nodes.iter() {
                let Kind::Element { tag } = &node.kind else {
                    continue;
                };
                if tag != "img" {
                    continue;
                }
                let Some(source) = node.attributes.get("src") else {
                    continue;
                };
                if self.announced_images.get(id) == Some(source) {
                    continue;
                }
                match self.interaction.images.state(source) {
                    ImageState::Loading => {}
                    ImageState::Ready(_) => outcomes.push((*id, source.clone(), true)),
                    ImageState::Failed => outcomes.push((*id, source.clone(), false)),
                }
            }
        }
        for (id, source, loaded) in outcomes {
            self.announced_images.insert(id, source);
            let kind = if loaded { "load" } else { "error" };
            self.emit(id, kind, "{\"bubbles\":false}".to_string(), cx);
        }
    }

    fn report_window(&mut self, window: &Window) {
        let size = window.viewport_size();
        let current = (f32::from(size.width), f32::from(size.height));
        if current == self.window_size {
            return;
        }
        self.window_size = current;
        self.svelte.window(current.0, current.1);
    }

    /// One scripted step: a click, a focus, or a scroll of the nearest scrollable ancestor.
    fn run_step(&mut self, step: &str, window: &mut Window, cx: &mut Context<Self>) {
        let mut parts = step.splitn(3, ':');
        let action = parts.next().unwrap_or("");
        let needle = parts.next().unwrap_or("");
        if action == "wait" {
            return;
        }
        let Some(id) = self.find_node(needle) else {
            eprintln!("nothing matches {needle:?}");
            return;
        };
        match action {
            "click" => self.click(id, cx),
            "focus" => {
                let command = Command::Focus(id);
                if !self.carry_out(&command, window, cx) {
                    self.queued.push((command, 20));
                }
            }
            "type" => {
                let command = Command::Focus(id);
                self.carry_out(&command, window, cx);
                for character in parts.next().unwrap_or("").chars() {
                    let text = character.to_string();
                    let keystroke = Keystroke {
                        modifiers: Default::default(),
                        key: text.clone(),
                        key_char: Some(text),
                    };
                    self.handle_key(id, &keystroke, false, true, cx);
                }
            }
            "compose" | "commit" => {
                let command = Command::Focus(id);
                self.carry_out(&command, window, cx);
                let text = parts.next().unwrap_or("");
                if action == "compose" {
                    <Self as gpui::EntityInputHandler>::replace_and_mark_text_in_range(self, None, text, None, window, cx);
                } else {
                    <Self as gpui::EntityInputHandler>::replace_text_in_range(self, None, text, window, cx);
                }
            }
            "open" => self.toggle_select(id, cx),
            "choose" => {
                let index: usize = parts.next().unwrap_or("0").parse().unwrap_or(0);
                self.choose_option(id, index, cx);
            }
            "key" => {
                let spelled = parts.next().unwrap_or("");
                match Keystroke::parse(spelled) {
                    Ok(keystroke) => {
                        self.handle_key(id, &keystroke, false, true, cx);
                    }
                    Err(_) => eprintln!("cannot read keystroke {spelled:?}"),
                }
            }
            "scroll" => {
                let pixels: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
                let target = self.scrollable_ancestor(id);
                self.queued.push((Command::ScrollTo(target, 0.0, pixels), 20));
            }
            _ => eprintln!("unknown step {step:?}"),
        }
        cx.notify();
    }

    /// The closest element at or above `id` that scrolls.
    fn scrollable_ancestor(&self, id: NodeId) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let handles = self.interaction.scroll.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            if handles.contains_key(&candidate) {
                return candidate;
            }
            current = tree.nodes.get(&candidate).and_then(|node| node.parent);
        }
        id
    }

    /// An element by its text, or by `@attribute=value` such as `@placeholder=Your name`.
    fn find_node(&self, needle: &str) -> Option<NodeId> {
        let Some(query) = needle.strip_prefix('@') else {
            return self.find_by_text(needle);
        };
        let (name, value) = query.split_once('=')?;
        let tree = self.svelte.tree.borrow();
        let mut matches: Vec<NodeId> = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.attributes.get(name).map(String::as_str) == Some(value))
            .map(|(id, _)| *id)
            .collect();
        matches.sort();
        matches.first().copied()
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let started = Instant::now();
        let tree = self.svelte.tree.clone();
        let tree = tree.borrow();
        let painter = Painter {
            tree: &tree,
            resolver: &self.resolver,
            interaction: &self.interaction,
        };
        let document = painter.document(cx);
        if self.interaction.animating.replace(false) {
            window.request_animation_frame();
        }
        let elapsed = started.elapsed().as_secs_f64();
        self.stats.renders += 1;
        self.stats.render_seconds += elapsed;
        self.stats.worst_render_seconds = self.stats.worst_render_seconds.max(elapsed);
        document
    }
}

/// The DOM's name for a key: `Enter`, `ArrowLeft`, or the character typed.
fn dom_key(keystroke: &Keystroke) -> String {
    let named = match keystroke.key.as_str() {
        "enter" => Some("Enter"),
        "escape" => Some("Escape"),
        "backspace" => Some("Backspace"),
        "delete" => Some("Delete"),
        "tab" => Some("Tab"),
        "left" => Some("ArrowLeft"),
        "right" => Some("ArrowRight"),
        "up" => Some("ArrowUp"),
        "down" => Some("ArrowDown"),
        "home" => Some("Home"),
        "end" => Some("End"),
        "pageup" => Some("PageUp"),
        "pagedown" => Some("PageDown"),
        "space" => Some(" "),
        _ => None,
    };
    if let Some(name) = named {
        return name.to_string();
    }
    let held = keystroke.modifiers.control || keystroke.modifiers.alt || keystroke.modifiers.platform;
    if let (Some(text), false) = (&keystroke.key_char, held) {
        return text.clone();
    }
    if keystroke.key.chars().count() == 1 && keystroke.modifiers.shift {
        return keystroke.key.to_uppercase();
    }
    keystroke.key.clone()
}

/// The DOM's `code` for a key, which names the physical key.
fn dom_code(keystroke: &Keystroke) -> String {
    let key = dom_key(keystroke);
    if key == " " {
        return "Space".to_string();
    }
    if keystroke.key.len() == 1 {
        let character = keystroke.key.chars().next().unwrap_or(' ');
        if character.is_ascii_alphabetic() {
            return format!("Key{}", character.to_ascii_uppercase());
        }
        if character.is_ascii_digit() {
            return format!("Digit{character}");
        }
    }
    key
}

fn key_detail(keystroke: &Keystroke, repeat: bool) -> String {
    let key = dom_key(keystroke).replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "{{\"key\":\"{key}\",\"code\":\"{}\",\"repeat\":{repeat},\"ctrlKey\":{},\"shiftKey\":{},\"altKey\":{},\"metaKey\":{}}}",
        dom_code(keystroke),
        keystroke.modifiers.control,
        keystroke.modifiers.shift,
        keystroke.modifiers.alt,
        keystroke.modifiers.platform
    )
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
    let mut controls: Vec<(&u32, &tree::Node)> = tree
        .nodes
        .iter()
        .filter(|(_, node)| matches!(&node.kind, Kind::Element { tag } if tag == "input" || tag == "select" || tag == "textarea"))
        .collect();
    controls.sort_by_key(|(id, _)| **id);
    for (id, node) in controls {
        let mut attributes: Vec<String> = node.attributes.iter().map(|(k, v)| format!("{k}={v:?}")).collect();
        attributes.sort();
        println!("  control {id}: {}", attributes.join(" "));
    }
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
    let assets_dir = match flag("--assets") {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir().unwrap_or_default(),
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
                |window, cx| {
                    cx.new(|cx| {
                        let interaction = Interaction {
                            root_focus: cx.focus_handle(),
                            focus: RefCell::new(HashMap::new()),
                            scroll: RefCell::new(HashMap::new()),
                            sizes: Rc::new(RefCell::new(HashMap::new())),
                            fields: Rc::new(RefCell::new(HashMap::new())),
                            bounds: Rc::new(RefCell::new(HashMap::new())),
                            open_select: RefCell::new(None),
                            hovered: RefCell::new(HashSet::new()),
                            animations: RefCell::new(HashMap::new()),
                            animating: Cell::new(false),
                            images,
                        };
                        window.focus(&interaction.root_focus);
                        Workbench {
                            svelte,
                            resolver,
                            interaction,
                            started: Instant::now(),
                            seen_version: 0,
                            stats: Stats::default(),
                            focused: None,
                            last_scroll: HashMap::new(),
                            sent_sizes: HashMap::new(),
                            window_size: (0.0, 0.0),
                            announced_images: HashMap::new(),
                            queued: Vec::new(),
                        }
                    })
                },
            )
            .unwrap();
        cx.activate(true);

        // The clock: timers and animation frames inside the app run off this.
        cx.spawn(async move |cx| {
            let mut pause = 33;
            loop {
                cx.background_executor().timer(Duration::from_millis(pause)).await;
                let outcome = cx.update(|cx| {
                    window.update(cx, |workbench, window, cx| workbench.tick(window, cx)).ok()
                });
                let Ok(busy) = outcome else {
                    return;
                };
                pause = if busy.unwrap_or(false) { 16 } else { 33 };
            }
        })
        .detach();

        // Scripted clicks, one after another, for when nobody can click.
        if !steps.is_empty() {
            cx.spawn(async move |cx| {
                for step in steps {
                    cx.background_executor().timer(Duration::from_millis(900)).await;
                    cx.update(|cx| {
                        window
                            .update(cx, |workbench, window, cx| workbench.run_step(&step, window, cx))
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
