//! What a primitive looks like. The app passes a style object, which crosses as JSON
//! and is parsed here once per distinct string, then put onto GPUI elements through the
//! `Styled` trait. The same parse serves `hover`, `active` and `focus`, which GPUI takes
//! as refinements: only what such a patch sets is applied.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{CursorStyle, Rgba, SharedString};
use serde_json::Value;

mod apply;
mod parse;

pub use apply::apply;
pub use parse::parse_colour;

/// A size: pixels, a share of the parent, or `auto`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Px(f32),
    Percent(f32),
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Justify {
    Start,
    End,
    Center,
    Between,
    Around,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// The four sides: top, right, bottom, left.
pub type Sides<T> = [T; 4];

/// A timing function: a cubic Bezier through (0,0) and (1,1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Easing(pub f32, pub f32, pub f32, pub f32);

impl Easing {
    pub const EASE: Easing = Easing(0.25, 0.1, 0.25, 1.0);
    pub const LINEAR: Easing = Easing(0.0, 0.0, 1.0, 1.0);

    /// The eased progress for linear progress `time`, between 0 and 1.
    pub fn at(&self, time: f32) -> f32 {
        if *self == Easing::LINEAR {
            return time;
        }
        let Easing(x1, y1, x2, y2) = *self;
        let curve = |a: f32, b: f32, t: f32| 3.0 * a * (1.0 - t).powi(2) * t + 3.0 * b * (1.0 - t) * t * t + t.powi(3);
        let mut t = time;
        for _ in 0..8 {
            let error = curve(x1, x2, t) - time;
            let slope = (curve(x1, x2, t + 1e-3) - curve(x1, x2, t - 1e-3)) / 2e-3;
            if slope.abs() < 1e-6 {
                break;
            }
            t = (t - error / slope).clamp(0.0, 1.0);
        }
        curve(y1, y2, t)
    }
}

/// How changes to colours and opacity ease: `transition: 150` or `{ duration, delay, easing }`.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub duration_ms: f32,
    pub delay_ms: f32,
    pub easing: Easing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
}

/// A parsed style object. Every field is optional, so a patch applies only what it names.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub hidden: Option<bool>,
    pub row: Option<bool>,
    pub wrap: Option<bool>,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<Dimension>,
    pub align_items: Option<Align>,
    pub align_self: Option<Align>,
    pub justify: Option<Justify>,
    pub row_gap: Option<f32>,
    pub column_gap: Option<f32>,
    pub padding: Sides<Option<f32>>,
    pub margin: Sides<Option<Dimension>>,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,
    pub aspect_ratio: Option<f32>,
    pub absolute: Option<bool>,
    pub inset: Sides<Option<Dimension>>,
    pub clip: Option<bool>,
    /// An offset from where layout put the element, in pixels.
    pub translate: Option<(f32, f32)>,
    pub transition: Option<Transition>,
    pub background: Option<Rgba>,
    pub opacity: Option<f32>,
    pub border_width: Sides<Option<f32>>,
    pub border_color: Option<Rgba>,
    /// Corner radii: top left, top right, bottom right, bottom left.
    pub radius: [Option<f32>; 4],
    pub shadow: Option<Vec<Shadow>>,
    pub color: Option<Rgba>,
    pub font_size: Option<f32>,
    pub font_weight: Option<f32>,
    pub font_family: Option<SharedString>,
    pub line_height: Option<f32>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub text_align: Option<TextAlign>,
    pub nowrap: Option<bool>,
    pub truncate: Option<bool>,
    pub max_lines: Option<usize>,
    pub uppercase: Option<bool>,
    /// How far an element in a paragraph's lines sits above the baseline, in pixels.
    pub vertical_align: Option<f32>,
    pub cursor: Option<CursorStyle>,
}

pub const SANS: &str = "Geist";
pub const MONO: &str = "JetBrainsMono Nerd Font Mono";

thread_local! {
    static PARSED: RefCell<HashMap<String, Rc<Style>>> = RefCell::new(HashMap::new());
}

/// The style a JSON string describes, parsed once however many elements share it.
pub fn parsed(json: &str) -> Rc<Style> {
    if let Some(found) = PARSED.with(|cache| cache.borrow().get(json).cloned()) {
        return found;
    }
    let style = Rc::new(match serde_json::from_str::<Value>(json) {
        Ok(Value::Object(fields)) => {
            let mut style = Style::default();
            for (name, value) in &fields {
                if !style.set(name, value) {
                    eprintln!("svelte-gpui: style property {name}: {value} is not understood");
                }
            }
            style
        }
        _ => {
            eprintln!("svelte-gpui: a style is not an object: {json}");
            Style::default()
        }
    });
    PARSED.with(|cache| {
        let mut cache = cache.borrow_mut();
        // Styles that animate a position make a new string every frame; forgetting them all
        // now and then keeps memory flat, and the ones still in use are parsed again.
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(json.to_string(), style.clone());
    });
    style
}

impl Style {
    /// Lays `other` over this style: whatever `other` sets wins.
    pub fn overlay(&mut self, other: &Style) {
        macro_rules! take {
            ($($field:ident),*) => {
                $(if other.$field.is_some() { self.$field = other.$field.clone(); })*
            };
        }
        take!(
            hidden,
            row,
            wrap,
            grow,
            shrink,
            basis,
            align_items,
            align_self,
            justify,
            row_gap,
            column_gap,
            width,
            height,
            min_width,
            min_height,
            max_width,
            max_height,
            aspect_ratio,
            absolute,
            clip,
            translate,
            transition,
            background,
            opacity,
            border_color,
            shadow,
            color,
            font_size,
            font_weight,
            font_family,
            line_height,
            italic,
            underline,
            strikethrough,
            text_align,
            nowrap,
            truncate,
            max_lines,
            uppercase,
            vertical_align,
            cursor
        );
        for side in 0..4 {
            if other.padding[side].is_some() {
                self.padding[side] = other.padding[side];
            }
            if other.margin[side].is_some() {
                self.margin[side] = other.margin[side];
            }
            if other.inset[side].is_some() {
                self.inset[side] = other.inset[side];
            }
            if other.border_width[side].is_some() {
                self.border_width[side] = other.border_width[side];
            }
            if other.radius[side].is_some() {
                self.radius[side] = other.radius[side];
            }
        }
    }
}
