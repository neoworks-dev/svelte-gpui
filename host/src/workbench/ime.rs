//! What a platform input method sees of the focused field, in UTF-16 offsets.

use std::ops::Range;

use gpui::{Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window, point, px, size};
use serde_json::json;

use super::Workbench;
use crate::field::{byte_offset, utf16_offset};
use crate::tree::NodeId;

/// An input method's view of the focused field: its text, selection and preedit, in
/// UTF-16 offsets, which is what platform input methods count in.
impl EntityInputHandler for Workbench {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let id = self.input_target(window)?;
        let fields = self.interaction.fields.borrow();
        let field = fields.get(&id)?;
        let start = byte_offset(&field.text, range.start);
        let end = byte_offset(&field.text, range.end);
        *adjusted = Some(utf16_offset(&field.text, start)..utf16_offset(&field.text, end));
        Some(field.text[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let id = self.input_target(window)?;
        let fields = self.interaction.fields.borrow();
        let field = fields.get(&id)?;
        let range = field.selection();
        Some(UTF16Selection {
            range: utf16_offset(&field.text, range.start)..utf16_offset(&field.text, range.end),
            reversed: field.cursor < field.anchor,
        })
    }

    fn marked_text_range(&self, window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        let id = self.input_target(window)?;
        let fields = self.interaction.fields.borrow();
        let field = fields.get(&id)?;
        let marked = field.marked.clone()?;
        Some(utf16_offset(&field.text, marked.start)..utf16_offset(&field.text, marked.end))
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.input_target(window) else {
            return;
        };
        let data = {
            let mut fields = self.interaction.fields.borrow_mut();
            let Some(field) = fields.get_mut(&id) else {
                return;
            };
            match field.marked.take() {
                Some(range) => field.text[range].to_string(),
                None => return,
            }
        };
        self.emit(id, "compositionend", json!({ "data": data }).to_string(), cx);
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self.input_target(window) else {
            return;
        };
        if !self.editable(id) {
            return;
        }
        let bytes = self.bytes_for(id, range);
        self.compose(id, bytes, text, false, None, "insertText", cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self.input_target(window) else {
            return;
        };
        if !self.editable(id) {
            return;
        }
        let bytes = self.bytes_for(id, range);
        let inside = new_selected_range
            .map(|selected| byte_offset(new_text, selected.start)..byte_offset(new_text, selected.end));
        self.compose(id, bytes, new_text, true, inside, "insertCompositionText", cx);
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        element_bounds: Bounds<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let id = self.input_target(window)?;
        let fields = self.interaction.fields.borrow();
        let field = fields.get(&id)?;
        let start = byte_offset(&field.text, range.start);
        let anchor = field
            .layout
            .as_ref()
            .and_then(|layout| layout.position_for_index(start).map(|at| (at, layout.line_height())));
        match anchor {
            Some((at, height)) => Some(Bounds { origin: at, size: size(px(1.0), height) }),
            None => Some(Bounds {
                origin: point(element_bounds.origin.x, element_bounds.origin.y),
                size: size(px(1.0), element_bounds.size.height),
            }),
        }
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let id = self.input_target(window)?;
        let fields = self.interaction.fields.borrow();
        let field = fields.get(&id)?;
        let layout = field.layout.as_ref()?;
        let index = match layout.index_for_position(position) {
            Ok(index) | Err(index) => index.min(field.text.len()),
        };
        Some(utf16_offset(&field.text, index))
    }
}

impl Workbench {
    /// An input method's UTF-16 range, as byte offsets in the focused field.
    fn bytes_for(&self, id: NodeId, range: Option<Range<usize>>) -> Option<Range<usize>> {
        let range = range?;
        let fields = self.interaction.fields.borrow();
        let field = fields.get(&id)?;
        Some(byte_offset(&field.text, range.start)..byte_offset(&field.text, range.end))
    }
}
