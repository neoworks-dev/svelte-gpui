//! Scroll requests the app makes while it handles an event.

use gpui::{Context, point, px};

use super::Workbench;
use crate::tree::{Command, NodeId};

impl Workbench {
    /// Moves the scrolling elements the app just asked to move, before anything is drawn. A
    /// list that learns a row above the window is taller than it guessed scrolls by the
    /// difference in the same step as it grows, so the rows in view do not jump for a frame.
    /// A request for an element that has not been drawn yet waits for the next tick.
    pub(super) fn run_scroll_commands(&mut self, cx: &mut Context<Self>) {
        let mut moved = Vec::new();
        {
            let handles = self.interaction.scroll.borrow();
            self.svelte.tree.borrow_mut().commands.retain(|command| match command {
                Command::ScrollTo(id, left, top) => match handles.get(id) {
                    Some(handle) => {
                        handle.set_offset(point(px(-left), px(-top)));
                        moved.push(*id);
                        false
                    }
                    None => true,
                },
                _ => true,
            });
        }
        for id in moved {
            self.stop_gliding(id);
            self.touch(id, cx);
        }
    }

    /// A scroller that was asked to be somewhere stays there: what its wheel had left to cover is forgotten.
    pub(super) fn stop_gliding(&self, id: NodeId) {
        if let Some(state) = self.interaction.scrollers.borrow().get(&id) {
            state.borrow_mut().glide.cancel();
        }
    }
}
