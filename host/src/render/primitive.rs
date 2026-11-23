//! The primitives an app builds from, and the style each one starts from.

use std::rc::Rc;

use crate::style::{self, Style};
use crate::tree::{Kind, Node};

/// The primitives, by the tag their component renders.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Primitive {
    View,
    Text,
    Button,
    Input,
    Scroll,
    Image,
    Icon,
    Canvas,
    Overlay,
    /// A view the app's own crate draws (`Elements`).
    Element,
}

impl Primitive {
    pub fn of(tag: &str) -> Option<Primitive> {
        Some(match tag {
            "g-view" | "#root" => Primitive::View,
            "g-text" => Primitive::Text,
            "g-button" => Primitive::Button,
            "g-input" => Primitive::Input,
            "g-scroll" => Primitive::Scroll,
            "g-image" => Primitive::Image,
            "g-icon" => Primitive::Icon,
            "g-canvas" => Primitive::Canvas,
            "g-overlay" => Primitive::Overlay,
            "g-element" => Primitive::Element,
            _ => return None,
        })
    }

    /// What a primitive looks like before the app's style: views stack their children,
    /// as in React Native; buttons and inputs lay theirs out in a row.
    fn defaults(self) -> Style {
        let mut style = Style::default();
        match self {
            Primitive::Button => {
                style.row = Some(true);
                style.align_items = Some(style::Align::Center);
                style.cursor = Some(gpui::CursorStyle::PointingHand);
            }
            Primitive::Input => {
                style.row = Some(true);
                style.cursor = Some(gpui::CursorStyle::IBeam);
            }
            Primitive::Image | Primitive::Icon | Primitive::Canvas => {
                style.row = Some(false);
                style.shrink = Some(0.0);
            }
            _ => style.row = Some(false),
        }
        style
    }

    pub(super) fn focusable(self) -> bool {
        matches!(self, Primitive::Button | Primitive::Input)
    }
}

pub fn primitive_of(node: &Node) -> Option<Primitive> {
    match &node.kind {
        Kind::Element { tag } => Primitive::of(tag),
        Kind::Text { .. } => None,
    }
}

/// Everything a node needs to be drawn: its style, and the patches for its states.
#[derive(Clone)]
pub struct Computed {
    pub style: Style,
    pub hover: Option<Rc<Style>>,
    pub active: Option<Rc<Style>>,
    pub focus: Option<Rc<Style>>,
    /// The product of this element's `zoom` and its ancestors', which its lengths are already
    /// multiplied by.
    pub zoom: f32,
}

impl Computed {
    /// The style as it is laid out under `zoom`, the zoom of everything above it and its own.
    pub(super) fn zoomed(self, zoom: f32) -> Computed {
        if zoom == 1.0 {
            return Computed { zoom, ..self };
        }
        let patch = |style: Option<Rc<Style>>| style.map(|style| Rc::new(style.zoomed(zoom)));
        Computed {
            style: self.style.zoomed(zoom),
            hover: patch(self.hover),
            active: patch(self.active),
            focus: patch(self.focus),
            zoom,
        }
    }
}

pub(super) fn compute(node: &Node) -> Computed {
    let primitive = primitive_of(node).unwrap_or(Primitive::View);
    let mut style = primitive.defaults();
    if primitive == Primitive::Text && node.attributes.contains_key("selectable") {
        style.cursor = Some(gpui::CursorStyle::IBeam);
    }
    let patch = |name: &str| node.attributes.get(name).map(|json| style::parsed(json));
    if let Some(own) = patch("look") {
        style.overlay(&own);
    }
    for (name, value) in &node.style {
        style.apply_inline(name, value);
    }
    Computed { style, hover: patch("hover"), active: patch("active"), focus: patch("focus"), zoom: 1.0 }
}

#[cfg(test)]
mod tests {
    use gpui::CursorStyle;

    use super::compute;
    use crate::tree::{CREATE_ELEMENT, INSERT, ROOT, SET_ATTRIBUTE, Tree};

    /// The cursor of a `g-text` made with these attributes.
    fn cursor_of(attributes: &[(&str, &str)]) -> Option<CursorStyle> {
        let mut tree = Tree::new();
        let mut batch = vec![format!("[{CREATE_ELEMENT},2,\"g-text\"]"), format!("[{INSERT},{ROOT},2,0]")];
        for (name, value) in attributes {
            batch.push(format!("[{SET_ATTRIBUTE},2,{name:?},{value:?}]"));
        }
        tree.apply(&format!("[{}]", batch.join(",")));
        compute(&tree.nodes[&2]).style.cursor
    }

    #[test]
    fn selectable_text_has_the_text_cursor_unless_its_style_says_otherwise() {
        assert_eq!(cursor_of(&[]), None);
        assert_eq!(cursor_of(&[("selectable", "")]), Some(CursorStyle::IBeam));
        let pointer = r#"{"cursor":"pointer"}"#;
        assert_eq!(cursor_of(&[("selectable", ""), ("look", pointer)]), Some(CursorStyle::PointingHand));
    }
}
