//! A text field: its text, selection and blinking caret.

use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    Bounds, Context, Div, HighlightStyle, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Rgba,
    SharedString, StyledText, canvas, div, fill, px, size,
};

use super::{Inherited, Painter, attribute};
use crate::field::Field;
use crate::style::Style;
use crate::tree::{Node, NodeId};
use crate::workbench::Workbench;

const SELECTION: Rgba = Rgba { r: 0.23, g: 0.43, b: 0.66, a: 0.45 };

impl Painter<'_> {
    /// The editable text of a `g-input`, with its selection and caret.
    pub(super) fn field_content(
        &self,
        element: Div,
        id: NodeId,
        node: &Node,
        style: &Style,
        inherited: &Inherited,
        cx: &mut Context<Workbench>,
    ) -> Div {
        let multiline = node.attributes.contains_key("multiline");
        let secure = node.attributes.contains_key("secure");
        let placeholder = attribute(node, "placeholder").to_string();
        let handle = self.interaction.focus_handle(id, cx);
        let initial = attribute(node, "value").to_string();
        let font_size = inherited.font_size;
        let line_height = style.line_height.unwrap_or(font_size * 1.4);

        let (shown, cursor, selection, scroll, composing) = {
            let mut fields = self.interaction.fields.borrow_mut();
            let field = fields.entry(id).or_insert_with(Field::new);
            if field.applied.as_deref() != Some(initial.as_str()) {
                field.text = initial.clone();
                field.cursor = field.text.len();
                field.anchor = field.cursor;
                field.applied = Some(initial);
            }
            field.cursor = field.cursor.min(field.text.len());
            field.anchor = field.anchor.min(field.text.len());
            let shown = if secure { "\u{2022}".repeat(field.text.chars().count()) } else { field.text.clone() };
            let map = |byte: usize| -> usize {
                if secure { field.text[..byte].chars().count() * "\u{2022}".len() } else { byte }
            };
            let selection = field.selection();
            let composing = field.marked.clone().map(|range| map(range.start)..map(range.end));
            (shown, map(field.cursor), map(selection.start)..map(selection.end), field.scroll_x, composing)
        };

        let empty = shown.is_empty();
        let mut styled = StyledText::new(SharedString::from(if empty { placeholder } else { shown }));
        if let (Some(range), false) = (composing, empty) {
            let underline =
                gpui::UnderlineStyle { thickness: px(1.0), color: Some(inherited.colour.into()), wavy: false };
            styled = styled
                .with_highlights(vec![(range, HighlightStyle { underline: Some(underline), ..Default::default() })]);
        } else if !empty && !selection.is_empty() {
            let highlight = HighlightStyle { background_color: Some(Hsla::from(SELECTION)), ..Default::default() };
            styled = styled.with_highlights(vec![(selection, highlight)]);
        }
        if !empty {
            let layout = styled.layout().clone();
            if let Some(field) = self.interaction.fields.borrow_mut().get_mut(&id) {
                field.layout = Some(layout);
            }
        }

        let mut text = div();
        if !multiline {
            text = text.whitespace_nowrap().ml(px(-scroll));
        }
        if empty {
            text = text.text_color(Rgba { a: inherited.colour.a * 0.45, ..inherited.colour });
        }
        let text = text.child(styled);

        let fields = self.interaction.fields.clone();
        let caret_colour = inherited.colour;
        let caret_on = caret_visible(self.interaction.caret_epoch.get());
        let focus = handle.clone();
        let workbench = cx.entity();
        let caret = canvas(
            |bounds, _, _| bounds,
            move |bounds: Bounds<gpui::Pixels>, _, window, cx| {
                // The input method types into whatever is focused, through this handler.
                window.handle_input(&focus, gpui::ElementInputHandler::new(bounds, workbench.clone()), cx);
                if !focus.is_focused(window) {
                    return;
                }
                let mut fields = fields.borrow_mut();
                let Some(field) = fields.get_mut(&id) else {
                    return;
                };
                let measured = match (&field.layout, empty) {
                    (Some(layout), false) => {
                        layout.position_for_index(cursor).map(|position| (position, layout.line_height()))
                    }
                    _ => None,
                };
                let (position, height) = measured.unwrap_or((bounds.origin, px(line_height)));
                if !multiline {
                    let relative_x = f32::from(position.x - bounds.origin.x);
                    let width = f32::from(bounds.size.width);
                    let target = if relative_x > width - 2.0 {
                        field.scroll_x + relative_x - (width - 2.0)
                    } else if relative_x < 0.0 {
                        (field.scroll_x + relative_x).max(0.0)
                    } else {
                        field.scroll_x
                    };
                    if (target - field.scroll_x).abs() > 0.5 {
                        field.scroll_x = target;
                        window.request_animation_frame();
                    }
                }
                if caret_on {
                    window.paint_quad(fill(Bounds { origin: position, size: size(px(1.5), height) }, caret_colour));
                }
            },
        )
        // Pinned to the corner: an absolute box without insets sits where it would have
        // flowed, below the text, where a single-line field clips it.
        .absolute()
        .top_0()
        .left_0()
        .size_full();

        let mut container = div().relative().flex_1().min_w_0().child(text).child(caret);
        if !multiline {
            container = container.overflow_hidden();
        } else {
            let rows: f32 = attribute(node, "rows").parse().unwrap_or(3.0);
            container = container.min_h(px(rows * line_height));
        }
        element.child(container)
    }

    /// Mouse handling for a text field: place the cursor, and drag to select.
    pub(super) fn wire_field(
        &self,
        element: gpui::Stateful<Div>,
        id: NodeId,
        cx: &mut Context<Workbench>,
    ) -> gpui::Stateful<Div> {
        element
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |workbench, event: &MouseDownEvent, window, cx| {
                    workbench.focus_node(id, window);
                    workbench.field_mouse_down(id, event.position, event.modifiers.shift);
                    workbench.caret_moved(id, cx);
                }),
            )
            .on_mouse_move(cx.listener(move |workbench, event: &MouseMoveEvent, _window, cx| {
                let dragging = workbench.interaction.fields.borrow().get(&id).map(|field| field.dragging);
                if dragging == Some(true) {
                    workbench.field_mouse_move(id, event.position);
                    workbench.caret_moved(id, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |workbench, _event: &MouseUpEvent, _window, _cx| {
                    workbench.field_mouse_up(id);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(move |workbench, _event: &MouseUpEvent, _window, _cx| {
                    workbench.field_mouse_up(id);
                }),
            )
    }
}

/// How long the caret stays on, and then off.
const BLINK: Duration = Duration::from_millis(530);

/// Whether a caret that last moved at `epoch` shows now: solid for a moment after it
/// moves, then blinking.
pub fn caret_visible(epoch: Instant) -> bool {
    let elapsed = epoch.elapsed();
    elapsed < BLINK || (elapsed.as_millis() / BLINK.as_millis()) % 2 == 0
}

/// When a caret that last moved at `epoch` next turns on or off.
pub fn next_blink(epoch: Instant) -> Instant {
    let periods = epoch.elapsed().as_millis() / BLINK.as_millis() + 1;
    epoch + BLINK * periods as u32
}
