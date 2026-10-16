//! A press that no element took focus for, which ends focus as a click outside a field does on the web.

use gpui::{AnyElement, Context, DispatchPhase, MouseDownEvent, canvas, prelude::*};

use crate::workbench::Workbench;

/// An invisible element that hears every mouse press after the elements under it have, and takes
/// focus away when none of them claimed it. An element claims a press by taking focus, which GPUI
/// reports as `prevent_default`, or by having `keepfocus`. It listens to the whole window, so a
/// press on an overlay that hides the elements below it counts too.
pub(super) fn press_ends_focus(cx: &mut Context<Workbench>) -> AnyElement {
    let unclaimed = cx.listener(|workbench, _: &MouseDownEvent, window, _cx| {
        workbench.blur_focused(window);
    });
    canvas(
        |_, _, _| {},
        move |_, _, window, _| {
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, app| {
                if phase == DispatchPhase::Bubble && !window.default_prevented() {
                    unclaimed(event, window, app);
                }
            });
        },
    )
    .absolute()
    .size_0()
    .into_any_element()
}
