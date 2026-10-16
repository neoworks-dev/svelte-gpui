//! What the window learned that the app hears about: focus, scrolling, layout, images
//! and the window's size.

use gpui::{Context, Window};

use super::Workbench;
use crate::images::ImageState;
use crate::tree::{Kind, NodeId};

impl Workbench {
    /// Sends `blur` and `focus` when focus moved since the last tick.
    pub(super) fn report_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = self.focused_node(window);
        if now == self.focused {
            return;
        }
        let before = self.focused;
        self.focused = now;
        if let Some(id) = before {
            self.commit_change(id, cx);
            // The caret it drew has to go.
            self.touch(id, cx);
            self.emit(id, "blur", "{}".to_string(), cx);
        }
        if let Some(id) = now {
            self.field_focused(id);
            self.caret_moved(id, cx);
            self.emit(id, "focus", "{}".to_string(), cx);
        }
    }

    /// Sends `scroll` for the scrollable elements that listen and have moved, or whose furthest
    /// offset changed because their content grew or shrank.
    pub(super) fn report_scroll(&mut self, cx: &mut Context<Self>) {
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
                let previous = self.last_scroll.get(id).copied().unwrap_or((0.0, 0.0, 0.0, 0.0));
                let content = handle.max_offset();
                let furthest = (f32::from(content.width), f32::from(content.height));
                if !scroll_changed(previous, (current.0, current.1, furthest.0, furthest.1)) {
                    continue;
                }
                moved.push((*id, current.0, current.1, furthest.0, furthest.1));
            }
        }
        for (id, left, top, extra_width, extra_height) in moved {
            self.last_scroll.insert(id, (left, top, extra_width, extra_height));
            let detail = format!("{{\"x\":{left},\"y\":{top},\"maxX\":{extra_width},\"maxY\":{extra_height}}}");
            self.emit(id, "scroll", detail, cx);
        }
    }

    /// Tells the app where every element that listens for `layout` came out, when it changed, and
    /// the size of those that listen for `resize`, when that changed.
    pub(super) fn report_sizes(&mut self, cx: &mut Context<Self>) {
        let measured: Vec<(NodeId, (f32, f32, f32, f32))> = {
            let tree = self.svelte.tree.borrow();
            self.interaction
                .bounds
                .borrow()
                .iter()
                .filter(|(id, _)| tree.nodes.get(id).is_some_and(|node| node.observed()))
                .map(|(id, bounds)| {
                    let found = (
                        f32::from(bounds.origin.x),
                        f32::from(bounds.origin.y),
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                    );
                    (*id, found)
                })
                .collect()
        };
        for (id, found) in measured {
            let (moved, resized) = match self.sent_sizes.get(&id) {
                Some(old) => (
                    (old.0 - found.0).abs() >= 0.5 || (old.1 - found.1).abs() >= 0.5,
                    (old.2 - found.2).abs() >= 0.05 || (old.3 - found.3).abs() >= 0.05,
                ),
                None => (true, true),
            };
            if !moved && !resized {
                continue;
            }
            self.sent_sizes.insert(id, found);
            let (x, y, width, height) = found;
            let detail = format!("{{\"x\":{x},\"y\":{y},\"width\":{width},\"height\":{height}}}");
            let (wants_layout, wants_resize) = {
                let tree = self.svelte.tree.borrow();
                let node = tree.nodes.get(&id);
                (
                    node.is_some_and(|node| node.listeners.contains("layout")),
                    node.is_some_and(|node| node.listeners.contains("resize")),
                )
            };
            if wants_layout {
                self.emit(id, "layout", detail.clone(), cx);
            }
            if resized && wants_resize {
                self.emit(id, "resize", detail, cx);
            }
        }
    }

    /// Fires `load` or `error` on each image once its source has settled.
    pub(super) fn report_images(&mut self, cx: &mut Context<Self>) {
        let mut outcomes: Vec<(NodeId, String, bool)> = Vec::new();
        {
            let tree = self.svelte.tree.borrow();
            for (id, node) in tree.nodes.iter() {
                let Kind::Element { tag } = &node.kind else {
                    continue;
                };
                if tag != "g-image" {
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
            self.emit(id, kind, "{}".to_string(), cx);
        }
    }
}

/// Whether a scroller's offset or furthest offset, as `(x, y, max x, max y)`, differs from what the app was told.
fn scroll_changed(told: (f32, f32, f32, f32), now: (f32, f32, f32, f32)) -> bool {
    let differs = |a: f32, b: f32| (a - b).abs() >= 0.5;
    differs(now.0, told.0) || differs(now.1, told.1) || differs(now.2, told.2) || differs(now.3, told.3)
}

#[cfg(test)]
mod tests {
    use super::scroll_changed;

    #[test]
    fn a_move_is_reported() {
        assert!(scroll_changed((0.0, 0.0, 0.0, 500.0), (0.0, 40.0, 0.0, 500.0)));
    }

    #[test]
    fn content_growing_under_a_still_scroller_is_reported() {
        assert!(scroll_changed((0.0, 0.0, 0.0, 500.0), (0.0, 0.0, 0.0, 900.0)));
    }

    #[test]
    fn a_scroller_that_has_not_changed_stays_quiet() {
        assert!(!scroll_changed((0.0, 40.0, 0.0, 500.0), (0.0, 40.2, 0.0, 500.3)));
        assert!(!scroll_changed((0.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 0.0)));
    }
}
