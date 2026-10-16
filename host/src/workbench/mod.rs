//! The window's one GPUI entity: it owns the app, hands input to it, and draws what it built.

mod clock;
mod editing;
mod file_drop;
mod ime;
mod io;
mod keys;
mod overlay;
mod pointer;
mod reload;
mod reports;
mod script;
mod scrolling;
mod windowing;

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use gpui::{Context, IntoElement, Render, Window, div, point, prelude::*, px};

use crate::diagnostics::Stats;
use crate::errors::AppError;
use crate::images::Images;
use crate::render::{self, Interaction, Painter};
use crate::runtime::{App as Svelte, Outcome};
use crate::tree::{Command, NodeId, Tree};
use crate::window_control::WindowState;

pub use overlay::ErrorOutput;

pub struct Workbench {
    pub(crate) svelte: Svelte,
    pub(crate) interaction: Interaction,
    seen_version: u64,
    pub(crate) stats: Stats,
    /// The node that had focus at the last tick.
    focused: Option<NodeId>,
    /// Each scroller's offset and furthest offset, as the app was last told: `(x, y, max x, max y)`.
    last_scroll: HashMap<NodeId, (f32, f32, f32, f32)>,
    sent_sizes: HashMap<NodeId, (f32, f32, f32, f32)>,
    /// What the app was last told about the window, so it hears only changes.
    reported_window: Option<WindowState>,
    /// Where the app's errors go: an overlay over it, or a log file.
    error_output: ErrorOutput,
    /// Why the last build failed, while the app still runs the build before it.
    build_error: Option<AppError>,
    /// How many errors the app had run into when the window last looked.
    errors_seen: u64,
    /// Whether the user has dismissed the overlay since the last new error.
    errors_dismissed: bool,
    /// Which run of the app this is. A reload starts a new one, and whatever the old one
    /// had in flight is ignored when it arrives.
    generation: u64,
    /// The source each `img` has been told about, so `load` and `error` fire once.
    announced_images: HashMap<NodeId, String>,
    /// Requests for elements that did not exist yet when the app asked, with tries left.
    queued: Vec<(Command, u8)>,
    /// When each frame began, while the `frames:N` step is measuring throughput.
    frame_times: Vec<Instant>,
    /// How many frames a throughput test still wants, set by the `frames:N` step.
    frame_target: usize,
    /// The element the `poke:` step has changed on every tick, as hovering one would.
    poked: Option<NodeId>,
    /// The element a button was pressed on, which hears the pointer until it is released.
    captured: Option<NodeId>,
    /// Whether a tick waits for the display's next frame.
    frame_tick_pending: bool,
    /// When the earliest tick waiting on a timer fires.
    timer_due: Option<Instant>,
    /// Files being dragged in from another program, and what they are over.
    external_drag: file_drop::ExternalDrag,
}

impl Workbench {
    pub fn new(svelte: Svelte, images: Images, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let interaction = Interaction::new(images, cx);
        window.focus(&interaction.root_focus);
        let mut workbench = Workbench {
            svelte,
            interaction,
            seen_version: 0,
            stats: Stats::default(),
            focused: None,
            last_scroll: HashMap::new(),
            sent_sizes: HashMap::new(),
            reported_window: None,
            error_output: ErrorOutput::Overlay,
            build_error: None,
            errors_seen: 0,
            errors_dismissed: false,
            generation: 0,
            announced_images: HashMap::new(),
            queued: Vec::new(),
            frame_times: Vec::new(),
            frame_target: 0,
            poked: None,
            captured: None,
            frame_tick_pending: false,
            timer_due: None,
            external_drag: Default::default(),
        };
        workbench.listen_for_io(cx);
        workbench.watch_window(window, cx);
        workbench
    }

    /// Reports an event on a node to the app, then redraws.
    pub(crate) fn emit(&mut self, id: NodeId, kind: &str, detail: String, cx: &mut Context<Self>) -> Outcome {
        let started = Instant::now();
        let before = self.svelte.counters.borrow().operations;
        let outcome = self.svelte.event(id, kind, &detail);
        self.write_requested_clipboard(cx);
        self.run_main_jobs(cx);
        self.run_scroll_commands(cx);
        self.stats.events += 1;
        self.stats.event_seconds += started.elapsed().as_secs_f64();
        self.flush_dirty(cx);
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

    /// Draws one view: what is under `root`, inside the box the outer layout gave it.
    pub fn build_view(
        &mut self,
        root: NodeId,
        inherited: render::Inherited,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let tree = self.svelte.tree.clone();
        let tree = tree.borrow();
        let painter = Painter::new(&tree, &self.interaction, Some(root));
        painter.view_content(root, inherited, cx)
    }

    /// The view an element is drawn in, when it is drawn in one.
    fn enclosing_view(&self, tree: &Tree, id: NodeId) -> Option<NodeId> {
        let roots = self.interaction.view_roots.borrow();
        let mut current = Some(id);
        while let Some(node) = current {
            if roots.contains(&node) {
                return Some(node);
            }
            current = tree.nodes.get(&node).and_then(|found| found.parent);
        }
        None
    }

    /// Has one view drawn again.
    fn dirty_view(&self, root: NodeId, cx: &mut Context<Self>) {
        let entity = self.interaction.views.borrow().get(&root).cloned();
        if let Some(entity) = entity {
            entity.update(cx, |_, cx| cx.notify());
        }
    }

    /// Has every view that sits under `id` drawn again.
    fn dirty_views_under(&self, tree: &Tree, id: NodeId, cx: &mut Context<Self>) {
        let mut pending = vec![id];
        while let Some(next) = pending.pop() {
            if self.interaction.view_roots.borrow().contains(&next) {
                self.dirty_view(next, cx);
                continue;
            }
            if let Some(node) = tree.nodes.get(&next) {
                pending.extend(node.children.iter().copied());
            }
        }
    }

    /// Has every view drawn again.
    fn dirty_all_views(&self, cx: &mut Context<Self>) {
        let roots: Vec<NodeId> = self.interaction.views.borrow().keys().copied().collect();
        for root in roots {
            self.dirty_view(root, cx);
        }
    }

    /// Something about this element changed: it, and the view it is in, are drawn again.
    fn touch(&mut self, id: NodeId, cx: &mut Context<Self>) {
        let tree = self.svelte.tree.clone();
        let tree = tree.borrow();
        if let Some(root) = self.enclosing_view(&tree, id) {
            self.dirty_view(root, cx);
        }
        cx.notify();
    }

    /// Has drawn again whatever the app's last changes to the tree touched.
    fn flush_dirty(&mut self, cx: &mut Context<Self>) {
        let (dirty, deep) = {
            let mut tree = self.svelte.tree.borrow_mut();
            (std::mem::take(&mut tree.dirty), std::mem::take(&mut tree.dirty_deep))
        };
        if dirty.is_empty() && deep.is_empty() {
            return;
        }
        let tree = self.svelte.tree.clone();
        let tree = tree.borrow();
        let mut redraw: HashSet<NodeId> = HashSet::new();
        for id in dirty.iter().chain(deep.iter()) {
            if let Some(root) = self.enclosing_view(&tree, *id) {
                redraw.insert(root);
            }
        }
        for root in redraw {
            self.dirty_view(root, cx);
        }
        // What an attribute or a style changes may reach everything below it.
        for id in deep {
            let own_view = self.enclosing_view(&tree, id);
            if own_view.is_none() || own_view == Some(id) {
                self.dirty_views_under(&tree, id, cx);
            }
        }
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
                self.caret_moved(*id, cx);
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
                let Some(handle) = self.interaction.scroll.borrow().get(id).cloned() else {
                    return false;
                };
                handle.set_offset(point(px(-left), px(-top)));
                // A view that holds the scroller keeps what it drew until it is told otherwise.
                self.touch(*id, cx);
                true
            }
        }
    }
}

impl Render for Workbench {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let started = Instant::now();
        let viewport = window.viewport_size();
        self.interaction.viewport.set((f32::from(viewport.width), f32::from(viewport.height)));
        self.interaction.scale.set(window.scale_factor());
        let gone: Vec<_> = std::mem::take(&mut self.svelte.tree.borrow_mut().dropped_canvases);
        for surface in gone {
            for image in surface.borrow_mut().release() {
                window.drop_image(image).ok();
            }
        }
        let tree = self.svelte.tree.clone();
        let tree = tree.borrow();
        let painter = Painter::new(&tree, &self.interaction, None);
        let document = painter.document(cx);
        let document = match self.error_overlay(window, cx) {
            Some(overlay) => div().size_full().relative().child(document).child(overlay).into_any_element(),
            None => document,
        };
        let wanted = self.frame_target;
        if wanted > 0 && self.frame_times.len() < wanted {
            self.frame_times.push(Instant::now());
            window.request_animation_frame();
        }
        if self.interaction.animating.replace(false) {
            window.request_animation_frame();
        }
        // What this frame laid out is reported, and animation frames run, when the
        // display next draws.
        self.tick_next_frame(window, cx);
        let elapsed = started.elapsed().as_secs_f64();
        {
            let mut frames = self.svelte.frames.borrow_mut();
            frames.push_back((Instant::now(), elapsed * 1000.0));
            if frames.len() > 1000 {
                frames.pop_front();
            }
        }
        self.stats.renders += 1;
        self.stats.render_seconds += elapsed;
        self.stats.worst_render_seconds = self.stats.worst_render_seconds.max(elapsed);
        document
    }
}
