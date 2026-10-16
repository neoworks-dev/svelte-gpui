//! Overlays: content drawn above everything else, placed against another element.
//!
//! A `g-overlay` is a view that is drawn in a layer of its own, after the rest of the window,
//! so nothing covers it and no ancestor's clipping cuts it. Its position comes from its
//! parent's box (or from a point in the window), the side and alignment the app asked for, and
//! the size the content came out at. Where the content would leave the window it is flipped
//! to the other side, and then shifted back inside. All of that is worked out here, after layout,
//! against the box the host has just measured, so the overlay never lags a frame behind the
//! element it is anchored to, and follows it while it scrolls.
//!
//! GPUI's own `anchored` element only switches between corners and cannot be told to leave a gap,
//! so the placement is done by [`place`], a function of plain numbers.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AlignItems, AnyElement, App, Bounds, Context, Display, Edges, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, JustifyContent, LayoutId, Length, MouseDownEvent, Pixels, Point, Position, Window,
    div, px, relative,
};

use super::events::button_number;
use super::text::Inherited;
use super::{Computed, Painter, Primitive, pointer_detail};
use crate::style;
use crate::tree::{Kind, Node, NodeId};
use crate::workbench::Workbench;

/// A rectangle in the window, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn right(&self) -> f32 {
        self.x + self.width
    }

    fn bottom(&self) -> f32 {
        self.y + self.height
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.bottom()
    }
}

impl From<Bounds<Pixels>> for Rect {
    fn from(bounds: Bounds<Pixels>) -> Self {
        Rect {
            x: f32::from(bounds.origin.x),
            y: f32::from(bounds.origin.y),
            width: f32::from(bounds.size.width),
            height: f32::from(bounds.size.height),
        }
    }
}

/// Which edge of the anchor the overlay sits against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    fn opposite(self) -> Side {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    fn vertical(self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }
}

/// Where along that edge the overlay lines up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

impl Alignment {
    fn opposite(self) -> Alignment {
        match self {
            Alignment::Start => Alignment::End,
            Alignment::Center => Alignment::Center,
            Alignment::End => Alignment::Start,
        }
    }
}

/// What the app asked for.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub side: Side,
    pub alignment: Alignment,
    /// The gap between the anchor and the overlay, along the side.
    pub offset: f32,
    /// A shift along the edge, toward the end.
    pub cross_offset: f32,
    /// Moves to the opposite side when the preferred one has too little room.
    pub flip: bool,
    /// Slides back inside the window when it would stick out.
    pub shift: bool,
    /// The least distance kept from the window's edges.
    pub margin: f32,
    /// The anchor is a point rather than a box, so alignment flips like the side does.
    pub at_point: bool,
}

/// The top left corner of an overlay of size `size` placed against `anchor` in a window of
/// size `window`, and the side it ended up on.
pub fn place(anchor: Rect, size: (f32, f32), window: (f32, f32), wanted: &Placement) -> (f32, f32, Side) {
    let side = chosen_side(anchor, size, window, wanted);
    let mut alignment = wanted.alignment;
    if wanted.flip && wanted.at_point {
        let (x, y) = corner(anchor, size, wanted, side, alignment);
        let (start, end, limit) = if side.vertical() { (x, x + size.0, window.0) } else { (y, y + size.1, window.1) };
        if start < wanted.margin || end > limit - wanted.margin {
            alignment = alignment.opposite();
        }
    }
    let (mut x, mut y) = corner(anchor, size, wanted, side, alignment);
    if wanted.shift {
        x = clamp_inside(x, size.0, window.0, wanted.margin);
        y = clamp_inside(y, size.1, window.1, wanted.margin);
    }
    (x, y, side)
}

/// The preferred side, or the opposite one when the preferred has too little room and the other has more.
fn chosen_side(anchor: Rect, size: (f32, f32), window: (f32, f32), wanted: &Placement) -> Side {
    if !wanted.flip {
        return wanted.side;
    }
    let room = |side: Side| match side {
        Side::Bottom => window.1 - wanted.margin - anchor.bottom() - wanted.offset,
        Side::Top => anchor.y - wanted.offset - wanted.margin,
        Side::Right => window.0 - wanted.margin - anchor.right() - wanted.offset,
        Side::Left => anchor.x - wanted.offset - wanted.margin,
    };
    let needed = if wanted.side.vertical() { size.1 } else { size.0 };
    let preferred = room(wanted.side);
    let other = room(wanted.side.opposite());
    if preferred < needed && other > preferred { wanted.side.opposite() } else { wanted.side }
}

/// Where the overlay's top left corner goes for a side and an alignment, before keeping it in the window.
fn corner(anchor: Rect, size: (f32, f32), wanted: &Placement, side: Side, alignment: Alignment) -> (f32, f32) {
    let along = |start: f32, length: f32, overlay: f32| match alignment {
        Alignment::Start => start + wanted.cross_offset,
        Alignment::Center => start + (length - overlay) / 2.0 + wanted.cross_offset,
        Alignment::End => start + length - overlay - wanted.cross_offset,
    };
    match side {
        Side::Bottom => (along(anchor.x, anchor.width, size.0), anchor.bottom() + wanted.offset),
        Side::Top => (along(anchor.x, anchor.width, size.0), anchor.y - wanted.offset - size.1),
        Side::Right => (anchor.right() + wanted.offset, along(anchor.y, anchor.height, size.1)),
        Side::Left => (anchor.x - wanted.offset - size.0, along(anchor.y, anchor.height, size.1)),
    }
}

/// Moves a span of `length` so it lies inside `0..limit` with `margin` to spare; one that is
/// too long for the room starts at the margin.
fn clamp_inside(start: f32, length: f32, limit: f32, margin: f32) -> f32 {
    start.min(limit - margin - length).max(margin)
}

thread_local! {
    /// Where each overlay's anchor was in the last frame, for deciding whether a press was on it.
    static ANCHORS: RefCell<HashMap<NodeId, Rect>> = RefCell::new(HashMap::new());
    /// How many overlays are being laid out around this call, which makes an overlay inside
    /// another one know it cannot be drawn in a layer of its own: GPUI does not nest those.
    static DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// The content of an overlay, which finds its place once layout has said how large it is.
struct Placed {
    content: AnyElement,
    placement: Placement,
    /// The anchor in window coordinates, written by [`Overlay`] when it knows it.
    anchor: Rc<Cell<Option<Rect>>>,
}

impl Element for Placed {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.content.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _state: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(anchor) = self.anchor.get() else {
            return;
        };
        let viewport = window.viewport_size();
        let window_size = (f32::from(viewport.width), f32::from(viewport.height));
        let size = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let (x, y, _) = place(anchor, size, window_size, &self.placement);
        let offset = Point { x: px(x - f32::from(bounds.origin.x)), y: px(y - f32::from(bounds.origin.y)) };
        let offset = Point { x: offset.x.round(), y: offset.y.round() };
        window.with_element_offset(offset, |window| self.content.prepaint(window, cx));
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _state: &mut (),
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.content.paint(window, cx);
    }
}

impl IntoElement for Placed {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

/// The element a `g-overlay` becomes. It takes no room where it sits: its layout box is its
/// parent's box, which is what it is anchored to. Its content is drawn later, in a layer above
/// the window, unless it sits inside another overlay, where it is drawn in place after the content around it.
pub struct Overlay {
    id: NodeId,
    placed: Option<AnyElement>,
    anchor: Rc<Cell<Option<Rect>>>,
    /// A fixed anchor in window coordinates, replacing the parent's box.
    at: Option<(f32, f32)>,
    layer: usize,
}

pub struct OverlayLayout {
    nested: bool,
}

impl Overlay {
    fn new(
        id: NodeId,
        content: AnyElement,
        placement: Placement,
        anchor: Rc<Cell<Option<Rect>>>,
        at: Option<(f32, f32)>,
        layer: usize,
    ) -> Self {
        let placed = Placed { content, placement, anchor: anchor.clone() }.into_any_element();
        Overlay { id, placed: Some(placed), anchor, at, layer }
    }
}

impl Element for Overlay {
    type RequestLayoutState = OverlayLayout;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, OverlayLayout) {
        let nested = DEPTH.with(|depth| depth.get()) > 0;
        DEPTH.with(|depth| depth.set(depth.get() + 1));
        let content = self.placed.as_mut().map(|placed| placed.request_layout(window, cx));
        DEPTH.with(|depth| depth.set(depth.get() - 1));
        // The box of the parent, with the content free to hang out of it at the top left.
        let zero: Length = px(0.).into();
        let style = gpui::Style {
            position: Position::Absolute,
            display: Display::Flex,
            inset: Edges { top: zero, right: zero, bottom: zero, left: zero },
            align_items: Some(AlignItems::FlexStart),
            justify_content: Some(JustifyContent::FlexStart),
            ..gpui::Style::default()
        };
        (window.request_layout(style, content, cx), OverlayLayout { nested })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut OverlayLayout,
        window: &mut Window,
        cx: &mut App,
    ) {
        let anchor = match self.at {
            Some((x, y)) => Rect { x, y, width: 0.0, height: 0.0 },
            None => bounds.into(),
        };
        self.anchor.set(Some(anchor));
        ANCHORS.with(|anchors| {
            let mut anchors = anchors.borrow_mut();
            // Overlays that went away leave their entries behind; the ones still open write theirs again.
            if anchors.len() > 256 {
                anchors.clear();
            }
            anchors.insert(self.id, anchor);
        });
        let Some(mut placed) = self.placed.take() else {
            return;
        };
        if state.nested {
            placed.prepaint(window, cx);
            self.placed = Some(placed);
        } else {
            window.defer_draw(placed, window.element_offset(), self.layer);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _state: &mut OverlayLayout,
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(placed) = self.placed.as_mut() {
            placed.paint(window, cx);
        }
    }
}

impl IntoElement for Overlay {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

fn number(node: &Node, name: &str, default: f32) -> f32 {
    node.attributes.get(name).and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn switch(node: &Node, name: &str, default: bool) -> bool {
    match node.attributes.get(name).map(String::as_str) {
        Some("false") => false,
        Some(_) => true,
        None => default,
    }
}

/// The placement an overlay's attributes describe.
fn placement_of(node: &Node, at_point: bool) -> Placement {
    let side = match node.attributes.get("side").map(String::as_str) {
        Some("top") => Side::Top,
        Some("left") => Side::Left,
        Some("right") => Side::Right,
        _ => Side::Bottom,
    };
    let alignment = match node.attributes.get("align").map(String::as_str) {
        Some("center") => Alignment::Center,
        Some("end") => Alignment::End,
        _ => Alignment::Start,
    };
    Placement {
        side,
        alignment,
        offset: number(node, "offset", 0.0),
        cross_offset: number(node, "cross-offset", 0.0),
        flip: switch(node, "flip", true),
        shift: switch(node, "shift", true),
        margin: number(node, "margin", 4.0),
        at_point,
    }
}

/// The fixed window position in `at="x,y"`.
fn point_of(node: &Node) -> Option<(f32, f32)> {
    let (x, y) = node.attributes.get("at")?.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

impl Painter<'_> {
    /// A `g-overlay`: its children, drawn in a layer above the window and placed against its
    /// parent's box. It draws nothing while it has no children, so an app can keep one mounted
    /// and fill it when it opens.
    pub(super) fn overlay_element(
        &self,
        id: NodeId,
        node: &Node,
        computed: &Computed,
        inherited: &Inherited,
        cx: &mut Context<Workbench>,
    ) -> Option<AnyElement> {
        let style = &computed.style;
        let inherited = inherited.under(style);
        let children = self.children(id, &inherited, style.row == Some(true), cx);
        if children.is_empty() {
            return None;
        }
        let at = point_of(node);
        let anchor = Rc::new(Cell::new(None));
        let mut content = style::apply(div().flex().flex_none(), style).children(children).child(self.measure(id));
        if node.attributes.contains_key("match-width") {
            content = content.min_w(relative(1.0));
        }
        if !node.attributes.contains_key("passthrough") {
            content = content.occlude();
        }
        if node.listeners.contains("outside") {
            content = content.on_mouse_down_out(cx.listener(move |workbench, event: &MouseDownEvent, _window, cx| {
                let button = button_number(event.button);
                workbench.pressed_outside_overlay(id, event.position, event.modifiers, button, cx);
            }));
        }
        let content = self.interactive(content, id, node, Primitive::View, computed, false, cx);
        let layer = number(node, "layer", 0.0).max(0.0) as usize;
        Some(Overlay::new(id, content, placement_of(node, at.is_some()), anchor, at, layer).into_any_element())
    }
}

impl Workbench {
    /// A button went down outside an overlay that listens for it. The app hears where, and whether
    /// that was on the overlay's anchor, so a press on the button that opens a popover does not
    /// close it just before the same press toggles it. A press inside an overlay opened from within this one
    /// is not outside either.
    pub(crate) fn pressed_outside_overlay(
        &mut self,
        id: NodeId,
        position: Point<Pixels>,
        modifiers: gpui::Modifiers,
        button: i32,
        cx: &mut Context<Self>,
    ) {
        let (x, y) = (f32::from(position.x), f32::from(position.y));
        if self.inside_nested_overlay(id, x, y) {
            return;
        }
        let anchor = ANCHORS.with(|anchors| anchors.borrow().get(&id).copied());
        let on_anchor = anchor.is_some_and(|anchor| anchor.contains(x, y));
        let mut detail = pointer_detail(id, position, modifiers, button, 1, self);
        detail.pop();
        detail.push_str(&format!(",\"anchor\":{on_anchor}}}"));
        self.emit(id, "outside", detail, cx);
    }

    /// What a real press does before the app hears of it, for scripted presses: every overlay
    /// that listens for outside presses and does not contain the point is told.
    pub(crate) fn simulate_press_outside(&mut self, position: Point<Pixels>, button: i32, cx: &mut Context<Self>) {
        let outside: Vec<NodeId> = {
            let tree = self.svelte.tree.borrow();
            let bounds = self.interaction.bounds.borrow();
            let mut found: Vec<NodeId> = tree
                .nodes
                .iter()
                .filter(|(_, node)| matches!(&node.kind, Kind::Element { tag } if tag == "g-overlay"))
                .filter(|(id, node)| {
                    node.listeners.contains("outside") && !bounds.get(id).is_some_and(|b| b.contains(&position))
                })
                .map(|(id, _)| *id)
                .collect();
            found.sort();
            found
        };
        for id in outside {
            self.pressed_outside_overlay(id, position, gpui::Modifiers::default(), button, cx);
        }
    }

    /// Whether a point is inside an overlay that sits under `id` in the tree.
    fn inside_nested_overlay(&self, id: NodeId, x: f32, y: f32) -> bool {
        let tree = self.svelte.tree.borrow();
        let bounds = self.interaction.bounds.borrow();
        let mut pending: Vec<NodeId> = tree.nodes.get(&id).map(|node| node.children.clone()).unwrap_or_default();
        while let Some(next) = pending.pop() {
            let Some(node) = tree.nodes.get(&next) else {
                continue;
            };
            let Kind::Element { tag } = &node.kind else {
                continue;
            };
            if tag == "g-overlay" && bounds.get(&next).is_some_and(|found| Rect::from(*found).contains(x, y)) {
                return true;
            }
            pending.extend(node.children.iter().copied());
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: (f32, f32) = (1000.0, 800.0);

    fn wanted(side: Side, alignment: Alignment) -> Placement {
        Placement {
            side,
            alignment,
            offset: 4.0,
            cross_offset: 0.0,
            flip: true,
            shift: true,
            margin: 8.0,
            at_point: false,
        }
    }

    fn button(x: f32, y: f32) -> Rect {
        Rect { x, y, width: 100.0, height: 30.0 }
    }

    #[test]
    fn sits_below_the_anchor_with_a_gap() {
        let (x, y, side) = place(button(200.0, 100.0), (150.0, 200.0), WINDOW, &wanted(Side::Bottom, Alignment::Start));
        assert_eq!((x, y, side), (200.0, 134.0, Side::Bottom));
    }

    #[test]
    fn centres_and_ends_along_the_edge() {
        let (x, _, _) = place(button(200.0, 100.0), (200.0, 50.0), WINDOW, &wanted(Side::Bottom, Alignment::Center));
        assert_eq!(x, 150.0);
        let (x, _, _) = place(button(200.0, 100.0), (200.0, 50.0), WINDOW, &wanted(Side::Bottom, Alignment::End));
        assert_eq!(x, 100.0);
    }

    #[test]
    fn flips_above_near_the_bottom_edge() {
        let (_, y, side) = place(button(200.0, 700.0), (150.0, 200.0), WINDOW, &wanted(Side::Bottom, Alignment::Start));
        assert_eq!((y, side), (496.0, Side::Top));
    }

    #[test]
    fn stays_put_when_neither_side_fits_better() {
        let (_, _, side) = place(button(200.0, 100.0), (150.0, 900.0), WINDOW, &wanted(Side::Bottom, Alignment::Start));
        assert_eq!(side, Side::Bottom);
    }

    #[test]
    fn shifts_back_inside_at_the_right_edge() {
        let (x, _, _) = place(button(950.0, 100.0), (150.0, 100.0), WINDOW, &wanted(Side::Bottom, Alignment::Start));
        assert_eq!(x, 1000.0 - 8.0 - 150.0);
    }

    #[test]
    fn side_placements_flip_horizontally() {
        let (x, _, side) = place(button(900.0, 100.0), (150.0, 100.0), WINDOW, &wanted(Side::Right, Alignment::Start));
        assert_eq!((x, side), (900.0 - 4.0 - 150.0, Side::Left));
    }

    #[test]
    fn a_point_flips_its_alignment_too() {
        let point = Rect { x: 950.0, y: 750.0, width: 0.0, height: 0.0 };
        let mut at = wanted(Side::Bottom, Alignment::Start);
        at.at_point = true;
        at.offset = 0.0;
        let (x, y, side) = place(point, (200.0, 100.0), WINDOW, &at);
        assert_eq!((x, y, side), (750.0, 650.0, Side::Top));
    }

    #[test]
    fn without_flip_and_shift_it_goes_where_asked() {
        let mut free = wanted(Side::Bottom, Alignment::Start);
        free.flip = false;
        free.shift = false;
        let (x, y, side) = place(button(950.0, 780.0), (150.0, 100.0), WINDOW, &free);
        assert_eq!((x, y, side), (950.0, 814.0, Side::Bottom));
    }
}
