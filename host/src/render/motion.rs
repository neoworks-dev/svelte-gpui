//! Transitions: an element whose style changes eases from where it was.

use std::time::{Duration, Instant};

use gpui::Rgba;

use super::{Computed, Painter};
use crate::style::{self, Fill, mix};
use crate::tree::NodeId;

/// The properties a transition eases between.
#[derive(Clone, Copy, PartialEq)]
pub struct Eased {
    background: Option<Fill>,
    color: Option<Rgba>,
    border: Option<Rgba>,
    opacity: f32,
}

/// A transition in flight: from where the element was to where its style now says.
pub struct Animation {
    from: Eased,
    to: Eased,
    /// What the style asked for, before a text colour it leaves out was filled in for the easing.
    asked: Eased,
    /// When the easing begins, delay included; None once it has finished.
    start: Option<Instant>,
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
    /// An element that has only ever looked like `target`.
    fn settled(target: Eased) -> Self {
        Animation { from: target, to: target, asked: target, start: None }
    }

    /// Begins easing from where the element is at `now` to `target`.
    fn retarget(&mut self, target: Eased, now: Instant, transition: &style::Transition, inherited: Rgba) {
        let (current, _) = self.value_at(now, transition);
        self.from = current;
        self.to = target;
        self.asked = target;
        self.start = Some(now + Duration::from_secs_f32(transition.delay_ms / 1000.0));
        self.fill_text_colour(inherited);
    }

    /// A text colour a style leaves out is the one the element inherits, not transparent: easing
    /// to or from a colour that only the other end names would otherwise make the text fade in or out.
    fn fill_text_colour(&mut self, inherited: Rgba) {
        match (self.from.color, self.to.color) {
            (Some(_), None) => self.to.color = Some(inherited),
            (None, Some(_)) => self.from.color = Some(inherited),
            _ => {}
        }
    }

    /// Where the easing has got to at `now`, and how far along it is.
    fn value_at(&self, now: Instant, transition: &style::Transition) -> (Eased, f32) {
        let Some(start) = self.start else {
            return (self.asked, 1.0);
        };
        if now < start {
            return (self.from, 0.0);
        }
        let elapsed = now.duration_since(start).as_secs_f32() * 1000.0;
        let progress = (elapsed / transition.duration_ms.max(1.0)).clamp(0.0, 1.0);
        if progress >= 1.0 {
            return (self.asked, 1.0);
        }
        let eased = transition.easing.at(progress);
        let value = Eased {
            background: Fill::ease(self.from.background, self.to.background, eased),
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
    /// `inherited_colour` is the text colour the element's parent hands down.
    pub(super) fn ease_style(&self, id: NodeId, computed: &mut Computed, inherited_colour: Rgba) {
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
        let animation = animations.entry(id).or_insert_with(|| Animation::settled(target));
        if animation.asked != target {
            animation.retarget(target, now, &transition, inherited_colour);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{Easing, Transition};

    const WHITE: Rgba = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    const GREY: Rgba = Rgba { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
    const TRANSPARENT: Rgba = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    fn linear(duration_ms: f32) -> Transition {
        Transition { duration_ms, delay_ms: 0.0, easing: Easing::LINEAR }
    }

    fn eased(color: Option<Rgba>) -> Eased {
        Eased { background: None, color, border: None, opacity: 1.0 }
    }

    #[test]
    fn easing_from_transparent_keeps_the_colour_it_fades_to() {
        let halfway = mix(TRANSPARENT, WHITE, 0.5);
        assert_eq!((halfway.r, halfway.g, halfway.b), (1.0, 1.0, 1.0));
        assert_eq!(halfway.a, 0.5);
        let fading = mix(WHITE, TRANSPARENT, 0.25);
        assert_eq!((fading.r, fading.a), (1.0, 0.75));
    }

    #[test]
    fn a_translucent_colour_mixes_by_how_much_it_covers() {
        let faint = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.2 };
        let halfway = mix(GREY, faint, 0.5);
        assert!((halfway.a - 0.6).abs() < 1e-6);
        assert!(halfway.r > GREY.r && halfway.r < 1.0);
    }

    #[test]
    fn a_text_colour_the_style_leaves_out_eases_from_the_inherited_one() {
        let start = Instant::now();
        let mut animation = Animation::settled(eased(None));
        animation.retarget(eased(Some(WHITE)), start, &linear(100.0), GREY);
        let (halfway, progress) = animation.value_at(start + Duration::from_millis(50), &linear(100.0));
        assert!(progress < 1.0);
        let colour = halfway.color.expect("the text keeps a colour while it eases");
        assert_eq!(colour.a, 1.0);
        assert!(colour.r > GREY.r && colour.r < WHITE.r);
    }

    #[test]
    fn a_finished_transition_goes_back_to_inheriting() {
        let start = Instant::now();
        let mut animation = Animation::settled(eased(Some(WHITE)));
        animation.retarget(eased(None), start, &linear(100.0), GREY);
        let (finished, progress) = animation.value_at(start + Duration::from_millis(100), &linear(100.0));
        assert_eq!(progress, 1.0);
        assert_eq!(finished.color, None);
    }
}
