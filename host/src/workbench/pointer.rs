//! Hover and pointer capture: a button pressed on an element holds the pointer until
//! it is released, wherever it goes.

use gpui::Context;

use super::Workbench;
use crate::render::{self};
use crate::tree::NodeId;

impl Workbench {
    /// The pointer entered or left an element that tracks it, which restyles it.
    pub(crate) fn hover_changed(&mut self, id: NodeId, hovered: bool, cx: &mut Context<Self>) {
        let mut set = self.interaction.hovered.borrow_mut();
        let changed = if hovered { set.insert(id) } else { set.remove(&id) };
        drop(set);
        if !changed {
            return;
        }
        let tree = self.svelte.tree.clone();
        let listens = tree.borrow().nodes.get(&id).is_some_and(|node| node.listeners.contains("hover"));
        if let Some(root) = self.enclosing_view(&tree.borrow(), id) {
            self.dirty_view(root, cx);
        }
        cx.notify();
        if listens {
            self.emit(id, "hover", format!("{{\"hovered\":{hovered}}}"), cx);
        }
    }

    /// The element that hears the pointer until the button is released, if any.
    pub fn captured(&self) -> Option<NodeId> {
        self.captured
    }

    /// A button went down on `id`: it hears the pointer, wherever it goes, until release.
    pub fn capture_pointer(&mut self, id: NodeId) {
        self.captured = Some(id);
    }

    /// The pointer moved anywhere in the window while an element holds it.
    pub(crate) fn captured_motion(&mut self, event: &gpui::MouseMoveEvent, cx: &mut Context<Self>) {
        let Some(id) = self.captured else {
            return;
        };
        let detail = render::pointer_detail(id, event.position, event.modifiers, -1, 0, self);
        self.emit(id, "move", detail, cx);
    }

    /// A button was released anywhere in the window: the element holding the pointer hears it.
    pub(crate) fn release_pointer(&mut self, event: &gpui::MouseUpEvent, cx: &mut Context<Self>) {
        let Some(id) = self.captured.take() else {
            return;
        };
        let button = match event.button {
            gpui::MouseButton::Left => 0,
            gpui::MouseButton::Middle => 1,
            _ => 2,
        };
        let detail = render::pointer_detail(id, event.position, event.modifiers, button, event.click_count, self);
        self.emit(id, "up", detail, cx);
    }
}
