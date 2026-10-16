//! Transitions: an element whose style changes eases from where it was.

use std::time::{Duration, Instant};

use gpui::Rgba;

use super::{Computed, Painter};
use crate::style::{self};
use crate::tree::NodeId;

/// The properties a transition eases between.
#[derive(Clone, Copy, PartialEq)]
pub struct Eased {
    background: Option<Rgba>,
    color: Option<Rgba>,
    border: Option<Rgba>,
    opacity: f32,
}

/// A transition in flight: from where the element was to where its style now says.
pub struct Animation {
    from: Eased,
    to: Eased,
    /// When the easing begins, delay included; None once it has finished.
    start: Option<Instant>,
}

fn mix(from: Rgba, to: Rgba, amount: f32) -> Rgba {
    Rgba {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

/// A colour easing towards another; a missing one counts as transparent.
fn mix_option(from: Option<Rgba>, to: Option<Rgba>, amount: f32) -> Option<Rgba> {
    match (from, to) {
        (Some(from), Some(to)) => Some(mix(from, to, amount)),
        (None, Some(to)) => Some(mix(Rgba { a: 0.0, ..to }, to, amount)),
        (Some(from), None) => Some(mix(from, Rgba { a: 0.0, ..from }, amount)),
        (None, None) => None,
    }
}

impl Animation {
    /// Where the easing has got to at `now`, and how far along it is.
    fn value_at(&self, now: Instant, transition: &style::Transition) -> (Eased, f32) {
        let Some(start) = self.start else {
            return (self.to, 1.0);
        };
        if now < start {
            return (self.from, 0.0);
        }
        let elapsed = now.duration_since(start).as_secs_f32() * 1000.0;
        let progress = (elapsed / transition.duration_ms.max(1.0)).clamp(0.0, 1.0);
        if progress >= 1.0 {
            return (self.to, 1.0);
        }
        let eased = transition.easing.at(progress);
        let value = Eased {
            background: mix_option(self.from.background, self.to.background, eased),
            color: mix_option(self.from.color, self.to.color, eased),
            border: mix_option(self.from.border, self.to.border, eased),
            opacity: self.from.opacity + (self.to.opacity - self.from.opacity) * eased,
        };
        (value, progress)
    }
}

impl Painter<'_> {
    /// Applies hover to the style itself, and eases from the last drawn colours to the new
    /// ones. Elements that ease take hover from the host rather than GPUI's refinement.
    pub(super) fn ease_style(&self, id: NodeId, computed: &mut Computed) {
        if self.interaction.hovered.borrow().contains(&id) {
            if let Some(hover) = computed.hover.take() {
                computed.style.overlay(&hover);
            }
        }
        computed.hover = None;
        let style = &mut computed.style;
        let Some(transition) = style.transition.clone() else {
            return;
        };
        let target = Eased {
            background: style.background,
            color: style.color,
            border: style.border_color,
            opacity: style.opacity.unwrap_or(1.0),
        };
        let now = Instant::now();
        let mut animations = self.interaction.animations.borrow_mut();
        let animation = animations.entry(id).or_insert(Animation { from: target, to: target, start: None });
        if animation.to != target {
            let (current, _) = animation.value_at(now, &transition);
            animation.from = current;
            animation.to = target;
            animation.start = Some(now + Duration::from_secs_f32(transition.delay_ms / 1000.0));
        }
        let (shown, progress) = animation.value_at(now, &transition);
        if progress < 1.0 {
            self.interaction.animating.set(true);
            self.interaction.animating_ids.borrow_mut().insert(id);
        } else {
            animation.start = None;
        }
        style.background = shown.background;
        style.color = shown.color;
        style.border_color = shown.border;
        if shown.opacity < 1.0 || style.opacity.is_some() {
            style.opacity = Some(shown.opacity);
        }
    }
}
