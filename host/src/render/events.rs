//! Input: which events an element listens for decides which GPUI handlers it gets, and
//! each handler reports to the app by node id.

use gpui::prelude::*;
use gpui::{
    AnyElement, ClickEvent, Context, Div, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, ScrollDelta,
    ScrollHandle, ScrollWheelEvent, Window,
};

use super::{Computed, Painter, Primitive};
use crate::style::{self};
use crate::tree::{Node, NodeId};
use crate::workbench::Workbench;

impl Painter<'_> {
    /// Gives an element an id, so it can take clicks, hover, focus and scrolling, and wires
    /// the handlers for the events the app listens for.
    pub(super) fn interactive(
        &self,
        element: Div,
        id: NodeId,
        node: &Node,
        primitive: Primitive,
        computed: &Computed,
        eased: bool,
        cx: &mut Context<Workbench>,
    ) -> AnyElement {
        let mut element = element.id(id as usize);
        let style = &computed.style;
        let disabled = node.attributes.contains_key("disabled");
        let listens = |name: &str| node.listeners.contains(name);

        if eased || listens("hover") {
            element = element.on_hover(cx.listener(move |workbench, hovered: &bool, _window, cx| {
                workbench.hover_changed(id, *hovered, cx);
            }));
        }
        let focusable = (primitive.focusable() && !disabled) || node.attributes.contains_key("focusable");
        if focusable {
            let handle = self.interaction.focus_handle(id, cx);
            element = element.track_focus(&handle);
            if let Some(focus) = computed.focus.clone() {
                element = element.focus(move |refinement| style::apply(refinement, &focus));
            }
        }
        if primitive == Primitive::Scroll {
            let handle = self.interaction.scroll_handle(id);
            let horizontal = node.attributes.contains_key("horizontal");
            element = element.track_scroll(&handle);
            element = if horizontal { element.overflow_x_scroll() } else { element.overflow_y_scroll() };
            // GPUI scrolls every container under the pointer. The innermost one that moved keeps
            // the wheel, so an outer one only scrolls once the inner one is at its end.
            element = element.on_scroll_wheel(move |event: &ScrollWheelEvent, window, cx| {
                if scrolled_by(&handle, event, horizontal, window) {
                    cx.stop_propagation();
                }
            });
        }
        if node.attributes.contains_key("keepfocus") {
            // Before the elements under it can take focus: a press here leaves focus where it is.
            element = element.capture_any_mouse_down(|_, window, _| window.prevent_default());
        }
        if disabled {
            return element.into_any_element();
        }
        if let Some(hover) = computed.hover.clone() {
            element = element.hover(move |refinement| style::apply(refinement, &hover));
        }
        if let Some(active) = computed.active.clone() {
            element = element.active(move |refinement| style::apply(refinement, &active));
        }
        if style.cursor.is_none() && listens("press") {
            element = element.cursor_pointer();
        }
        if listens("press") {
            element = element.on_click(cx.listener(move |workbench, event: &ClickEvent, _window, cx| {
                let detail = pointer_detail(id, event.position(), event.modifiers(), 0, event.click_count(), workbench);
                workbench.emit(id, "press", detail, cx);
                cx.stop_propagation();
            }));
        }
        if primitive == Primitive::Input {
            element = self.wire_field(element, id, cx);
        }
        if primitive == Primitive::Element && self.interaction.focus.borrow().contains_key(&id) {
            // After the view's own handlers, a press that nothing took gives the view focus.
            element = element.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |workbench, _: &MouseDownEvent, window, _cx| {
                    workbench.focus_node(id, window);
                }),
            );
        }
        if listens("down") {
            for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
                // A button it does not take is left for the elements around it, which is also how
                // GPUI passes a press up: only a handler that stops it keeps it.
                if !takes_button(node, button_number(button)) {
                    continue;
                }
                element = element.on_mouse_down(
                    button,
                    cx.listener(move |workbench, event: &MouseDownEvent, window, cx| {
                        let detail = pointer_detail(
                            id,
                            event.position,
                            event.modifiers,
                            button_number(event.button),
                            event.click_count,
                            workbench,
                        );
                        workbench.capture_pointer(id);
                        workbench.emit(id, "down", detail, cx);
                        // Nothing above this element hears the press, so what they would do with focus is done here.
                        if !window.default_prevented() {
                            workbench.press_focus(id, window);
                        }
                        cx.stop_propagation();
                    }),
                );
            }
        }
        if listens("move") {
            element = element.on_mouse_move(cx.listener(move |workbench, event: &MouseMoveEvent, _window, cx| {
                // While the pointer is held after a press here, the window reports the motion.
                if workbench.captured() == Some(id) {
                    return;
                }
                let detail = pointer_detail(id, event.position, event.modifiers, -1, 0, workbench);
                workbench.emit(id, "move", detail, cx);
            }));
        }
        if listens("wheel") {
            element = element.on_scroll_wheel(cx.listener(move |workbench, event: &ScrollWheelEvent, _window, cx| {
                let (dx, dy) = match event.delta {
                    ScrollDelta::Pixels(delta) => (f32::from(delta.x), f32::from(delta.y)),
                    ScrollDelta::Lines(delta) => (delta.x * 16.0, delta.y * 16.0),
                };
                let detail = format!("{{\"dx\":{},\"dy\":{}{}}}", -dx, -dy, modifier_fields(event.modifiers));
                workbench.emit(id, "wheel", detail, cx);
            }));
        }
        element.into_any_element()
    }
}

/// Whether the wheel event moved this container. GPUI's own scroll listener has already
/// added the event's delta to the offset, unclamped, so the offset before it is the
/// difference.
fn scrolled_by(handle: &ScrollHandle, event: &ScrollWheelEvent, horizontal: bool, window: &Window) -> bool {
    let delta = event.delta.pixel_delta(window.line_height());
    let (along, across) = if horizontal { (delta.x, delta.y) } else { (delta.y, delta.x) };
    // A vertical wheel scrolls a horizontal container too, as GPUI does.
    let applied = f32::from(if f32::from(along) != 0.0 { along } else { across });
    let offset = handle.offset();
    let max_offset = handle.max_offset();
    let (now, furthest) = if horizontal {
        (f32::from(offset.x), f32::from(max_offset.width))
    } else {
        (f32::from(offset.y), f32::from(max_offset.height))
    };
    moved(now, applied, furthest)
}

/// Whether an offset of `now` after a wheel delta of `applied` moved a container that can scroll
/// `furthest`. Offsets count down from 0, and both are clamped into range before they are
/// compared: GPUI only clamps an offset when it lays the container out again, so one it has not
/// drawn since can be out of range, and would otherwise seem to move with every notch while the
/// container cannot move at all (#67).
fn moved(now: f32, applied: f32, furthest: f32) -> bool {
    let clamp = |offset: f32| offset.clamp(-furthest.max(0.0), 0.0);
    (clamp(now) - clamp(now - applied)).abs() > 0.01
}

pub(super) fn listens_for_pointer(node: &Node) -> bool {
    node.listeners.contains("down") || node.listeners.contains("move")
}

/// The events for files dragged in from other programs. Which element gets them is decided
/// by where it was laid out, so an element that listens for any of them is measured.
pub const DRAG_EVENTS: [&str; 4] = ["dragenter", "dragleave", "dragover", "drop"];

pub fn listens_for_drag(node: &Node) -> bool {
    DRAG_EVENTS.iter().any(|name| node.listeners.contains(*name))
}

/// Whether the `down` listener on a node takes presses of this button (0 left, 1 middle, 2 right).
/// Its `buttons` attribute names the ones it takes, such as `left,middle`; without one it takes all.
pub fn takes_button(node: &Node, button: i32) -> bool {
    let Some(buttons) = node.attributes.get("buttons") else {
        return true;
    };
    let name = match button {
        0 => "left",
        1 => "middle",
        2 => "right",
        _ => return false,
    };
    buttons.split(',').any(|taken| taken.trim() == name)
}

pub(super) fn button_number(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        _ => 3,
    }
}

pub(super) fn modifier_fields(modifiers: gpui::Modifiers) -> String {
    format!(
        ",\"ctrl\":{},\"shift\":{},\"alt\":{},\"meta\":{}",
        modifiers.control, modifiers.shift, modifiers.alt, modifiers.platform
    )
}

/// Where a pointer event happened, in the window and inside the element, with its button.
pub fn pointer_detail(
    id: NodeId,
    position: gpui::Point<gpui::Pixels>,
    modifiers: gpui::Modifiers,
    button: i32,
    clicks: usize,
    workbench: &Workbench,
) -> String {
    let (x, y) = (f32::from(position.x), f32::from(position.y));
    let origin = workbench
        .interaction
        .bounds
        .borrow()
        .get(&id)
        .map(|bounds| (f32::from(bounds.origin.x), f32::from(bounds.origin.y)))
        .unwrap_or((0.0, 0.0));
    format!(
        "{{\"x\":{},\"y\":{},\"windowX\":{x},\"windowY\":{y},\"button\":{button},\"clicks\":{clicks}{}}}",
        x - origin.0,
        y - origin.1,
        modifier_fields(modifiers)
    )
}

#[cfg(test)]
mod tests {
    use super::moved;

    #[test]
    fn a_wheel_with_room_to_move_moves_the_container() {
        // At the top of a container that can scroll 300 px, a notch down moves it 78 px.
        assert!(moved(-78.0, -78.0, 300.0));
        // Near its end, the part of the notch that fits still moves it.
        assert!(moved(-338.0, -78.0, 300.0));
    }

    #[test]
    fn a_wheel_past_an_end_does_not_move_the_container() {
        assert!(!moved(78.0, 78.0, 300.0));
        assert!(!moved(-378.0, -78.0, 300.0));
    }

    #[test]
    fn a_container_that_cannot_scroll_never_keeps_the_wheel_whatever_its_offset() {
        // The stale offset of a code block that had not been laid out again (#67).
        assert!(!moved(54.0, 78.0, 0.0));
        assert!(!moved(0.0, 78.0, 0.0));
        assert!(!moved(-24.0, -78.0, 0.0));
    }
}
