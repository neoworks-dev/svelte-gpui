//! What a scroller does on top of GPUI's own scroll container, which only ever jumps and draws no bar.
//!
//! - **The wheel glides.** A notch (a line delta) is not added to the offset as it arrives: it goes
//!   to a [`Glide`], which hands out the distance a little at a time, once per display frame, before the
//!   scroller lays out. A touchpad's pixel deltas still move it 1:1. Whether the wheel moved this
//!   scroller is decided here too, so the innermost scroller that can move keeps it.
//! - **An overlay scrollbar** lies over the edge of the content: a thin thumb that shows while the
//!   scroller scrolls or the pointer is near, fades out after about a second, can be dragged, and
//!   jumps to where the track is pressed.
//!
//! `Scroller` wraps the scroller (and its anchor, if it has one) and leaves its layout alone, as
//! `ScrollAnchor` does. Both are drawn inside the cached view the scroller sits in, which is told
//! to draw again whenever the glide or the fade has another frame to show.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Element, ElementId, EntityId, GlobalElementId, Hitbox, HitboxBehavior,
    Hsla, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseExitEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, Rgba, ScrollDelta, ScrollHandle, ScrollWheelEvent, Size, WeakEntity, Window, fill,
    point, px, size,
};

use super::glide::Glide;
use super::scrollbar::{BarLook, FADE, HOLD, INSET, Metrics, ZONE, opacity};
use crate::tree::NodeId;
use crate::workbench::Workbench;

/// What a scroller remembers between frames.
#[derive(Default)]
pub struct ScrollerState {
    /// The notches the wheel gave it that it has not covered yet.
    pub glide: Glide,
    /// Its offset before the wheel event being delivered: GPUI's own listener has moved it by the time ours hears it.
    wheel_before: Option<Point<Pixels>>,
    bar: BarState,
}

pub type SharedScroller = Rc<RefCell<ScrollerState>>;

#[derive(Default)]
struct BarState {
    /// How far it was scrolled when last drawn, to tell that it moved.
    scrolled: Option<f32>,
    /// The last scroll or pointer move near the bar, from which it holds and fades.
    activity: Option<Instant>,
    /// Whether the pointer is in the strip along the edge.
    near: bool,
    /// How far into the thumb the pointer took hold of it, while it is dragged.
    grab: Option<f32>,
    /// Whether a timer is waiting to draw the bar again when its hold ends.
    timer: bool,
}

impl BarState {
    fn opacity(&self, now: Instant) -> f32 {
        let held = self.near || self.grab.is_some();
        match self.activity {
            Some(activity) => opacity(now.saturating_duration_since(activity), held),
            None if held => 1.0,
            None => 0.0,
        }
    }
}

/// The whole length of content a scroller shows only part of, and where the part starts in it, for a
/// `List` taller than the host lays out at once.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    pub total: f32,
    pub origin: f32,
}

/// Wraps a scroller, to glide its wheel and draw its bar.
pub struct Scroller {
    pub inner: AnyElement,
    pub node: NodeId,
    pub handle: ScrollHandle,
    pub horizontal: bool,
    pub state: SharedScroller,
    pub look: BarLook,
    pub extent: Option<Extent>,
    /// The text colour in the scroller, which the thumb is tinted with unless the app picked one.
    pub text: Rgba,
    pub workbench: Option<WeakEntity<Workbench>>,
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

/// Where the scroller stands, in the units the bar draws in.
fn metrics_of(handle: &ScrollHandle, viewport: f32, horizontal: bool, extent: Option<Extent>) -> Metrics {
    let furthest = along_size(handle.max_offset(), horizontal).max(0.0);
    let scrolled = (-along(handle.offset(), horizontal)).clamp(0.0, furthest);
    match extent {
        Some(Extent { total, origin }) => {
            Metrics { viewport, content: total.max(viewport), scrolled: origin + scrolled }
        }
        None => Metrics { viewport, content: viewport + furthest, scrolled },
    }
}

fn move_to(handle: &ScrollHandle, horizontal: bool, scrolled: f32) {
    let offset = handle.offset();
    let moved = px(-scrolled);
    handle.set_offset(if horizontal { point(moved, offset.y) } else { point(offset.x, moved) });
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

    fn bar(&self, bounds: Bounds<Pixels>, hitbox: &Hitbox, window: &mut Window, cx: &mut App) {
        if self.look.hidden {
            return;
        }
        let view = window.current_view();
        let bar = Bar {
            state: self.state.clone(),
            handle: self.handle.clone(),
            horizontal: self.horizontal,
            extent: self.extent,
            bounds,
            node: self.node,
            view,
            workbench: self.workbench.clone(),
        };
        let metrics = bar.metrics();
        let now = Instant::now();
        bar.note_scroll(metrics.scrolled, now);
        bar.listen(hitbox, window);
        let Some(thumb) = metrics.thumb() else {
            return;
        };
        let (shown, held, idle) = {
            let state = self.state.borrow();
            let idle = state.bar.activity.map(|activity| now.saturating_duration_since(activity));
            (state.bar.opacity(now), state.bar.near || state.bar.grab.is_some(), idle)
        };
        if shown > 0.0 {
            self.paint_thumb(bounds, thumb.start, thumb.length, held, shown, window);
        }
        match idle {
            _ if held => {}
            Some(idle) if idle < HOLD => bar.wake_after(HOLD - idle, cx),
            Some(idle) if idle < HOLD + FADE => window.request_animation_frame(),
            _ => {}
        }
    }

    fn paint_thumb(
        &self,
        bounds: Bounds<Pixels>,
        start: f32,
        length: f32,
        near: bool,
        shown: f32,
        window: &mut Window,
    ) {
        let thickness = self.look.thickness(near);
        let mut color = self.look.thumb_color(self.text, near);
        color.a *= shown;
        let thumb = if self.horizontal {
            Bounds::new(
                point(bounds.left() + px(start), bounds.bottom() - px(INSET + thickness)),
                size(px(length), px(thickness)),
            )
        } else {
            Bounds::new(
                point(bounds.right() - px(INSET + thickness), bounds.top() + px(start)),
                size(px(thickness), px(length)),
            )
        };
        window.paint_quad(fill(thumb, Hsla::from(color)).corner_radii(px(thickness / 2.0)));
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

/// What the bar's listeners need, cloned into each of them.
#[derive(Clone)]
struct Bar {
    state: SharedScroller,
    handle: ScrollHandle,
    horizontal: bool,
    extent: Option<Extent>,
    bounds: Bounds<Pixels>,
    node: NodeId,
    view: EntityId,
    workbench: Option<WeakEntity<Workbench>>,
}

impl Bar {
    fn metrics(&self) -> Metrics {
        metrics_of(&self.handle, along_size(self.bounds.size, self.horizontal), self.horizontal, self.extent)
    }

    /// Remembers that it moved, which is what makes the bar show.
    fn note_scroll(&self, scrolled: f32, now: Instant) {
        let mut state = self.state.borrow_mut();
        if state.bar.scrolled.is_some_and(|before| (before - scrolled).abs() > 0.01) {
            state.bar.activity = Some(now);
        }
        state.bar.scrolled = Some(scrolled);
    }

    /// Where a window position is along the bar, from the scroller's start.
    fn position(&self, window_position: Point<Pixels>) -> f32 {
        along(window_position - self.bounds.origin, self.horizontal)
    }

    /// Whether a window position is in the strip along the edge that belongs to the bar.
    fn in_zone(&self, window_position: Point<Pixels>) -> bool {
        let local = window_position - self.bounds.origin;
        let (across, with, thickness, length) = if self.horizontal {
            (local.y, local.x, self.bounds.size.height, self.bounds.size.width)
        } else {
            (local.x, local.y, self.bounds.size.width, self.bounds.size.height)
        };
        let (across, with, thickness, length) =
            (f32::from(across), f32::from(with), f32::from(thickness), f32::from(length));
        across >= thickness - ZONE && across <= thickness && with >= 0.0 && with <= length
    }

    fn listen(&self, hitbox: &Hitbox, window: &mut Window) {
        let down = (self.clone(), hitbox.clone());
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if phase == DispatchPhase::Bubble && down.0.press(event, &down.1, window, cx) {
                // The content under the bar does not hear a press the bar took.
                cx.stop_propagation();
            }
        });
        let moved = (self.clone(), hitbox.clone());
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture {
                moved.0.motion(event, &moved.1, window, cx);
            }
        });
        let up = self.clone();
        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                up.release(cx);
            }
        });
        let left = self.clone();
        window.on_mouse_event(move |_: &MouseExitEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                left.leave(cx);
            }
        });
    }

    /// A press in the strip takes hold of the thumb, or puts the thumb under the pointer and takes hold of that.
    /// A bar that has faded is not there to be pressed.
    fn press(&self, event: &MouseDownEvent, hitbox: &Hitbox, window: &mut Window, cx: &mut App) -> bool {
        if event.button != MouseButton::Left || !self.in_zone(event.position) || !hitbox.is_hovered(window) {
            return false;
        }
        let metrics = self.metrics();
        let Some(thumb) = metrics.thumb() else {
            return false;
        };
        let now = Instant::now();
        if self.state.borrow().bar.opacity(now) <= 0.0 {
            return false;
        }
        let pointer = self.position(event.position);
        let on_thumb = pointer >= thumb.start && pointer <= thumb.start + thumb.length;
        let grab = if on_thumb { pointer - thumb.start } else { thumb.length / 2.0 };
        {
            let mut state = self.state.borrow_mut();
            state.bar.grab = Some(grab);
            state.bar.near = true;
            state.bar.activity = Some(now);
            state.glide.cancel();
        }
        self.drag_to(pointer, grab, metrics, thumb.length, cx);
        window.prevent_default();
        cx.notify(self.view);
        true
    }

    fn motion(&self, event: &MouseMoveEvent, hitbox: &Hitbox, window: &mut Window, cx: &mut App) {
        let grab = self.state.borrow().bar.grab;
        if let Some(grab) = grab {
            if event.pressed_button != Some(MouseButton::Left) {
                self.release(cx);
                return;
            }
            let metrics = self.metrics();
            if let Some(thumb) = metrics.thumb() {
                self.drag_to(self.position(event.position), grab, metrics, thumb.length, cx);
            }
            self.state.borrow_mut().bar.activity = Some(Instant::now());
            cx.notify(self.view);
            return;
        }
        let near = self.in_zone(event.position) && hitbox.is_hovered(window) && self.metrics().thumb().is_some();
        let changed = {
            let mut state = self.state.borrow_mut();
            let changed = state.bar.near != near;
            state.bar.near = near;
            if near {
                state.bar.activity = Some(Instant::now());
            }
            changed
        };
        if changed {
            cx.notify(self.view);
        }
    }

    fn release(&self, cx: &mut App) {
        let mut state = self.state.borrow_mut();
        if state.bar.grab.take().is_some() {
            state.bar.activity = Some(Instant::now());
            cx.notify(self.view);
        }
    }

    fn leave(&self, cx: &mut App) {
        let mut state = self.state.borrow_mut();
        if std::mem::replace(&mut state.bar.near, false) {
            cx.notify(self.view);
        }
    }

    /// Puts the thumb, held `grab` into its length, under the pointer at `pointer`.
    fn drag_to(&self, pointer: f32, grab: f32, metrics: Metrics, length: f32, cx: &mut App) {
        let scrolled = metrics.scrolled_for(length, pointer - grab);
        self.scroll_to(scrolled, cx);
    }

    /// Scrolls to an offset counted over the whole content. A scroller that shows only a stretch of a longer
    /// list cannot do it by itself, so the app is asked, as `scrollTo` would be.
    fn scroll_to(&self, scrolled: f32, cx: &mut App) {
        if let (Some(_), Some(workbench)) = (self.extent, &self.workbench) {
            workbench.update(cx, |workbench, cx| workbench.scrollbar_moved(self.node, scrolled, cx)).ok();
            return;
        }
        self.state.borrow_mut().glide.cancel();
        move_to(&self.handle, self.horizontal, scrolled);
    }

    /// Has the view drawn again once the bar's hold is over, so it can start to fade.
    fn wake_after(&self, wait: Duration, cx: &mut App) {
        if std::mem::replace(&mut self.state.borrow_mut().bar.timer, true) {
            return;
        }
        let (state, view) = (self.state.clone(), self.view);
        cx.spawn(async move |cx| {
            cx.background_executor().timer(wait).await;
            state.borrow_mut().bar.timer = false;
            cx.update(|cx| cx.notify(view)).ok();
        })
        .detach();
    }
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
        bounds: Bounds<Pixels>,
        _layout: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.listen_for_wheel(hitbox, window);
        self.inner.paint(window, cx);
        self.bar(bounds, hitbox, window, cx);
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
