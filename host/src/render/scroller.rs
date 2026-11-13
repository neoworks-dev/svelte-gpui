//! What a scroller does on top of GPUI's own scroll container, which only ever jumps.
//!
//! **The wheel glides.** A notch (a line delta) is not added to the offset as it arrives: it goes
//! to a [`Glide`], which hands out the distance a little at a time, once per display frame, before the
//! scroller lays out. A touchpad's pixel deltas still move it 1:1. Whether the wheel moved this
//! scroller is decided here too, so the innermost scroller that can move keeps it.
//!
//! `Scroller` wraps the scroller (and its anchor, if it has one) and leaves its layout alone, as
//! `ScrollAnchor` does. It is drawn inside the cached view the scroller sits in, which is told to
//! draw again whenever the glide has another frame to show.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;
use std::time::Instant;

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Element, ElementId, GlobalElementId, Hitbox, HitboxBehavior,
    InspectorElementId, IntoElement, LayoutId, Pixels, Point, ScrollDelta, ScrollHandle, ScrollWheelEvent, Size,
    Window, point, px,
};

use super::glide::Glide;

/// What a scroller remembers between frames.
#[derive(Default)]
pub struct ScrollerState {
    /// The notches the wheel gave it that it has not covered yet.
    pub glide: Glide,
    /// Its offset before the wheel event being delivered: GPUI's own listener has moved it by the time ours hears it.
    wheel_before: Option<Point<Pixels>>,
}

pub type SharedScroller = Rc<RefCell<ScrollerState>>;

/// Wraps a scroller, to glide its wheel.
pub struct Scroller {
    pub inner: AnyElement,
    pub handle: ScrollHandle,
    pub horizontal: bool,
    pub state: SharedScroller,
}

fn along(point: Point<Pixels>, horizontal: bool) -> f32 {
    f32::from(if horizontal { point.x } else { point.y })
}

fn along_size(size: Size<Pixels>, horizontal: bool) -> f32 {
    f32::from(if horizontal { size.width } else { size.height })
}

/// What a wheel delta moves a scroller by: the delta along its axis, or the one across it when
/// there is none, as GPUI's own listener does.
fn applied_along(delta: Point<Pixels>, horizontal: bool) -> f32 {
    let (with, across) = if horizontal { (delta.x, delta.y) } else { (delta.y, delta.x) };
    f32::from(if f32::from(with) != 0.0 { with } else { across })
}

/// Whether a wheel delta of `applied`, from an offset of `before`, moves a scroller that can scroll
/// `furthest`. Offsets count down from 0, and both ends are clamped into range before they are
/// compared: GPUI only clamps an offset when it lays the container out again, so one it has not
/// drawn since can be out of range, and would otherwise seem to move with every notch while the
/// container cannot move at all (#67).
pub fn can_move(before: f32, applied: f32, furthest: f32) -> bool {
    let clamp = |offset: f32| offset.clamp(-furthest.max(0.0), 0.0);
    (clamp(before + applied) - clamp(before)).abs() > 0.01
}

impl Scroller {
    /// Moves the scroller by what the glide hands out this frame, before it lays out and clamps.
    fn glide(&self, window: &mut Window) {
        let mut state = self.state.borrow_mut();
        let step = state.glide.step(Instant::now());
        if step != 0.0 {
            let offset = self.handle.offset();
            let moved = px(step);
            let next =
                if self.horizontal { point(offset.x + moved, offset.y) } else { point(offset.x, offset.y + moved) };
            self.handle.set_offset(next);
        }
        if state.glide.is_active() {
            window.request_animation_frame();
        }
    }

    /// Takes the wheel over from GPUI's own listener, which has already added a delta to the offset
    /// by the time ours runs: this one is registered before the scroller's, so it hears the event after.
    fn listen_for_wheel(&self, hitbox: &Hitbox, window: &mut Window) {
        let line_height = window.line_height();
        let (state, handle, horizontal) = (self.state.clone(), self.handle.clone(), self.horizontal);
        {
            let (state, handle) = (state.clone(), handle.clone());
            window.on_mouse_event(move |_: &ScrollWheelEvent, phase, _, _| {
                if phase == DispatchPhase::Capture {
                    state.borrow_mut().wheel_before = Some(handle.offset());
                }
            });
        }
        let hitbox = hitbox.clone();
        window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble || !hitbox.should_handle_scroll(window) {
                return;
            }
            let before = state.borrow_mut().wheel_before.take().unwrap_or_else(|| handle.offset());
            let delta = event.delta.pixel_delta(line_height);
            let applied = applied_along(delta, horizontal);
            let precise = matches!(event.delta, ScrollDelta::Pixels(_));
            if wheel(&state, &handle, horizontal, before, applied, precise) {
                cx.stop_propagation();
            }
        });
    }
}

/// Decides what a wheel event does to a scroller, and says whether it moved it. A notch (not
/// `precise`) is undone here, GPUI having moved the offset already, and given to the glide
/// instead; a touchpad's pixels stay where GPUI put them.
fn wheel(
    state: &SharedScroller,
    handle: &ScrollHandle,
    horizontal: bool,
    before: Point<Pixels>,
    applied: f32,
    precise: bool,
) -> bool {
    let furthest = along_size(handle.max_offset(), horizontal).max(0.0);
    let before_along = along(before, horizontal);
    let mut state = state.borrow_mut();
    if precise {
        state.glide.cancel();
        return can_move(before_along, applied, furthest);
    }
    handle.set_offset(before);
    // What the glide still has to cover counts as done, so a notch at the end the glide is heading for
    // does not keep the wheel from the scroller around this one.
    let target = (before_along + state.glide.pending()).clamp(-furthest, 0.0);
    let next = (target + applied).clamp(-furthest, 0.0);
    if (next - target).abs() < 0.01 {
        return false;
    }
    state.glide.add(next - target, Instant::now());
    true
}

impl Element for Scroller {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

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
        (self.inner.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        self.glide(window);
        self.inner.prepaint(window, cx);
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _layout: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.listen_for_wheel(hitbox, window);
        self.inner.paint(window, cx);
    }
}

impl IntoElement for Scroller {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::can_move;

    #[test]
    fn a_wheel_with_room_to_move_moves_the_container() {
        // At the top of a container that can scroll 300 px, a notch down moves it 78 px.
        assert!(can_move(0.0, -78.0, 300.0));
        // Near its end, the part of the notch that fits still moves it.
        assert!(can_move(-260.0, -78.0, 300.0));
    }

    #[test]
    fn a_wheel_past_an_end_does_not_move_the_container() {
        assert!(!can_move(0.0, 78.0, 300.0));
        assert!(!can_move(-300.0, -78.0, 300.0));
    }

    #[test]
    fn a_container_that_cannot_scroll_never_keeps_the_wheel_whatever_its_offset() {
        // The stale offset of a code block that had not been laid out again (#67).
        assert!(!can_move(-24.0, 78.0, 0.0));
        assert!(!can_move(0.0, 78.0, 0.0));
        assert!(!can_move(-24.0, -78.0, 0.0));
        assert!(!can_move(54.0, -78.0, 0.0));
    }
}
