//! The app's own GPUI views (`Elements`), drawn inside the box of their node.

use gpui::prelude::*;
use gpui::{Div, Rgba, StyleRefinement, div};

use super::{Painter, attribute};
use crate::tree::{Node, NodeId};

const MISSING: Rgba = Rgba { r: 0.94, g: 0.33, b: 0.31, a: 1.0 };

impl Painter<'_> {
    /// The view behind a `g-element`, filling the node's box. It is cached, so it is drawn again only when
    /// it says it changed. A node whose element is not registered says so where the view would be.
    pub(super) fn element_content(&self, element: Div, id: NodeId, node: &Node) -> Div {
        let view = self.interaction.instances.borrow().live.get(&id).map(|live| live.handle.view());
        let Some(view) = view else {
            let message = format!("no element named {:?} is registered", attribute(node, "name"));
            return element.child(div().text_color(MISSING).child(message));
        };
        // The view's own content scrolls or clips itself; nothing it draws may leave the box.
        element.overflow_hidden().child(view.cached(StyleRefinement::default().size_full()))
    }
}
