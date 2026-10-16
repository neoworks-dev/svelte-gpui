//! Canvases, images and icons.

use gpui::prelude::*;
use gpui::{Div, ObjectFit, SharedString, StyledImage, canvas, img, px, svg};

use super::{Inherited, Painter, attribute, is_far_away};
use crate::assets::register_svg;
use crate::images::ImageState;
use crate::style::Style;
use crate::tree::{Node, NodeId};

impl Painter<'_> {
    /// The canvas's pixels, sized to its box at the window's scale and drawn as an image.
    pub(super) fn canvas_content(&self, element: Div, id: NodeId, style: &Style) -> Div {
        let Some(surface) = self.tree.canvases.get(&id).cloned() else {
            return element;
        };
        let radii = style.radius.map(|radius| px(radius.unwrap_or(0.0)));
        let layer = canvas(
            |bounds, _, _| bounds,
            move |bounds, _, window, _cx| {
                let scale = window.scale_factor();
                let mut surface = surface.borrow_mut();
                let width = (f32::from(bounds.size.width) * scale).round() as u32;
                let height = (f32::from(bounds.size.height) * scale).round() as u32;
                surface.resize(width, height, scale);
                for old in std::mem::take(&mut surface.stale) {
                    window.drop_image(old).ok();
                }
                if let Some(image) = surface.image() {
                    let corners = gpui::Corners {
                        top_left: radii[0],
                        top_right: radii[1],
                        bottom_right: radii[2],
                        bottom_left: radii[3],
                    };
                    window.paint_image(bounds, corners, image, 0, false).ok();
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        element.child(layer)
    }

    /// An image draws its file once the source has resolved to one, and only while it is
    /// near the window, decoded no larger than it is shown.
    pub(super) fn image_content(&self, mut element: Div, id: NodeId, node: &Node, style: &Style) -> Div {
        let source = attribute(node, "src");
        if source.is_empty() {
            return element;
        }
        let images = &self.interaction.images;
        let ImageState::Ready(path) = images.state(source) else {
            return element;
        };
        element = element.child(self.measure(id));

        // Until it is drawn the box takes the image's own proportions, so the page does not jump.
        if let (Some((natural_width, natural_height)), None) = (images.dimensions(&path), style.width) {
            if style.height.is_none() {
                element = element.w(px(natural_width as f32)).max_w_full();
            }
            if style.aspect_ratio.is_none() {
                element.style().aspect_ratio = Some(natural_width as f32 / natural_height.max(1) as f32);
            }
        }

        let bounds = self.interaction.bounds.borrow().get(&id).copied();
        let view = self.interaction.viewport.get();
        let Some(placed) = bounds.filter(|placed| !is_far_away(*placed, view)) else {
            return element;
        };
        let wanted = f32::from(placed.size.width) * self.interaction.scale.get();
        let bucket = ((wanted.ceil() as u32).max(1)).div_ceil(64) * 64;
        let ImageState::Ready(file) = images.scaled(&path, bucket) else {
            return element;
        };
        let fit = match attribute(node, "fit") {
            "cover" => ObjectFit::Cover,
            "fill" => ObjectFit::Fill,
            _ => ObjectFit::Contain,
        };
        let mut picture = img(file).size_full().object_fit(fit);
        let [top_left, top_right, bottom_right, bottom_left] = style.radius;
        let corners = &mut picture.style().corner_radii;
        corners.top_left = top_left.map(|radius| px(radius).into());
        corners.top_right = top_right.map(|radius| px(radius).into());
        corners.bottom_right = bottom_right.map(|radius| px(radius).into());
        corners.bottom_left = bottom_left.map(|radius| px(radius).into());
        element.child(picture)
    }
}

/// An icon: SVG markup the app passed, or a file, tinted with the text colour.
pub(super) fn icon_content(mut element: Div, node: &Node, style: &Style, inherited: &Inherited) -> Div {
    let path = match (node.attributes.get("svg"), node.attributes.get("src")) {
        (Some(markup), _) => register_svg(markup.clone()),
        (None, Some(source)) => source.clone(),
        (None, None) => return element,
    };
    let side = inherited.font_size;
    if style.width.is_none() {
        element = element.w(px(side));
    }
    if style.height.is_none() {
        element = element.h(px(side));
    }
    element.child(svg().path(SharedString::from(path)).size_full().text_color(inherited.colour))
}
