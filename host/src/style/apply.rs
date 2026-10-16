//! Putting a parsed style onto GPUI elements and refinements.

use gpui::{BoxShadow, FontWeight, Length, Styled, auto, point, px, relative};

use super::{Align, Dimension, Justify, Style, TextAlign};

fn length(dimension: Dimension) -> Length {
    match dimension {
        Dimension::Px(pixels) => px(pixels).into(),
        Dimension::Percent(percent) => relative(percent / 100.0).into(),
        Dimension::Auto => auto(),
    }
}

/// Puts what a style sets onto anything GPUI can style: an element, or a refinement for
/// hover, press or focus.
pub fn apply<T: Styled>(mut element: T, style: &Style) -> T {
    if style.hidden == Some(true) {
        element = element.hidden();
    }
    match style.row {
        Some(true) => element = element.flex_row(),
        Some(false) => element = element.flex_col(),
        None => {}
    }
    if style.wrap == Some(true) {
        element = element.flex_wrap();
    }
    let refinement = element.style();
    if let Some(grow) = style.grow {
        refinement.flex_grow = Some(grow);
    }
    if let Some(shrink) = style.shrink {
        refinement.flex_shrink = Some(shrink);
    }
    if let Some(basis) = style.basis {
        refinement.flex_basis = Some(length(basis));
    }
    element = match style.align_items {
        Some(Align::Start) => element.items_start(),
        Some(Align::End) => element.items_end(),
        Some(Align::Center) => element.items_center(),
        Some(Align::Baseline) => element.items_baseline(),
        Some(Align::Stretch) => {
            element.style().align_items = Some(gpui::AlignItems::Stretch);
            element
        }
        None => element,
    };
    if let Some(alignment) = style.align_self {
        element.style().align_self = Some(match alignment {
            Align::Start => gpui::AlignSelf::FlexStart,
            Align::End => gpui::AlignSelf::FlexEnd,
            Align::Center => gpui::AlignSelf::Center,
            Align::Stretch => gpui::AlignSelf::Stretch,
            Align::Baseline => gpui::AlignSelf::Baseline,
        });
    }
    element = match style.justify {
        Some(Justify::Start) => element.justify_start(),
        Some(Justify::End) => element.justify_end(),
        Some(Justify::Center) => element.justify_center(),
        Some(Justify::Between) => element.justify_between(),
        Some(Justify::Around) => element.justify_around(),
        None => element,
    };
    if let Some(gap) = style.row_gap {
        element = element.gap_y(px(gap));
    }
    if let Some(gap) = style.column_gap {
        element = element.gap_x(px(gap));
    }
    let [top, right, bottom, left] = style.padding;
    let refinement = element.style();
    if let Some(value) = top {
        refinement.padding.top = Some(px(value).into());
    }
    if let Some(value) = right {
        refinement.padding.right = Some(px(value).into());
    }
    if let Some(value) = bottom {
        refinement.padding.bottom = Some(px(value).into());
    }
    if let Some(value) = left {
        refinement.padding.left = Some(px(value).into());
    }
    let [top, right, bottom, left] = style.margin;
    if let Some(value) = top {
        refinement.margin.top = Some(length(value));
    }
    if let Some(value) = right {
        refinement.margin.right = Some(length(value));
    }
    if let Some(value) = bottom {
        refinement.margin.bottom = Some(length(value));
    }
    if let Some(value) = left {
        refinement.margin.left = Some(length(value));
    }
    if let Some(value) = style.width {
        refinement.size.width = Some(length(value));
    }
    if let Some(value) = style.height {
        refinement.size.height = Some(length(value));
    }
    if let Some(value) = style.min_width {
        refinement.min_size.width = Some(length(value));
    }
    if let Some(value) = style.min_height {
        refinement.min_size.height = Some(length(value));
    }
    if let Some(value) = style.max_width {
        refinement.max_size.width = Some(length(value));
    }
    if let Some(value) = style.max_height {
        refinement.max_size.height = Some(length(value));
    }
    if let Some(ratio) = style.aspect_ratio {
        refinement.aspect_ratio = Some(ratio);
    }
    let (shift_x, shift_y) = style.translate.unwrap_or((0.0, 0.0));
    let absolute = style.absolute == Some(true);
    if absolute {
        element = element.absolute();
    } else if style.absolute == Some(false) || shift_x != 0.0 || shift_y != 0.0 {
        element = element.relative();
    }
    // A translation moves the box from where it was laid out, which an offset does for a
    // relative box and adds to the insets of an absolute one.
    let shifted = |value: Option<Dimension>, shift: f32| match value {
        Some(Dimension::Px(pixels)) => Some(Dimension::Px(pixels + shift)),
        None if shift != 0.0 => Some(Dimension::Px(shift)),
        other => other,
    };
    let [top, right, bottom, left] = style.inset;
    let refinement = element.style();
    if let Some(value) = shifted(top, shift_y) {
        refinement.inset.top = Some(length(value));
    }
    if let Some(value) = right {
        refinement.inset.right = Some(length(value));
    }
    if let Some(value) = bottom {
        refinement.inset.bottom = Some(length(value));
    }
    if let Some(value) = shifted(left, shift_x) {
        refinement.inset.left = Some(length(value));
    }
    if style.clip == Some(true) {
        element = element.overflow_hidden();
    }
    if let Some(colour) = style.background {
        element = element.bg(colour);
    }
    if let Some(opacity) = style.opacity {
        element = element.opacity(opacity);
    }
    let [top, right, bottom, left] = style.border_width;
    let refinement = element.style();
    if let Some(width) = top {
        refinement.border_widths.top = Some(px(width).into());
    }
    if let Some(width) = right {
        refinement.border_widths.right = Some(px(width).into());
    }
    if let Some(width) = bottom {
        refinement.border_widths.bottom = Some(px(width).into());
    }
    if let Some(width) = left {
        refinement.border_widths.left = Some(px(width).into());
    }
    if let Some(colour) = style.border_color {
        element = element.border_color(colour);
    }
    let [top_left, top_right, bottom_right, bottom_left] = style.radius;
    let refinement = element.style();
    if let Some(radius) = top_left {
        refinement.corner_radii.top_left = Some(px(radius).into());
    }
    if let Some(radius) = top_right {
        refinement.corner_radii.top_right = Some(px(radius).into());
    }
    if let Some(radius) = bottom_right {
        refinement.corner_radii.bottom_right = Some(px(radius).into());
    }
    if let Some(radius) = bottom_left {
        refinement.corner_radii.bottom_left = Some(px(radius).into());
    }
    if let Some(shadows) = &style.shadow {
        refinement.box_shadow = Some(
            shadows
                .iter()
                .map(|shadow| BoxShadow {
                    color: shadow.color.into(),
                    offset: point(px(shadow.x), px(shadow.y)),
                    blur_radius: px(shadow.blur),
                    spread_radius: px(shadow.spread),
                })
                .collect(),
        );
    }
    if let Some(cursor) = style.cursor {
        refinement.mouse_cursor = Some(cursor);
    }
    apply_text(element, style)
}

/// The text properties, which children inherit.
fn apply_text<T: Styled>(mut element: T, style: &Style) -> T {
    if let Some(colour) = style.color {
        element = element.text_color(colour);
    }
    if let Some(size) = style.font_size {
        element = element.text_size(px(size));
    }
    if let Some(weight) = style.font_weight {
        element = element.font_weight(FontWeight(weight));
    }
    if let Some(family) = &style.font_family {
        element = element.font_family(family.clone());
    }
    if let Some(height) = style.line_height {
        element = element.line_height(px(height));
    }
    if style.italic == Some(true) {
        element = element.italic();
    }
    if style.underline == Some(true) {
        element = element.underline();
    }
    if style.strikethrough == Some(true) {
        element = element.line_through();
    }
    element = match style.text_align {
        Some(TextAlign::Left) => element.text_left(),
        Some(TextAlign::Center) => element.text_center(),
        Some(TextAlign::Right) => element.text_right(),
        None => element,
    };
    if style.nowrap == Some(true) {
        element = element.whitespace_nowrap();
    }
    if style.truncate == Some(true) {
        element = element.truncate();
    }
    if let Some(lines) = style.max_lines {
        element = element.line_clamp(lines);
    }
    element
}
