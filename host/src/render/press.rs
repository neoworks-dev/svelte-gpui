//! Window-wide pointer listeners: a press that no element took focus for, which ends focus as a
//! click outside a field does on the web, and the moves and release of a captured pointer.

use gpui::{AnyElement, Context, DispatchPhase, MouseDownEvent, MouseMoveEvent, MouseUpEvent, canvas, prelude::*};

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

/// An invisible element that hands the element holding the pointer its moves and its release,
/// wherever the pointer is. GPUI's element listeners only hear the pointer over the window, but
/// while a button is held the window hears it outside too, and a drag must not lose its release
/// there.
pub(super) fn captured_pointer(cx: &mut Context<Workbench>) -> AnyElement {
    let moved = cx.listener(|workbench, event: &MouseMoveEvent, _window, cx| {
        workbench.captured_motion(event, cx);
    });
    let released = cx.listener(|workbench, event: &MouseUpEvent, _window, cx| {
        workbench.release_pointer(event, cx);
    });
    canvas(
        |_, _, _| {},
        move |_, _, window, _| {
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, app| {
                if phase == DispatchPhase::Bubble {
                    moved(event, window, app);
                }
            });
            window.on_mouse_event(move |event: &MouseUpEvent, phase, window, app| {
                if phase == DispatchPhase::Capture {
                    released(event, window, app);
                }
            });
        },
    )
    .absolute()
    .size_0()
    .into_any_element()
}
