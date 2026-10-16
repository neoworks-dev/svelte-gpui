//! Draws the host's tree with GPUI. Every element is one of a fixed set of
//! primitives (`g-view`, `g-text`, `g-button`, `g-input`, `g-scroll`, `g-image`,
//! `g-icon`, `g-canvas`, `g-overlay`), styled by the object the app passed and drawn as a `div`.
//! An overlay is drawn in a layer above the rest (`overlay.rs`).
//!
//! Input goes the other way. Which events an element listens for decides which
//! GPUI handlers it gets, and each handler reports to the app by node id.

mod events;
mod field;
mod file_drop;
mod media;
mod motion;
mod overlay;
mod primitive;
mod text;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Instant;

use gpui::{
    AnyElement, AnyView, Bounds, Context, DragMoveEvent, Entity, ExternalPaths, FocusHandle, IntoElement,
    MouseMoveEvent, MouseUpEvent, Render, Rgba, ScrollHandle, WeakEntity, Window, canvas, div, prelude::*, px,
    relative,
};

use crate::field::Field;
use crate::images::Images;
use crate::style::{self, Dimension, SANS, Style};
use crate::tree::{Kind, Node, NodeId, ROOT, Tree};
use crate::workbench::Workbench;

use events::listens_for_pointer;
pub use events::{listens_for_drag, pointer_detail, takes_button};
pub use field::{caret_visible, next_blink};
use file_drop::external_drag_ended;
use media::icon_content;
use motion::Animation;
use primitive::compute;
pub use primitive::{Computed, Primitive, primitive_of};
pub use text::Inherited;
use text::{DEFAULT_FONT_SIZE, DEFAULT_TEXT, collapse, text_box, transform};

/// What lives between frames: the handles GPUI needs to keep focus and scroll
/// positions, what the last layout measured, and the images being loaded.
pub struct Interaction {
    pub root_focus: FocusHandle,
    pub focus: RefCell<HashMap<NodeId, FocusHandle>>,
    pub scroll: RefCell<HashMap<NodeId, ScrollHandle>>,
    /// Where each measured element came out in the window.
    pub bounds: Rc<RefCell<HashMap<NodeId, Bounds<gpui::Pixels>>>>,
    pub fields: Rc<RefCell<HashMap<NodeId, Field>>>,
    /// Elements the pointer is over, for hover styles that ease instead of snapping.
    pub hovered: RefCell<HashSet<NodeId>>,
    /// Transitions in flight, by element.
    pub animations: RefCell<HashMap<NodeId, Animation>>,
    /// Set while painting when something is still easing, so another frame is asked for.
    pub animating: Cell<bool>,
    /// The elements whose colours are still easing, which must be built again for the next frame.
    pub animating_ids: RefCell<HashSet<NodeId>>,
    /// Styles worked out for the current version of the tree; any change to it empties this.
    pub styles: RefCell<(u64, HashMap<NodeId, Rc<Computed>>)>,
    /// The window's size when the frame began, for deciding which images to load.
    pub viewport: Rc<Cell<(f32, f32)>>,
    /// When the focused field's caret last moved; it blinks from then on.
    pub caret_epoch: Cell<Instant>,
    /// Whether the caret was drawn on, the last time it was drawn.
    pub caret_on: Cell<bool>,
    /// The window's scale factor, for choosing how large to decode an image.
    pub scale: Cell<f32>,
    /// Set when an image has been measured for the first time or has moved, so one more frame follows.
    pub bounds_changed: Rc<RefCell<HashSet<NodeId>>>,
    /// The parts of the document drawn as views of their own, by their topmost element.
    pub views: RefCell<HashMap<NodeId, Entity<NodeView>>>,
    /// The elements that are the top of a view, as of the last time they were built.
    pub view_roots: RefCell<HashSet<NodeId>>,
    pub workbench: RefCell<Option<WeakEntity<Workbench>>>,
    pub images: Images,
}

impl Interaction {
    pub fn new(images: Images, cx: &mut Context<Workbench>) -> Self {
        Interaction {
            root_focus: cx.focus_handle(),
            focus: RefCell::new(HashMap::new()),
            scroll: RefCell::new(HashMap::new()),
            fields: Rc::new(RefCell::new(HashMap::new())),
            bounds: Rc::new(RefCell::new(HashMap::new())),
            hovered: RefCell::new(HashSet::new()),
            animations: RefCell::new(HashMap::new()),
            animating: Cell::new(false),
            animating_ids: RefCell::new(HashSet::new()),
            styles: RefCell::new((u64::MAX, HashMap::new())),
            viewport: Rc::new(Cell::new((0.0, 0.0))),
            caret_epoch: Cell::new(Instant::now()),
            caret_on: Cell::new(true),
            scale: Cell::new(1.0),
            bounds_changed: Rc::new(RefCell::new(HashSet::new())),
            views: RefCell::new(HashMap::new()),
            view_roots: RefCell::new(HashSet::new()),
            workbench: RefCell::new(Some(cx.weak_entity())),
            images,
        }
    }

    /// Forgets everything kept for the old app's elements, for a reload: the new tree
    /// numbers its elements from the start again, so nothing may carry over by id.
    pub fn reset(&self) {
        self.focus.borrow_mut().clear();
        self.scroll.borrow_mut().clear();
        self.bounds.borrow_mut().clear();
        self.fields.borrow_mut().clear();
        self.hovered.borrow_mut().clear();
        self.animations.borrow_mut().clear();
        self.animating.set(false);
        self.animating_ids.borrow_mut().clear();
        *self.styles.borrow_mut() = (u64::MAX, HashMap::new());
        self.caret_epoch.set(Instant::now());
        self.caret_on.set(true);
        self.bounds_changed.borrow_mut().clear();
        self.views.borrow_mut().clear();
        self.view_roots.borrow_mut().clear();
    }

    fn focus_handle(&self, id: NodeId, cx: &mut Context<Workbench>) -> FocusHandle {
        self.focus.borrow_mut().entry(id).or_insert_with(|| cx.focus_handle()).clone()
    }

    fn scroll_handle(&self, id: NodeId) -> ScrollHandle {
        self.scroll.borrow_mut().entry(id).or_insert_with(ScrollHandle::new).clone()
    }
}

const DEFAULT_BACKGROUND: Rgba = Rgba { r: 0.094, g: 0.094, b: 0.106, a: 1.0 };

/// A part of the document that is its own GPUI view. GPUI keeps what a view drew and its
/// layout until the view changes, so a change somewhere else in the window costs this part
/// nothing.
pub struct NodeView {
    pub workbench: WeakEntity<Workbench>,
    pub root: NodeId,
    pub inherited: Inherited,
}

/// How many views were built, and the nanoseconds that took, for the report.
pub static VIEW_BUILDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static VIEW_NANOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Render for NodeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(workbench) = self.workbench.upgrade() else {
            return div().into_any_element();
        };
        let root = self.root;
        let inherited = self.inherited.clone();
        let started = Instant::now();
        let element = workbench.update(cx, |workbench, cx| workbench.build_view(root, inherited, cx));
        VIEW_BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        VIEW_NANOS.fetch_add(started.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        element
    }
}

/// Parts of the document holding fewer nodes than this are not worth a view of their own.
const VIEW_MIN_WEIGHT: usize = 24;
/// Parts holding more than this are split further, so a change does not redraw too much.
const VIEW_MAX_WEIGHT: usize = 2500;

pub struct Painter<'a> {
    pub tree: &'a Tree,
    pub interaction: &'a Interaction,
    /// How many nodes each element holds, worked out as they are asked for.
    weights: RefCell<HashMap<NodeId, usize>>,
    /// The view being built, whose top element is drawn in place rather than as a view.
    /// Views do not nest: building one never starts another.
    inside_view: Option<NodeId>,
}

impl<'a> Painter<'a> {
    pub fn new(tree: &'a Tree, interaction: &'a Interaction, inside_view: Option<NodeId>) -> Self {
        Self { tree, interaction, weights: RefCell::new(HashMap::new()), inside_view }
    }

    /// How many nodes are under `id`, itself included.
    fn weight(&self, id: NodeId) -> usize {
        if let Some(known) = self.weights.borrow().get(&id) {
            return *known;
        }
        let Some(node) = self.tree.nodes.get(&id) else {
            return 0;
        };
        let weight = 1 + node.children.iter().map(|child| self.weight(*child)).sum::<usize>();
        self.weights.borrow_mut().insert(id, weight);
        weight
    }

    /// The style of a node, worked out once per version of the tree.
    fn computed(&self, id: NodeId) -> Rc<Computed> {
        let mut cache = self.interaction.styles.borrow_mut();
        if cache.0 != self.tree.version {
            cache.0 = self.tree.version;
            cache.1.clear();
        }
        if let Some(found) = cache.1.get(&id) {
            return found.clone();
        }
        let computed = match self.tree.nodes.get(&id) {
            Some(node) => Rc::new(compute(node)),
            None => Rc::new(Computed { style: Style::default(), hover: None, active: None, focus: None }),
        };
        cache.1.insert(id, computed.clone());
        computed
    }

    /// The whole document: the root's children, filling the window.
    pub fn document(&self, cx: &mut Context<Workbench>) -> AnyElement {
        let mut element = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(DEFAULT_BACKGROUND)
            .text_color(DEFAULT_TEXT)
            .font_family(SANS)
            .text_size(px(DEFAULT_FONT_SIZE))
            .line_height(relative(1.4))
            .track_focus(&self.interaction.root_focus)
            .on_key_down(cx.listener(|workbench, event, window, cx| {
                // A key the host or the app used must not also be typed by the input method.
                if workbench.key(event, window, cx) {
                    cx.stop_propagation();
                }
            }))
            .on_key_up(cx.listener(|workbench, event, window, cx| {
                workbench.key_up(event, window, cx);
            }))
            .capture_any_mouse_up(cx.listener(|workbench, event: &MouseUpEvent, _window, cx| {
                workbench.release_pointer(event, cx);
            }))
            .on_mouse_move(cx.listener(|workbench, event: &MouseMoveEvent, _window, cx| {
                workbench.captured_motion(event, cx);
            }))
            .on_drag_move(cx.listener(|workbench, event: &DragMoveEvent<ExternalPaths>, _window, cx| {
                let paths = event.drag(cx).paths().to_vec();
                workbench.external_drag_over(event.event.position, paths, cx);
            }))
            .on_drop(cx.listener(|workbench, dropped: &ExternalPaths, window, cx| {
                workbench.external_drop(window.mouse_position(), dropped.paths().to_vec(), cx);
            }))
            .child(external_drag_ended(cx));
        let inherited = Inherited::root();
        for child in self.children(ROOT, &inherited, false, cx) {
            element = element.child(child);
        }
        element.into_any_element()
    }

    /// The contents of a view: its top element, drawn to fill the box the outer layout gave it.
    pub fn view_content(&self, root: NodeId, inherited: Inherited, cx: &mut Context<Workbench>) -> AnyElement {
        match self.element(root, &inherited, cx) {
            Some(element) => element,
            None => div().into_any_element(),
        }
    }

    /// Whether the outer layout can size this element without looking inside it: its box
    /// comes from its parent (a share, a fixed length, stretching, or filling what is left).
    fn sized_by_parent(&self, id: NodeId, style: &Style) -> bool {
        if style.absolute == Some(true) || style.translate.is_some() || style.hidden == Some(true) {
            return false;
        }
        let Some(parent) = self.tree.nodes.get(&id).and_then(|node| node.parent) else {
            return false;
        };
        let parent_style = self.computed(parent).style.clone();
        let fixed =
            |dimension: Option<Dimension>| matches!(dimension, Some(Dimension::Px(_)) | Some(Dimension::Percent(_)));
        let nothing = |dimension: Option<Dimension>| matches!(dimension, Some(Dimension::Px(zero)) | Some(Dimension::Percent(zero)) if zero == 0.0);
        let stretches = matches!(style.align_self.or(parent_style.align_items), None | Some(style::Align::Stretch));
        let fills = style.grow.unwrap_or(0.0) > 0.0 && nothing(style.basis);
        let (main, main_minimum, cross) = if parent_style.row == Some(true) {
            (style.width, style.min_width, style.height)
        } else {
            (style.height, style.min_height, style.width)
        };
        let sized_by_basis = fixed(style.basis) && nothing(main_minimum);
        let main_ok = fixed(main) || sized_by_basis || (fills && nothing(main_minimum));
        let cross_ok = fixed(cross) || stretches;
        main_ok && cross_ok
    }

    /// The element as a cached view of its own, when it is big enough to be worth one and the
    /// layout around it can size it without it.
    fn view_element(
        &self,
        id: NodeId,
        style: &Style,
        inherited: &Inherited,
        cx: &mut Context<Workbench>,
    ) -> Option<AnyElement> {
        if self.inside_view.is_some() || std::env::var_os("SVELTE_GPUI_NO_VIEWS").is_some() {
            return None;
        }
        let weight = self.weight(id);
        if !(VIEW_MIN_WEIGHT..=VIEW_MAX_WEIGHT).contains(&weight) || !self.sized_by_parent(id, style) {
            return None;
        }
        let workbench = self.interaction.workbench.borrow().clone()?;
        let existing = self.interaction.views.borrow().get(&id).cloned();
        let entity = match existing {
            Some(entity) => entity,
            None => {
                let inherited = inherited.clone();
                let entity = cx.new(|_| NodeView { workbench, root: id, inherited });
                self.interaction.views.borrow_mut().insert(id, entity.clone());
                entity
            }
        };
        entity.update(cx, |view, cx| {
            if view.inherited != *inherited {
                view.inherited = inherited.clone();
                cx.notify();
            }
        });
        self.interaction.view_roots.borrow_mut().insert(id);
        let mut outline = style::apply(div().flex(), style);
        let refinement = outline.style().clone();
        Some(AnyView::from(entity).cached(refinement).into_any_element())
    }

    /// The children of a container. Whitespace between elements is dropped; other text
    /// outside a `g-text` is drawn as it is, which React Native would refuse.
    fn children(
        &self,
        parent: NodeId,
        inherited: &Inherited,
        _row: bool,
        cx: &mut Context<Workbench>,
    ) -> Vec<AnyElement> {
        let Some(node) = self.tree.nodes.get(&parent) else {
            return Vec::new();
        };
        let mut elements = Vec::new();
        for child in &node.children {
            let Some(child_node) = self.tree.nodes.get(child) else {
                continue;
            };
            match &child_node.kind {
                Kind::Text { data } => {
                    let text = collapse(data);
                    let text = text.trim();
                    if !text.is_empty() {
                        elements.push(text_box(transform(text, inherited), inherited.font_size));
                    }
                }
                Kind::Element { .. } => {
                    if let Some(element) = self.element(*child, inherited, cx) {
                        elements.push(element);
                    }
                }
            }
        }
        elements
    }

    /// One element, with its children, or nothing when it is not displayed.
    fn element(&self, id: NodeId, inherited: &Inherited, cx: &mut Context<Workbench>) -> Option<AnyElement> {
        let node = self.tree.nodes.get(&id)?;
        let Some(primitive) = primitive_of(node) else {
            if let Kind::Element { tag } = &node.kind {
                eprintln!("svelte-gpui: <{tag}> is not a primitive and is not drawn");
            }
            return None;
        };
        let mut computed = (*self.computed(id)).clone();
        if computed.style.hidden == Some(true) {
            return None;
        }
        if primitive == Primitive::Overlay {
            // An overlay is drawn in a layer of its own, never as a cached view in the flow.
            return self.overlay_element(id, node, &computed, inherited, cx);
        }
        let is_view_top = self.inside_view == Some(id);
        if !is_view_top {
            if let Some(view) = self.view_element(id, &computed.style, inherited, cx) {
                return Some(view);
            }
        }
        let eased = computed.style.transition.is_some();
        if eased {
            self.ease_style(id, &mut computed);
        }
        let style = &computed.style;
        let inherited = inherited.under(style);

        let mut element = style::apply(div().flex(), style);
        if is_view_top {
            // The outer layout sized this box; it fills it, and the margins are the outer layout's.
            element = element.w_full().h_full().m_0();
        }
        element = match primitive {
            Primitive::Text => self.text_content(element, id, node, style, &inherited),
            Primitive::Image => self.image_content(element, id, node, style),
            Primitive::Icon => icon_content(element, node, style, &inherited),
            Primitive::Input => self.field_content(element, id, node, style, &inherited, cx),
            Primitive::Canvas => self.canvas_content(element, id, style),
            Primitive::View | Primitive::Button | Primitive::Scroll | Primitive::Overlay => {
                let row = style.row == Some(true);
                element.children(self.children(id, &inherited, row, cx))
            }
        };
        if node.observed() || listens_for_pointer(node) || listens_for_drag(node) {
            element = element.child(self.measure(id));
        }
        Some(self.interactive(element, id, node, primitive, &computed, eased, cx))
    }

    /// An invisible layer that records where its parent came out.
    fn measure(&self, id: NodeId) -> AnyElement {
        let placed = self.interaction.bounds.clone();
        let changed = self.interaction.bounds_changed.clone();
        let viewport = self.interaction.viewport.clone();
        canvas(
            move |bounds, _, _| {
                let view = viewport.get();
                let before = placed.borrow_mut().insert(id, bounds);
                let was_near = before.map(|old| !is_far_away(old, view));
                if was_near != Some(!is_far_away(bounds, view)) {
                    changed.borrow_mut().insert(id);
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element()
    }
}

/// Whether a box is more than a screen away from the window.
fn is_far_away(bounds: Bounds<gpui::Pixels>, window: (f32, f32)) -> bool {
    let (width, height) = window;
    if width <= 0.0 || height <= 0.0 {
        return false;
    }
    f32::from(bounds.bottom()) < -height
        || f32::from(bounds.top()) > height * 2.0
        || f32::from(bounds.right()) < -width
        || f32::from(bounds.left()) > width * 2.0
}

fn attribute<'a>(node: &'a Node, name: &str) -> &'a str {
    match node.attributes.get(name) {
        Some(value) => value.as_str(),
        None => "",
    }
}
