//! Turns an element's classes and inline declarations into the few properties a
//! GPUI element can take. Which rules apply comes from the stylesheet; what a
//! declaration means is decided here, one property at a time.

use std::collections::{HashMap, HashSet};

use gpui::Rgba;

use crate::tree::{Kind, NodeId, Tree};

use crate::css::{
    Length, Pseudo, Rule, Stylesheet, Variables, parse_colour, parse_length, parse_number,
    split_top_level,
};

/// A size that may be unset, a number of pixels, a share of the parent, or `auto`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Px(f32),
    Percent(f32),
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Display {
    #[default]
    Block,
    Flex,
    /// `inline` and friends, which GPUI cannot flow with text: drawn as a row.
    Inline,
    None,
    Contents,
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

/// The four sides, in CSS order: top, right, bottom, left.
pub type Sides<T> = [T; 4];

/// A CSS timing function: a cubic Bezier through (0,0) and (1,1).
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

/// What a `transition` declares: which properties ease, and how.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub colours: bool,
    pub opacity: bool,
    pub duration_ms: f32,
    pub delay_ms: f32,
    pub easing: Easing,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            colours: false,
            opacity: false,
            duration_ms: 0.0,
            delay_ms: 0.0,
            easing: Easing::EASE,
        }
    }
}

/// How an image fills its box: `object-fit`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Fit {
    #[default]
    Contain,
    Cover,
    Fill,
}

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub display: Display,
    pub column: bool,
    pub wrap: bool,
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
    pub absolute: bool,
    pub inset: Sides<Option<Dimension>>,
    pub clip_x: bool,
    pub clip_y: bool,
    pub scroll_x: bool,
    pub scroll_y: bool,
    pub fit: Fit,
    /// `transform: translate(...)`, in pixels; the only transform a div can take.
    pub translate: (f32, f32),
    pub transition: Transition,
    pub background: Option<Rgba>,
    pub color: Option<Rgba>,
    pub opacity: Option<f32>,
    pub border_width: Sides<f32>,
    pub border_color: Sides<Option<Rgba>>,
    /// Corner radii: top left, top right, bottom right, bottom left.
    pub radius: [f32; 4],
    pub font_size: Option<f32>,
    pub font_weight: Option<f32>,
    pub mono: Option<bool>,
    pub line_height: Option<f32>,
    pub italic: bool,
    pub underline: bool,
    pub center_text: bool,
    pub end_text: bool,
    pub nowrap: bool,
    pub ellipsis: bool,
    pub uppercase: bool,
    pub pointer: bool,
}

/// The parts of an element's state that pseudo-classes ask about.
#[derive(Debug, Clone, Copy, Default)]
pub struct ElementState {
    pub checked: bool,
    pub disabled: bool,
}

/// What `:hover` changes. GPUI's hover refinement can take these.
#[derive(Debug, Clone, Default)]
pub struct HoverStyle {
    pub background: Option<Rgba>,
    pub color: Option<Rgba>,
    pub opacity: Option<f32>,
    pub border_color: Option<Rgba>,
}

impl HoverStyle {
    pub fn is_empty(&self) -> bool {
        self.background.is_none()
            && self.color.is_none()
            && self.opacity.is_none()
            && self.border_color.is_none()
    }
}

/// Everything a node needs to be drawn.
#[derive(Clone)]
pub struct Computed {
    pub style: Style,
    pub hover: HoverStyle,
    /// Changes while an ancestor carrying a group class (`group`, or `group/name`) is hovered,
    /// by the class's name.
    pub group_hover: Vec<(String, HoverStyle)>,
}

/// Properties that matter for animation or input affordances and nothing for a still frame.
const IGNORED: &[&str] = &[
    "transition-behavior", "animation", "user-select", "pointer-events", "outline", "outline-style", "outline-width",
    "outline-color", "outline-offset", "box-sizing", "will-change", "scrollbar-width",
    "-webkit-user-select", "appearance", "-webkit-appearance", "z-index", "list-style",
    "text-decoration-line", "text-underline-offset", "letter-spacing", "word-break",
    "overflow-wrap", "-webkit-font-smoothing", "font-feature-settings", "vertical-align",
    "touch-action", "border-style", "border-top-style", "border-right-style",
    "border-bottom-style", "border-left-style", "--tw-border-style", "visibility",
    "tab-size", "-webkit-text-size-adjust", "-webkit-tap-highlight-color", "font-variation-settings",
    "text-wrap", "text-wrap-mode", "resize", "caret-color", "accent-color", "fill", "stroke",
    "stroke-width", "flex-shrink-0", "text-transform-none",
    "scale", "rotate", "filter", "backdrop-filter", "box-shadow", "--tw-shadow",
    "mask", "mix-blend-mode", "background-image", "place-items", "place-content", "isolation",
    "contain", "text-indent", "hyphens", "white-space-collapse", "font-variant-numeric",
    "text-decoration-thickness", "font-stretch", "text-size-adjust", "content",
];

pub struct Resolver {
    pub sheet: Stylesheet,
    unsupported: std::cell::RefCell<HashSet<String>>,
}

impl Resolver {
    pub fn new(sheet: Stylesheet) -> Self {
        Self {
            sheet,
            unsupported: std::cell::RefCell::new(HashSet::new()),
        }
    }

    /// Declarations the engine saw and could not honour, for a coverage report.
    pub fn unsupported(&self) -> Vec<String> {
        let mut found: Vec<String> = self.unsupported.borrow().iter().cloned().collect();
        found.sort();
        found
    }

    /// The style the document root starts from: the stylesheet's `html` and `:root` rules.
    pub fn root_style(&self) -> Style {
        let mut style = Style::default();
        for (name, value) in &self.sheet.root_declarations {
            self.apply(&mut style, name, value);
        }
        style
    }

    pub fn compute(&self, tree: &Tree, id: NodeId) -> Computed {
        let Some(node) = tree.nodes.get(&id) else {
            return Computed {
                style: Style::default(),
                hover: HoverStyle::default(),
                group_hover: Vec::new(),
            };
        };
        let tag = match &node.kind {
            Kind::Element { tag } => tag.as_str(),
            Kind::Text { .. } => "span",
        };
        let class_attribute = node.attributes.get("class").map(String::as_str).unwrap_or("");
        let state = ElementState {
            checked: node.attributes.get("checked").map(String::as_str) == Some("true"),
            disabled: node.attributes.contains_key("disabled"),
        };
        let inline = &node.style;
        let classes: Vec<&str> = class_attribute.split_whitespace().collect();
        let mut indices: Vec<usize> = Vec::new();
        indices.extend_from_slice(self.sheet.rules_for_tag(tag));
        indices.extend_from_slice(self.sheet.unindexed_rules());
        for class in &classes {
            indices.extend_from_slice(self.sheet.rules_for_class(class));
        }
        indices.sort_unstable();
        indices.dedup();

        let mut computed = Computed {
            style: default_for_tag(tag),
            hover: HoverStyle::default(),
            group_hover: Vec::new(),
        };
        let mut hover_style = Style::default();
        let mut group_styles: Vec<(String, Style)> = Vec::new();
        for index in indices {
            let rule = &self.sheet.rules[index];
            if !rule.selector.matches(tree, id) {
                continue;
            }
            match &rule.selector.pseudo {
                Pseudo::None => self.apply_all(&mut computed.style, rule),
                Pseudo::Hover => self.apply_all(&mut hover_style, rule),
                Pseudo::GroupHover(group) => {
                    let position = group_styles.iter().position(|(name, _)| name == group);
                    let position = match position {
                        Some(position) => position,
                        None => {
                            group_styles.push((group.clone(), Style::default()));
                            group_styles.len() - 1
                        }
                    };
                    self.apply_all(&mut group_styles[position].1, rule);
                }
                Pseudo::Checked if state.checked => self.apply_all(&mut computed.style, rule),
                Pseudo::Disabled if state.disabled => self.apply_all(&mut computed.style, rule),
                Pseudo::Checked | Pseudo::Focus | Pseudo::Disabled | Pseudo::Other => {}
            }
        }
        for (name, value) in inline {
            self.apply(&mut computed.style, name, value);
        }
        centre_control_content(tag, &mut computed.style);
        computed.hover = hover_of(&hover_style);
        computed.group_hover = group_styles
            .iter()
            .map(|(name, style)| (name.clone(), hover_of(style)))
            .collect();
        computed
    }

    fn apply_all(&self, style: &mut Style, rule: &Rule) {
        let local: HashMap<String, String> = rule
            .declarations
            .iter()
            .filter(|(name, _)| name.starts_with("--"))
            .cloned()
            .collect();
        let overlay = if local.is_empty() { None } else { Some(&local) };
        for (name, value) in &rule.declarations {
            self.apply_with(style, name, value, overlay);
        }
    }

    fn variables<'a>(&'a self, local: Option<&'a HashMap<String, String>>) -> Variables<'a> {
        Variables {
            root: &self.sheet.variables,
            local,
        }
    }

    fn note_unsupported(&self, name: &str, value: &str) {
        self.unsupported.borrow_mut().insert(format!("{name}: {value}"));
    }

    /// Applies one declaration to a style.
    pub fn apply(&self, style: &mut Style, name: &str, raw_value: &str) {
        self.apply_with(style, name, raw_value, None);
    }

    fn apply_with(&self, style: &mut Style, name: &str, raw_value: &str, local: Option<&HashMap<String, String>>) {
        if name.starts_with("--") || IGNORED.contains(&name) {
            return;
        }
        let value = self.variables(local).substitute(raw_value);
        let value = value.trim();
        if value.is_empty() {
            return;
        }
        let known = self.apply_layout(style, name, value)
            || self.apply_box(style, name, value)
            || self.apply_text(style, name, value);
        if !known {
            self.note_unsupported(name, raw_value);
        }
    }

    fn dimension(&self, value: &str) -> Option<Dimension> {
        if value == "auto" {
            return Some(Dimension::Auto);
        }
        match parse_length(value)? {
            Length::Px(pixels) => Some(Dimension::Px(pixels)),
            Length::Percent(percent) => Some(Dimension::Percent(percent)),
        }
    }

    fn pixels(&self, value: &str) -> Option<f32> {
        match parse_length(value)? {
            Length::Px(pixels) => Some(pixels),
            Length::Percent(_) => None,
        }
    }

    fn apply_layout(&self, style: &mut Style, name: &str, value: &str) -> bool {
        match name {
            "display" => {
                style.display = match value {
                    "flex" | "inline-flex" | "grid" | "inline-grid" => Display::Flex,
                    "block" | "flow-root" | "list-item" => Display::Block,
                    "inline" | "inline-block" => Display::Inline,
                    "none" => Display::None,
                    "contents" => Display::Contents,
                    _ => return false,
                };
            }
            "flex-direction" => style.column = value.starts_with("column"),
            "flex-wrap" => style.wrap = value.starts_with("wrap"),
            "flex" => return self.apply_flex(style, value),
            "flex-grow" => style.grow = parse_number(value),
            "flex-shrink" => style.shrink = parse_number(value),
            "flex-basis" => style.basis = self.dimension(value),
            "align-items" => style.align_items = align_of(value),
            "align-self" => style.align_self = align_of(value),
            "justify-content" => style.justify = justify_of(value),
            "gap" => return self.apply_gap(style, value),
            "row-gap" => style.row_gap = self.pixels(value),
            "column-gap" => style.column_gap = self.pixels(value),
            "width" => style.width = self.dimension(value),
            "height" => style.height = self.dimension(value),
            "min-width" => style.min_width = self.dimension(value),
            "min-height" => style.min_height = self.dimension(value),
            "max-width" => style.max_width = self.dimension(value),
            "max-height" => style.max_height = self.dimension(value),
            "position" => style.absolute = matches!(value, "absolute" | "fixed"),
            "top" => style.inset[0] = self.dimension(value),
            "right" => style.inset[1] = self.dimension(value),
            "bottom" => style.inset[2] = self.dimension(value),
            "left" => style.inset[3] = self.dimension(value),
            "inset" => style.inset = self.dimension_sides(value),
            "inset-inline" | "inset-block" => return false,
            "overflow" => self.apply_overflow(style, value, true, true),
            "overflow-x" => self.apply_overflow(style, value, true, false),
            "overflow-y" => self.apply_overflow(style, value, false, true),
            _ => return false,
        }
        true
    }

    fn apply_overflow(&self, style: &mut Style, value: &str, horizontal: bool, vertical: bool) {
        let clipped = matches!(value, "hidden" | "clip" | "auto" | "scroll");
        if horizontal {
            style.clip_x = clipped;
            style.scroll_x = matches!(value, "auto" | "scroll");
        }
        if vertical {
            style.clip_y = clipped;
            style.scroll_y = matches!(value, "auto" | "scroll");
        }
    }

    fn apply_flex(&self, style: &mut Style, value: &str) -> bool {
        match value {
            "none" => {
                style.grow = Some(0.0);
                style.shrink = Some(0.0);
                style.basis = Some(Dimension::Auto);
                return true;
            }
            "auto" => {
                style.grow = Some(1.0);
                style.shrink = Some(1.0);
                style.basis = Some(Dimension::Auto);
                return true;
            }
            "initial" => {
                style.grow = Some(0.0);
                style.shrink = Some(1.0);
                style.basis = Some(Dimension::Auto);
                return true;
            }
            _ => {}
        }
        let parts: Vec<&str> = value.split_whitespace().collect();
        let Some(grow) = parts.first().and_then(|part| parse_number(part)) else {
            return false;
        };
        style.grow = Some(grow);
        style.shrink = Some(1.0);
        style.basis = Some(Dimension::Percent(0.0));
        if let Some(second) = parts.get(1) {
            match parse_number(second) {
                Some(shrink) => style.shrink = Some(shrink),
                None => style.basis = self.dimension(second),
            }
        }
        if let Some(third) = parts.get(2) {
            style.basis = self.dimension(third);
        }
        true
    }

    fn apply_gap(&self, style: &mut Style, value: &str) -> bool {
        let parts = split_values(value);
        let Some(row) = parts.first().and_then(|part| self.pixels(part)) else {
            return false;
        };
        style.row_gap = Some(row);
        style.column_gap = Some(row);
        if let Some(column) = parts.get(1).and_then(|part| self.pixels(part)) {
            style.column_gap = Some(column);
        }
        true
    }

    fn dimension_sides(&self, value: &str) -> Sides<Option<Dimension>> {
        let parts: Vec<Option<Dimension>> = split_values(value)
            .iter()
            .map(|part| self.dimension(part))
            .collect();
        expand_sides(&parts)
    }

    fn pixel_sides(&self, value: &str) -> Sides<Option<f32>> {
        let parts: Vec<Option<f32>> = split_values(value)
            .iter()
            .map(|part| self.pixels(part))
            .collect();
        expand_sides(&parts)
    }

    fn apply_box(&self, style: &mut Style, name: &str, value: &str) -> bool {
        if let Some(corner) = corner_of(name) {
            style.radius[corner] = self.pixels(value).unwrap_or(0.0);
            return true;
        }
        if let Some(side) = side_of(name, "padding") {
            return self.apply_padding(style, side, value);
        }
        if let Some(side) = side_of(name, "margin") {
            return self.apply_margin(style, side, value);
        }
        if let Some(side) = side_of(name, "border") {
            return self.apply_border_side(style, side, name, value);
        }
        match name {
            "background-color" | "background" => style.background = self.colour(value),
            "opacity" => style.opacity = parse_number(value),
            "border-radius" => self.apply_radius(style, value),
            "cursor" => style.pointer = value == "pointer",
            "transform" => style.translate = parse_translate(value),
            "translate" => style.translate = parse_translate(&format!("translate({})", value.replace(' ', ", "))),
            "transition-property" => self.apply_transition_property(style, value),
            "transition-duration" => style.transition.duration_ms = first_time(value),
            "transition-delay" => style.transition.delay_ms = first_time(value),
            "transition-timing-function" => style.transition.easing = parse_easing(value),
            "transition" => self.apply_transition_shorthand(style, value),
            "object-fit" => {
                style.fit = match value {
                    "cover" => Fit::Cover,
                    "fill" => Fit::Fill,
                    _ => Fit::Contain,
                };
            }
            "border-width" => {
                let widths = self.pixel_sides(value);
                for side in 0..4 {
                    style.border_width[side] = widths[side].unwrap_or(0.0);
                }
            }
            "border-color" => {
                let colour = self.colour(value);
                style.border_color = [colour; 4];
            }
            "border" => return self.apply_border_shorthand(style, value, None),
            _ => return false,
        }
        true
    }

    fn apply_transition_property(&self, style: &mut Style, value: &str) {
        style.transition.colours = false;
        style.transition.opacity = false;
        for name in value.split(',').map(str::trim) {
            match name {
                "all" => {
                    style.transition.colours = true;
                    style.transition.opacity = true;
                }
                "color" | "background-color" | "border-color" | "background" | "fill" | "stroke" => {
                    style.transition.colours = true;
                }
                "opacity" => style.transition.opacity = true,
                _ => {}
            }
        }
    }

    /// `transition: opacity 200ms ease-out 50ms`, for the first listed transition.
    fn apply_transition_shorthand(&self, style: &mut Style, value: &str) {
        let first = split_top_level(value, ',')[0].trim().to_string();
        if first == "none" {
            style.transition = Transition::default();
            return;
        }
        let mut times = Vec::new();
        let mut property = "all".to_string();
        style.transition.easing = Easing::EASE;
        for token in split_values(&first) {
            if token.ends_with("ms") || (token.ends_with('s') && token[..token.len() - 1].parse::<f32>().is_ok()) {
                times.push(first_time(token));
            } else if token.starts_with("cubic-bezier") || matches!(token, "ease" | "linear" | "ease-in" | "ease-out" | "ease-in-out") {
                style.transition.easing = parse_easing(token);
            } else {
                property = token.to_string();
            }
        }
        self.apply_transition_property(style, &property);
        style.transition.duration_ms = times.first().copied().unwrap_or(0.0);
        style.transition.delay_ms = times.get(1).copied().unwrap_or(0.0);
    }

    fn colour(&self, value: &str) -> Option<Rgba> {
        if value.eq_ignore_ascii_case("currentcolor") {
            return None;
        }
        parse_colour(value)
    }

    fn apply_padding(&self, style: &mut Style, side: Side, value: &str) -> bool {
        match side {
            Side::All => style.padding = self.pixel_sides(value),
            Side::One(index) => style.padding[index] = self.pixels(value),
            Side::Inline => {
                let both = self.pixels(value);
                style.padding[1] = both;
                style.padding[3] = both;
            }
            Side::Block => {
                let both = self.pixels(value);
                style.padding[0] = both;
                style.padding[2] = both;
            }
        }
        true
    }

    fn apply_margin(&self, style: &mut Style, side: Side, value: &str) -> bool {
        match side {
            Side::All => style.margin = self.dimension_sides(value),
            Side::One(index) => style.margin[index] = self.dimension(value),
            Side::Inline => {
                let both = self.dimension(value);
                style.margin[1] = both;
                style.margin[3] = both;
            }
            Side::Block => {
                let both = self.dimension(value);
                style.margin[0] = both;
                style.margin[2] = both;
            }
        }
        true
    }

    fn apply_border_side(&self, style: &mut Style, side: Side, name: &str, value: &str) -> bool {
        let Side::One(index) = side else {
            return false;
        };
        if name.ends_with("-width") {
            style.border_width[index] = self.pixels(value).unwrap_or(0.0);
            return true;
        }
        if name.ends_with("-color") {
            style.border_color[index] = self.colour(value);
            return true;
        }
        self.apply_border_shorthand(style, value, Some(index))
    }

    /// `border: 1px solid #fff`, on every side or one.
    fn apply_border_shorthand(&self, style: &mut Style, value: &str, side: Option<usize>) -> bool {
        let mut width = None;
        let mut colour = None;
        for part in split_values(value) {
            if matches!(part, "solid" | "dashed" | "dotted" | "none" | "hidden" | "double") {
                continue;
            }
            if let Some(pixels) = self.pixels(part) {
                width = Some(pixels);
                continue;
            }
            colour = self.colour(part);
        }
        let sides: Vec<usize> = match side {
            Some(index) => vec![index],
            None => vec![0, 1, 2, 3],
        };
        for index in sides {
            if let Some(width) = width {
                style.border_width[index] = width;
            }
            if colour.is_some() {
                style.border_color[index] = colour;
            }
        }
        true
    }

    fn apply_radius(&self, style: &mut Style, value: &str) {
        let first = value.split('/').next().unwrap_or(value);
        let parts: Vec<Option<f32>> = split_values(first)
            .iter()
            .map(|part| self.pixels(part))
            .collect();
        let corners = expand_sides(&parts);
        for corner in 0..4 {
            style.radius[corner] = corners[corner].unwrap_or(0.0);
        }
    }

    fn apply_text(&self, style: &mut Style, name: &str, value: &str) -> bool {
        match name {
            "color" => style.color = self.colour(value),
            "font-size" => style.font_size = self.pixels(value),
            "font-weight" => style.font_weight = font_weight(value),
            "font-family" => style.mono = Some(is_mono(value)),
            "font-style" => style.italic = value == "italic",
            "line-height" => style.line_height = self.line_height(value),
            "text-align" => {
                style.center_text = value == "center";
                style.end_text = matches!(value, "right" | "end");
            }
            "white-space" => style.nowrap = value == "nowrap" || value == "pre",
            "text-overflow" => style.ellipsis = value == "ellipsis",
            "text-transform" => style.uppercase = value == "uppercase",
            "text-decoration" => style.underline = value.contains("underline"),
            _ => return false,
        }
        true
    }

    /// A line height as a multiple of the font size; fixed lengths are divided by it later.
    fn line_height(&self, value: &str) -> Option<f32> {
        if value == "normal" {
            return Some(1.2);
        }
        if let Some(number) = parse_number(value) {
            if !value.ends_with("px") && !value.ends_with("rem") {
                return Some(number);
            }
        }
        match parse_length(value)? {
            Length::Px(pixels) => Some(-pixels),
            Length::Percent(percent) => Some(percent / 100.0),
        }
    }
}

/// The pixels a `transform` moves an element: its translations, added up.
fn parse_translate(value: &str) -> (f32, f32) {
    let mut total = (0.0, 0.0);
    let mut rest = value;
    while let Some(open) = rest.find('(') {
        let name = rest[..open].trim().rsplit(' ').next().unwrap_or("").trim();
        let Some(close) = rest[open..].find(')') else {
            break;
        };
        let arguments: Vec<&str> = rest[open + 1..open + close].split(',').map(str::trim).collect();
        let pixels = |text: &str| match parse_length(text) {
            Some(Length::Px(pixels)) => pixels,
            _ => 0.0,
        };
        match name {
            "translate" | "translate3d" => {
                total.0 += pixels(arguments.first().copied().unwrap_or("0"));
                total.1 += pixels(arguments.get(1).copied().unwrap_or("0"));
            }
            "translateX" => total.0 += pixels(arguments[0]),
            "translateY" => total.1 += pixels(arguments[0]),
            _ => {}
        }
        rest = &rest[open + close + 1..];
    }
    total
}

/// The first time in a list, as milliseconds: `150ms` or `.15s`.
fn first_time(value: &str) -> f32 {
    let first = value.split(',').next().unwrap_or("").trim();
    if let Some(ms) = first.strip_suffix("ms") {
        return ms.trim().parse().unwrap_or(0.0);
    }
    if let Some(seconds) = first.strip_suffix('s') {
        return seconds.trim().parse::<f32>().map(|seconds| seconds * 1000.0).unwrap_or(0.0);
    }
    0.0
}

/// A timing function: a keyword or `cubic-bezier(a, b, c, d)`.
fn parse_easing(value: &str) -> Easing {
    let first = split_top_level(value, ',');
    let text = if value.trim_start().starts_with("cubic-bezier") {
        value.trim()
    } else {
        first[0].trim()
    };
    match text {
        "linear" => return Easing::LINEAR,
        "ease-in" => return Easing(0.42, 0.0, 1.0, 1.0),
        "ease-out" => return Easing(0.0, 0.0, 0.58, 1.0),
        "ease-in-out" => return Easing(0.42, 0.0, 0.58, 1.0),
        _ => {}
    }
    if let Some(inner) = text.strip_prefix("cubic-bezier(").and_then(|rest| rest.strip_suffix(')')) {
        let numbers: Vec<f32> = inner.split(',').filter_map(|part| part.trim().parse().ok()).collect();
        if numbers.len() == 4 {
            return Easing(numbers[0], numbers[1], numbers[2], numbers[3]);
        }
    }
    Easing::EASE
}

fn hover_of(style: &Style) -> HoverStyle {
    HoverStyle {
        background: style.background,
        color: style.color,
        opacity: style.opacity,
        border_color: style.border_color.iter().flatten().next().copied(),
    }
}

/// A control that is not a flex container centres what it holds, as browsers do for a
/// button; one that is a flex container lays its children out from the start, unless the
/// stylesheet says otherwise.
fn centre_control_content(tag: &str, style: &mut Style) {
    if !matches!(tag, "button" | "input" | "select" | "textarea") || style.display == Display::Flex {
        return;
    }
    if style.align_items.is_none() {
        style.align_items = Some(Align::Center);
    }
    if style.justify.is_none() {
        style.justify = Some(Justify::Center);
    }
}

fn default_for_tag(tag: &str) -> Style {
    let mut style = Style::default();
    match tag {
        "span" | "a" | "small" | "code" | "label" | "svg" | "img"
        | "kbd" | "sub" | "sup" | "abbr" | "time" => style.display = Display::Inline,
        "strong" | "b" => {
            style.display = Display::Inline;
            style.font_weight = Some(700.0);
        }
        "em" | "i" | "cite" => {
            style.display = Display::Inline;
            style.italic = true;
        }
        "button" | "input" | "select" | "textarea" => {
            style.display = Display::Inline;
            style.center_text = tag == "button";
        }
        "head" | "script" | "style" | "template" | "title" | "meta" | "link" => {
            style.display = Display::None;
        }
        _ => {}
    }
    style
}

fn align_of(value: &str) -> Option<Align> {
    match value {
        "flex-start" | "start" | "self-start" => Some(Align::Start),
        "flex-end" | "end" | "self-end" => Some(Align::End),
        "center" => Some(Align::Center),
        "stretch" => Some(Align::Stretch),
        "baseline" => Some(Align::Baseline),
        _ => None,
    }
}

fn justify_of(value: &str) -> Option<Justify> {
    match value {
        "flex-start" | "start" | "normal" => Some(Justify::Start),
        "flex-end" | "end" => Some(Justify::End),
        "center" => Some(Justify::Center),
        "space-between" => Some(Justify::Between),
        "space-around" | "space-evenly" => Some(Justify::Around),
        _ => None,
    }
}

fn font_weight(value: &str) -> Option<f32> {
    match value {
        "normal" => Some(400.0),
        "bold" | "bolder" => Some(700.0),
        "lighter" => Some(300.0),
        _ => parse_number(value),
    }
}

fn is_mono(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("mono") || lowered.contains("menlo") || lowered.contains("consolas")
}

/// Splits a shorthand value on spaces outside parentheses.
fn split_values(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = None;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        let separator = character.is_whitespace() && depth == 0;
        match (start, separator) {
            (None, false) => start = Some(index),
            (Some(begin), true) => {
                parts.push(&value[begin..index]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(begin) = start {
        parts.push(&value[begin..]);
    }
    let _ = split_top_level;
    parts
}

/// CSS's one-to-four value shorthand, as four sides.
fn expand_sides<T: Copy>(parts: &[Option<T>]) -> Sides<Option<T>> {
    match parts.len() {
        1 => [parts[0]; 4],
        2 => [parts[0], parts[1], parts[0], parts[1]],
        3 => [parts[0], parts[1], parts[2], parts[1]],
        4 => [parts[0], parts[1], parts[2], parts[3]],
        _ => [None; 4],
    }
}

/// Which corner a `border-*-radius` property names.
fn corner_of(name: &str) -> Option<usize> {
    match name {
        "border-top-left-radius" => Some(0),
        "border-top-right-radius" => Some(1),
        "border-bottom-right-radius" => Some(2),
        "border-bottom-left-radius" => Some(3),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
enum Side {
    All,
    One(usize),
    Inline,
    Block,
}

/// Which side a `padding`, `margin` or `border` property names.
fn side_of(name: &str, base: &str) -> Option<Side> {
    if name == base {
        if base == "border" {
            return None;
        }
        return Some(Side::All);
    }
    let suffix = name.strip_prefix(base)?.strip_prefix('-')?;
    match suffix {
        "top" => Some(Side::One(0)),
        "right" => Some(Side::One(1)),
        "bottom" => Some(Side::One(2)),
        "left" => Some(Side::One(3)),
        "inline-start" if base != "border" => Some(Side::One(3)),
        "inline-end" if base != "border" => Some(Side::One(1)),
        "inline" if base != "border" => Some(Side::Inline),
        "block" | "block-start" | "block-end" if base != "border" => Some(Side::Block),
        _ if base == "border" => {
            let side = suffix.split('-').next()?;
            match side {
                "top" => Some(Side::One(0)),
                "right" => Some(Side::One(1)),
                "bottom" => Some(Side::One(2)),
                "left" => Some(Side::One(3)),
                _ => None,
            }
        }
        _ => None,
    }
}
