//! A scroller with an anchor (`anchor` on `g-scroll`). The host keeps what the app wants in view while
//! it lays out, so the app never has to chase it with scroll requests of its own:
//!
//! - `content`: the child at the top of the window stays where it is on screen when what is above it
//!   changes size, as `overflow-anchor` does in a browser.
//! - `end`: the same, and a scroller at its end stays there as its content grows or shrinks. It opens
//!   at its end, and scrolling back to it pins it again.
//!
//! GPUI computes a scroller's content size and clamps its offset inside the scroller's own
//! `prepaint`, after layout and before its children are placed. `ScrollAnchor` wraps the scroller and
//! moves the offset just before that, once this frame's layout is known, so the frame is right the
//! first time it is drawn. The scroller's direct children are the anchors: each is wrapped in a
//! `LayoutSpy` that keeps its layout id, because GPUI does not give the wrapper the children's.

use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels,
    ScrollHandle, Window, point, px,
};

use crate::tree::NodeId;

/// What a nearer look at one child of the scroller remembers between frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub node: NodeId,
    /// How far below the top of the scroller the child's top was, with the content unscrolled.
    pub top: f32,
}

/// What the scroller remembers from the last frame.
#[derive(Default, Debug)]
pub struct AnchorState {
    /// Whether a frame has been drawn: before that, the scroller starts at its end.
    pub started: bool,
    /// Whether the scroller was at its end when the last frame was drawn.
    pub pinned: bool,
    /// How far it could scroll then.
    pub furthest: f32,
    /// The child at the top of the window, when not pinned.
    pub anchor: Option<Anchor>,
}

/// Where each child of the scroller sits in this frame's layout, by the child's node.
pub type Slots = Vec<(Option<NodeId>, Rc<Cell<Option<LayoutId>>>)>;

/// How close to the end counts as being at it, in pixels.
const AT_END: f32 = 0.5;

/// How far the offset has to move so the anchor stays where it was on screen; nothing when the anchor is gone.
pub fn shift_for(anchor: Option<Anchor>, children: &[(Option<NodeId>, f32)]) -> f32 {
    let Some(anchor) = anchor else {
        return 0.0;
    };
    match children.iter().find(|(node, _)| *node == Some(anchor.node)) {
        Some((_, top)) => top - anchor.top,
        None => 0.0,
    }
}

/// The first child that reaches below `window_top`, which is how far below the scroller's top the window is.
pub fn anchor_at(children: &[(Option<NodeId>, f32, f32)], window_top: f32) -> Option<Anchor> {
    for (node, top, bottom) in children {
        if let Some(node) = node {
            if *bottom > window_top + AT_END {
                return Some(Anchor { node: *node, top: *top });
            }
        }
    }
    None
}

/// Whether an offset, counted from the top, is at the end of a scroller that can scroll `furthest`.
pub fn is_at_end(offset: f32, furthest: f32) -> bool {
    offset >= furthest - AT_END
}

/// What a scroller keeps in view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchoring {
    /// The content at the top of the window.
    Content,
    /// The content at the top of the window, and the end while the scroller is at it.
    End,
}

/// Wraps the scroller, to move its offset after layout and before the scroller clamps it.
pub struct ScrollAnchor {
    pub anchoring: Anchoring,
    pub scroller: AnyElement,
    pub handle: ScrollHandle,
    pub state: Rc<RefCell<AnchorState>>,
    pub slots: Slots,
}

impl ScrollAnchor {
    /// Where each child sits in this frame's layout, as its node, its top and its bottom, counted from the top of the scroller.
    fn children(&self, bounds: Bounds<Pixels>, window: &mut Window) -> Vec<(Option<NodeId>, f32, f32)> {
        let mut found = Vec::with_capacity(self.slots.len());
        for (node, slot) in &self.slots {
            let Some(layout) = slot.get() else {
                continue;
            };
            let child = window.layout_bounds(layout);
            found.push((*node, f32::from(child.top() - bounds.top()), f32::from(child.bottom() - bounds.top())));
        }
        found
    }

    /// Settles the offset before the scroller places its children: at the end when it was there, otherwise
    /// moved by as much as its anchor moved.
    fn settle(&self, bounds: Bounds<Pixels>, window: &mut Window) {
        let children = self.children(bounds, window);
        let mut state = self.state.borrow_mut();
        let offset = self.handle.offset();
        let scrolled = -f32::from(offset.y);
        // Where it stood at the end of the last frame, so a scroller moved back to its end by the user or
        // the app is pinned again, and one the content grew under is still at the end it was at.
        state.pinned = self.anchoring == Anchoring::End && (!state.started || is_at_end(scrolled, state.furthest));
        if state.pinned {
            self.handle.scroll_to_bottom();
            return;
        }
        let tops: Vec<(Option<NodeId>, f32)> = children.iter().map(|(node, top, _)| (*node, *top)).collect();
        let shift = shift_for(state.anchor, &tops);
        if shift.abs() > 0.01 {
            self.handle.set_offset(point(offset.x, offset.y - px(shift)));
        }
    }

    /// Remembers where the scroller ended up, for the next frame.
    fn remember(&self, bounds: Bounds<Pixels>, window: &mut Window) {
        let children = self.children(bounds, window);
        let scrolled = -f32::from(self.handle.offset().y);
        let furthest = f32::from(self.handle.max_offset().height);
        let mut state = self.state.borrow_mut();
        state.started = true;
        state.furthest = furthest;
        state.pinned = self.anchoring == Anchoring::End && is_at_end(scrolled, furthest);
        state.anchor = if state.pinned { None } else { anchor_at(&children, scrolled) };
    }
}

impl Element for ScrollAnchor {
    type RequestLayoutState = LayoutId;
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
    ) -> (LayoutId, LayoutId) {
        for (_, slot) in &self.slots {
            slot.set(None);
        }
        let layout = self.scroller.request_layout(window, cx);
        (layout, layout)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut LayoutId,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.settle(bounds, window);
        self.scroller.prepaint(window, cx);
        self.remember(bounds, window);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _layout: &mut LayoutId,
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.scroller.paint(window, cx);
    }
}

impl IntoElement for ScrollAnchor {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

/// Wraps a child of an anchored scroller, to keep where its layout put it.
pub struct LayoutSpy {
    pub child: AnyElement,
    pub slot: Rc<Cell<Option<LayoutId>>>,
}

impl Element for LayoutSpy {
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
        let layout = self.child.request_layout(window, cx);
        self.slot.set(Some(layout));
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _layout: &mut (),
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

impl IntoElement for LayoutSpy {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_child_that_moved_down_moves_the_offset_down_with_it() {
        let anchor = Some(Anchor { node: 7, top: 300.0 });
        let children = [(Some(6), 100.0), (Some(7), 340.0)];
        assert_eq!(shift_for(anchor, &children), 40.0);
    }

    #[test]
    fn an_anchor_that_is_gone_moves_nothing() {
        let anchor = Some(Anchor { node: 7, top: 300.0 });
        assert_eq!(shift_for(anchor, &[(Some(8), 10.0)]), 0.0);
        assert_eq!(shift_for(None, &[(Some(8), 10.0)]), 0.0);
    }

    #[test]
    fn the_anchor_is_the_first_child_reaching_into_the_window() {
        let children = [(Some(1), 0.0, 100.0), (Some(2), 100.0, 250.0), (Some(3), 250.0, 400.0)];
        assert_eq!(anchor_at(&children, 120.0), Some(Anchor { node: 2, top: 100.0 }));
        assert_eq!(anchor_at(&children, 250.0), Some(Anchor { node: 3, top: 250.0 }));
        assert_eq!(anchor_at(&children, 900.0), None);
    }

    #[test]
    fn the_end_is_where_the_offset_reaches_the_furthest() {
        assert!(is_at_end(500.0, 500.0));
        assert!(is_at_end(499.7, 500.0));
        assert!(!is_at_end(450.0, 500.0));
        assert!(is_at_end(0.0, 0.0));
    }
}
