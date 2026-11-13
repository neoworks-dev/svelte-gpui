//! What an overlay scrollbar looks like and where it is: a thin thumb over the content's edge, in
//! numbers only. `scroller.rs` paints it and listens for the pointer.

use std::time::Duration;

use gpui::Rgba;
use serde_json::Value;

use crate::style::parse_colour;

/// The strip along the edge where the pointer wakes the bar, and where a press is the bar's.
pub const ZONE: f32 = 16.0;
/// The gap between the thumb and the scroller's edge, and between it and the track's ends.
pub const INSET: f32 = 2.0;
/// The shortest a thumb gets, however long the content.
pub const MIN_THUMB: f32 = 28.0;
/// How long the bar stays after the last scroll or pointer move, and how long it takes to fade.
pub const HOLD: Duration = Duration::from_millis(1000);
pub const FADE: Duration = Duration::from_millis(200);
/// The thumb's thickness while idle, and how much thicker it gets while the pointer is near or holds it.
pub const WIDTH: f32 = 6.0;
pub const WIDTH_NEAR: f32 = 4.0;

/// What the app asked of the bar: `scrollbar={false}` hides it, an object tints it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BarLook {
    pub hidden: bool,
    /// The thumb's colour while idle. Without one it is the text colour, translucent.
    pub color: Option<Rgba>,
    /// The thumb's colour while the pointer is near or holds it.
    pub active_color: Option<Rgba>,
    pub width: Option<f32>,
}

impl BarLook {
    /// Reads the `scrollbar` attribute: JSON `false`, `true` or `{ color, activeColor, width }`.
    pub fn parse(attribute: Option<&str>) -> BarLook {
        let Some(json) = attribute else {
            return BarLook::default();
        };
        match serde_json::from_str::<Value>(json) {
            Ok(Value::Bool(shown)) => BarLook { hidden: !shown, ..BarLook::default() },
            Ok(Value::Object(fields)) => BarLook {
                hidden: false,
                color: fields.get("color").and_then(Value::as_str).and_then(parse_colour),
                active_color: fields.get("activeColor").and_then(Value::as_str).and_then(parse_colour),
                width: fields.get("width").and_then(Value::as_f64).map(|width| width as f32),
            },
            _ => BarLook::default(),
        }
    }

    /// The thumb's colour, idle or near, given the text colour around the scroller.
    pub fn thumb_color(&self, text: Rgba, near: bool) -> Rgba {
        let own = if near { self.active_color.or(self.color.map(brighter)) } else { self.color };
        match own {
            Some(color) => color,
            None => Rgba { a: if near { 0.65 } else { 0.38 }, ..text },
        }
    }

    pub fn thickness(&self, near: bool) -> f32 {
        let idle = self.width.unwrap_or(WIDTH).max(1.0);
        if near { idle + WIDTH_NEAR } else { idle }
    }
}

/// A colour a little more opaque, for a thumb the app tinted but gave no colour for near states.
fn brighter(color: Rgba) -> Rgba {
    Rgba { a: (color.a * 1.6).min(1.0), ..color }
}

/// Where a scroller stands along its axis, in pixels. With a virtual scroller (a `List` taller than the
/// host lays out at once) `content` and `scrolled` count over the whole of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub viewport: f32,
    pub content: f32,
    pub scrolled: f32,
}

/// A thumb along the axis, counted from the scroller's start.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thumb {
    pub start: f32,
    pub length: f32,
}

impl Metrics {
    /// How far the content can scroll.
    pub fn furthest(&self) -> f32 {
        (self.content - self.viewport).max(0.0)
    }

    fn track(&self) -> f32 {
        (self.viewport - 2.0 * INSET).max(0.0)
    }

    /// The thumb for this position, or none when the content fits.
    pub fn thumb(&self) -> Option<Thumb> {
        let furthest = self.furthest();
        let track = self.track();
        if furthest < 0.5 || track <= 0.0 || self.content <= 0.0 {
            return None;
        }
        let length = (track * self.viewport / self.content).clamp(MIN_THUMB.min(track), track);
        let share = (self.scrolled / furthest).clamp(0.0, 1.0);
        Some(Thumb { start: INSET + share * (track - length), length })
    }

    /// The offset a thumb of `length` standing at `start` stands for.
    pub fn scrolled_for(&self, length: f32, start: f32) -> f32 {
        let room = self.track() - length;
        if room <= 0.0 {
            return 0.0;
        }
        ((start - INSET) / room).clamp(0.0, 1.0) * self.furthest()
    }
}

/// How visible the bar is `idle` after the last scroll or pointer move: fully while held or within `HOLD`,
/// then fading to nothing.
pub fn opacity(idle: Duration, held: bool) -> f32 {
    if held || idle <= HOLD {
        return 1.0;
    }
    let fading = (idle - HOLD).as_secs_f32() / FADE.as_secs_f32();
    (1.0 - fading).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn metrics(viewport: f32, content: f32, scrolled: f32) -> Metrics {
        Metrics { viewport, content, scrolled }
    }

    #[test]
    fn content_that_fits_has_no_thumb() {
        assert_eq!(metrics(400.0, 400.0, 0.0).thumb(), None);
        assert_eq!(metrics(400.0, 300.0, 0.0).thumb(), None);
    }

    #[test]
    fn the_thumb_is_as_long_as_the_window_is_of_the_content() {
        let thumb = metrics(400.0, 800.0, 0.0).thumb().unwrap();
        assert!((thumb.length - 198.0).abs() < 0.01, "half of a 396 pixel track: {}", thumb.length);
        assert_eq!(thumb.start, INSET);
    }

    #[test]
    fn the_thumb_ends_at_the_end_of_the_track_when_the_content_does() {
        let thumb = metrics(400.0, 800.0, 400.0).thumb().unwrap();
        assert!((thumb.start + thumb.length - (400.0 - INSET)).abs() < 0.01);
    }

    #[test]
    fn a_very_long_list_keeps_a_thumb_that_can_be_held() {
        let thumb = metrics(400.0, 2_800_000.0, 1_000_000.0).thumb().unwrap();
        assert_eq!(thumb.length, MIN_THUMB);
        assert!(thumb.start > INSET && thumb.start + thumb.length < 400.0 - INSET);
    }

    #[test]
    fn dragging_the_thumb_to_each_end_scrolls_to_each_end() {
        let at = metrics(400.0, 800.0, 100.0);
        let thumb = at.thumb().unwrap();
        assert_eq!(at.scrolled_for(thumb.length, INSET), 0.0);
        let last = 400.0 - INSET - thumb.length;
        assert!((at.scrolled_for(thumb.length, last) - 400.0).abs() < 0.01);
        assert_eq!(at.scrolled_for(thumb.length, -50.0), 0.0);
        assert!((at.scrolled_for(thumb.length, 9999.0) - 400.0).abs() < 0.01);
    }

    #[test]
    fn a_thumb_put_where_it_is_stands_for_where_it_is() {
        let at = metrics(400.0, 5000.0, 1234.0);
        let thumb = at.thumb().unwrap();
        assert!((at.scrolled_for(thumb.length, thumb.start) - 1234.0).abs() < 0.01);
    }

    #[test]
    fn the_bar_holds_then_fades_then_goes() {
        assert_eq!(opacity(Duration::from_millis(900), false), 1.0);
        assert_eq!(opacity(Duration::from_millis(1100), false), 0.5);
        assert_eq!(opacity(Duration::from_millis(1300), false), 0.0);
        assert_eq!(opacity(Duration::from_millis(5000), true), 1.0);
    }

    #[test]
    fn the_look_reads_false_true_and_tints() {
        assert!(BarLook::parse(Some("false")).hidden);
        assert!(!BarLook::parse(Some("true")).hidden);
        assert_eq!(BarLook::parse(None), BarLook::default());
        let tinted = BarLook::parse(Some(r##"{"color":"#ff0000","width":8}"##));
        assert_eq!(tinted.color.map(|color| color.r), Some(1.0));
        assert_eq!(tinted.width, Some(8.0));
        assert!(tinted.thumb_color(Rgba::default(), true).a >= tinted.thumb_color(Rgba::default(), false).a);
    }
}
