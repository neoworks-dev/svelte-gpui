//! What `background` can be: a colour, or a linear gradient between two colours.

use gpui::{Background, Hsla, Rgba, linear_color_stop, linear_gradient, solid_background};
use serde_json::Value;

use super::parse::parse_colour;

/// One end of a gradient: a colour, and how far along the gradient line it sits, from 0 at the
/// start of the box to 1 at its end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stop {
    pub colour: Rgba,
    pub at: f32,
}

/// A linear gradient. GPUI's renderer draws exactly two stops, so that is all there is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    /// Degrees, as in CSS: 0 points up and a larger angle turns clockwise, so 180 runs downwards.
    pub angle: f32,
    pub from: Stop,
    pub to: Stop,
}

/// A `background`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    Solid(Rgba),
    Gradient(Gradient),
}

/// A colour easing towards another, in premultiplied alpha as CSS does: a colour that is
/// fading in or out keeps its own hue, and does not pass through black on its way from or to
/// `transparent`.
pub fn mix(from: Rgba, to: Rgba, amount: f32) -> Rgba {
    let from_weight = from.a * (1.0 - amount);
    let to_weight = to.a * amount;
    let alpha = from_weight + to_weight;
    if alpha <= 0.0 {
        return Rgba { a: 0.0, ..to };
    }
    let channel = |from_channel: f32, to_channel: f32| (from_channel * from_weight + to_channel * to_weight) / alpha;
    Rgba { r: channel(from.r, to.r), g: channel(from.g, to.g), b: channel(from.b, to.b), a: alpha }
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

impl Stop {
    fn mix(from: Stop, to: Stop, amount: f32) -> Stop {
        Stop { colour: mix(from.colour, to.colour, amount), at: lerp(from.at, to.at, amount) }
    }

    fn faded(self) -> Stop {
        Stop { colour: Rgba { a: 0.0, ..self.colour }, ..self }
    }
}

impl Gradient {
    /// A gradient that shows `colour` all over, in the direction of `like`: what a plain colour
    /// is next to a gradient it eases to or from.
    fn flat(colour: Rgba, like: &Gradient) -> Gradient {
        Gradient { angle: like.angle, from: Stop { colour, at: like.from.at }, to: Stop { colour, at: like.to.at } }
    }

    /// The same gradient with nothing in it, which is what a missing background is next to one.
    fn faded(&self) -> Gradient {
        Gradient { angle: self.angle, from: self.from.faded(), to: self.to.faded() }
    }

    fn mix(from: &Gradient, to: &Gradient, amount: f32) -> Gradient {
        Gradient {
            angle: lerp(from.angle, to.angle, amount),
            from: Stop::mix(from.from, to.from, amount),
            to: Stop::mix(from.to, to.to, amount),
        }
    }

    /// The shader mixes the two colours without premultiplying them, so a fully transparent
    /// end takes the other end's hue, as CSS's premultiplied mixing would give it: a fade to
    /// `transparent` stays the colour it fades from instead of going grey on the way.
    fn with_hue_for_transparent_ends(mut self) -> Gradient {
        if self.from.colour.a == 0.0 {
            self.from.colour = Rgba { a: 0.0, ..self.to.colour };
        } else if self.to.colour.a == 0.0 {
            self.to.colour = Rgba { a: 0.0, ..self.from.colour };
        }
        self
    }
}

impl Fill {
    /// The colour, when this is a plain one.
    pub fn solid(&self) -> Option<Rgba> {
        match self {
            Fill::Solid(colour) => Some(*colour),
            Fill::Gradient(_) => None,
        }
    }

    /// What GPUI draws.
    pub fn to_gpui(&self) -> Background {
        match self {
            Fill::Solid(colour) => solid_background(Hsla::from(*colour)),
            Fill::Gradient(gradient) => {
                let gradient = gradient.with_hue_for_transparent_ends();
                let stop = |stop: Stop| linear_color_stop(Hsla::from(stop.colour), stop.at);
                linear_gradient(gradient.angle, stop(gradient.from), stop(gradient.to))
            }
        }
    }

    /// Where a transition from `from` to `to` has got to. A plain colour next to a gradient
    /// counts as a gradient of that one colour, and a missing one as a gradient of nothing.
    pub fn ease(from: Option<Fill>, to: Option<Fill>, amount: f32) -> Option<Fill> {
        let (from_gradient, to_gradient) = match (from, to) {
            (Some(Fill::Gradient(from)), Some(Fill::Gradient(to))) => (from, to),
            (Some(Fill::Gradient(from)), Some(Fill::Solid(to))) => (from, Gradient::flat(to, &from)),
            (Some(Fill::Solid(from)), Some(Fill::Gradient(to))) => (Gradient::flat(from, &to), to),
            (Some(Fill::Gradient(from)), None) => (from, from.faded()),
            (None, Some(Fill::Gradient(to))) => (to.faded(), to),
            _ => return Fill::ease_colours(from, to, amount),
        };
        Some(Fill::Gradient(Gradient::mix(&from_gradient, &to_gradient, amount)))
    }

    /// Easing between plain colours; a missing one counts as transparent.
    fn ease_colours(from: Option<Fill>, to: Option<Fill>, amount: f32) -> Option<Fill> {
        let from = from.and_then(|fill| fill.solid());
        let to = to.and_then(|fill| fill.solid());
        let colour = match (from, to) {
            (Some(from), Some(to)) => mix(from, to, amount),
            (None, Some(to)) => mix(Rgba { a: 0.0, ..to }, to, amount),
            (Some(from), None) => mix(from, Rgba { a: 0.0, ..from }, amount),
            (None, None) => return None,
        };
        Some(Fill::Solid(colour))
    }

    /// A colour string, or a gradient object: `{ angle?, from, to }`.
    pub fn parse(value: &Value) -> Option<Fill> {
        match value {
            Value::String(text) => parse_colour(text).map(Fill::Solid),
            Value::Object(fields) => {
                let angle = match fields.get("angle") {
                    None => 180.0,
                    Some(angle) => parse_angle(angle)?,
                };
                let from = parse_stop(fields.get("from")?, 0.0)?;
                let to = parse_stop(fields.get("to")?, 1.0)?;
                Some(Fill::Gradient(Gradient { angle, from, to }))
            }
            _ => None,
        }
    }
}

/// Degrees, or a side to run towards: `"to top"`, `"to right"`, `"to bottom"` or `"to left"`.
fn parse_angle(value: &Value) -> Option<f32> {
    if let Some(degrees) = value.as_f64() {
        return Some(degrees as f32);
    }
    match value.as_str()? {
        "to top" => Some(0.0),
        "to right" => Some(90.0),
        "to bottom" => Some(180.0),
        "to left" => Some(270.0),
        _ => None,
    }
}

/// A colour string, or `{ color, at }` with `at` between 0 and 1; `default_at` when it is left out.
fn parse_stop(value: &Value, default_at: f32) -> Option<Stop> {
    if let Value::String(text) = value {
        return Some(Stop { colour: parse_colour(text)?, at: default_at });
    }
    let fields = value.as_object()?;
    let colour = parse_colour(fields.get("color")?.as_str()?)?;
    let at = match fields.get("at") {
        None => default_at,
        Some(at) => at.as_f64()? as f32,
    };
    Some(Stop { colour, at })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const RED: Rgba = Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    const BLUE: Rgba = Rgba { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

    fn gradient(fill: Option<Fill>) -> Gradient {
        match fill {
            Some(Fill::Gradient(gradient)) => gradient,
            other => panic!("not a gradient: {other:?}"),
        }
    }

    #[test]
    fn a_colour_string_is_a_plain_fill() {
        assert_eq!(Fill::parse(&json!("#ff0000")), Some(Fill::Solid(RED)));
    }

    #[test]
    fn a_gradient_defaults_to_downwards_over_the_whole_box() {
        let parsed = gradient(Fill::parse(&json!({ "from": "#ff0000", "to": "#0000ff" })));
        assert_eq!(parsed.angle, 180.0);
        assert_eq!(parsed.from, Stop { colour: RED, at: 0.0 });
        assert_eq!(parsed.to, Stop { colour: BLUE, at: 1.0 });
    }

    #[test]
    fn angles_are_degrees_or_a_side() {
        let angle = |value| gradient(Fill::parse(&json!({ "angle": value, "from": "#f00", "to": "#00f" }))).angle;
        assert_eq!(angle(json!(45)), 45.0);
        assert_eq!(angle(json!("to top")), 0.0);
        assert_eq!(angle(json!("to right")), 90.0);
        assert_eq!(angle(json!("to bottom")), 180.0);
        assert_eq!(angle(json!("to left")), 270.0);
    }

    #[test]
    fn stops_take_a_position() {
        let parsed = gradient(Fill::parse(&json!({ "from": { "color": "#f00", "at": 0.25 }, "to": "#00f" })));
        assert_eq!(parsed.from.at, 0.25);
        assert_eq!(parsed.to.at, 1.0);
    }

    #[test]
    fn a_gradient_missing_an_end_or_with_a_bad_colour_is_refused() {
        assert_eq!(Fill::parse(&json!({ "from": "#f00" })), None);
        assert_eq!(Fill::parse(&json!({ "from": "#f00", "to": "nonsense" })), None);
        assert_eq!(Fill::parse(&json!({ "angle": "to nowhere", "from": "#f00", "to": "#00f" })), None);
        assert_eq!(Fill::parse(&json!(12)), None);
    }

    #[test]
    fn a_fade_to_transparent_keeps_the_hue_it_fades_from() {
        let fill = Fill::parse(&json!({ "from": "#ff0000", "to": "transparent" })).unwrap();
        let drawn = format!("{:?}", fill.to_gpui());
        let kept = format!(
            "{:?}",
            Fill::Gradient(Gradient {
                angle: 180.0,
                from: Stop { colour: RED, at: 0.0 },
                to: Stop { colour: Rgba { a: 0.0, ..RED }, at: 1.0 },
            })
            .to_gpui()
        );
        assert_eq!(drawn, kept);
    }

    #[test]
    fn gradients_ease_stop_by_stop() {
        let from = Fill::parse(&json!({ "from": "#ff0000", "to": "#ff0000", "angle": 0 }));
        let to = Fill::parse(&json!({ "from": "#0000ff", "to": "#0000ff", "angle": 180 }));
        let halfway = gradient(Fill::ease(from, to, 0.5));
        assert_eq!(halfway.angle, 90.0);
        assert!((halfway.from.colour.r - 0.5).abs() < 1e-6);
        assert!((halfway.to.colour.b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn a_plain_colour_eases_into_a_gradient_without_a_jump() {
        let to = Fill::parse(&json!({ "from": "#0000ff", "to": "transparent" }));
        let start = gradient(Fill::ease(Some(Fill::Solid(RED)), to, 0.0));
        assert_eq!(start.from.colour, RED);
        assert_eq!(start.to.colour, RED);
        let nothing = gradient(Fill::ease(None, to, 0.0));
        assert_eq!(nothing.from.colour.a, 0.0);
    }

    #[test]
    fn plain_colours_still_ease_in_premultiplied_alpha() {
        let halfway = Fill::ease(None, Some(Fill::Solid(RED)), 0.5);
        assert_eq!(halfway, Some(Fill::Solid(Rgba { a: 0.5, ..RED })));
        assert_eq!(Fill::ease(None, None, 0.5), None);
    }
}
