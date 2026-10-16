//! Files dragged onto the window from other programs. GPUI turns the platform's file-drop
//! events into mouse moves and a mouse release while a drag of `ExternalPaths` is active; only
//! the end of the drag, when the files leave the window without being dropped, arrives as
//! an event of its own, which this listens for.

use gpui::prelude::*;
use gpui::{AnyElement, Context, DispatchPhase, FileDropEvent, canvas};

use crate::workbench::Workbench;

/// An invisible element that tells the window when files that were being dragged over it left.
pub(super) fn external_drag_ended(cx: &mut Context<Workbench>) -> AnyElement {
    let ended = cx.listener(|workbench, _: &FileDropEvent, _window, cx| {
        workbench.external_drag_left(cx);
    });
    canvas(
        |_, _, _| {},
        move |_, _, window, _| {
            window.on_mouse_event(move |event: &FileDropEvent, phase, window, app| {
                if phase == DispatchPhase::Capture && matches!(event, FileDropEvent::Exited) {
                    ended(event, window, app);
                }
            });
        },
    )
    .absolute()
    .size_0()
    .into_any_element()
}
