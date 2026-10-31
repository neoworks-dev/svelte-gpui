//! Editing for `g-input`. A key changes the field's text and tells the app with an
//! `edit` event; the app writing `value` replaces the text.

use std::ops::Range;

use gpui::{ClipboardEntry, ClipboardItem, Context, Keystroke, Pixels, Point, Window};
use serde_json::json;

use super::Workbench;
use crate::clipboard::save_image;
use crate::field::{Field, byte_offset, utf16_offset};
use crate::tree::{Node, NodeId};

fn attribute<'a>(node: &'a Node, name: &str) -> &'a str {
    match node.attributes.get(name) {
        Some(value) => value.as_str(),
        None => "",
    }
}

pub fn is_multiline(node: &Node) -> bool {
    node.attributes.contains_key("multiline")
}

fn is_true(node: &Node, name: &str) -> bool {
    match node.attributes.get(name) {
        Some(value) => value != "false",
        None => false,
    }
}

/// What may be typed into a field of this kind.
fn accepts(node: &Node, text: &str) -> bool {
    if !node.attributes.contains_key("numeric") {
        return true;
    }
    text.chars().all(|character| character.is_ascii_digit() || matches!(character, '.' | '-' | '+' | 'e'))
}

impl Workbench {
    /// The field behind a node, once it has been drawn.
    fn has_field(&self, id: NodeId) -> bool {
        self.interaction.fields.borrow().contains_key(&id)
    }

    pub(super) fn editable(&self, id: NodeId) -> bool {
        let tree = self.svelte.tree.borrow();
        match tree.nodes.get(&id) {
            Some(node) => !is_true(node, "disabled") && !is_true(node, "readonly"),
            None => false,
        }
    }

    /// Writes the field's text back as the node's `value`, and tells the app what happened.
    pub fn after_edit(&mut self, id: NodeId, input_type: &str, data: Option<&str>, cx: &mut Context<Self>) {
        let (text, start, end) = {
            let mut fields = self.interaction.fields.borrow_mut();
            let Some(field) = fields.get_mut(&id) else {
                return;
            };
            field.applied = Some(field.text.clone());
            let range = field.selection();
            (field.text.clone(), utf16_offset(&field.text, range.start), utf16_offset(&field.text, range.end))
        };
        if let Some(node) = self.svelte.tree.borrow_mut().nodes.get_mut(&id) {
            node.attributes.insert("value".to_string(), text.clone());
        }
        let detail = json!({
            "value": text,
            "inputType": input_type,
            "data": data,
            "selectionStart": start,
            "selectionEnd": end,
        });
        self.emit(id, "edit", detail.to_string(), cx);
        // After `edit`, so the app has the new text when it hears where the cursor went.
        self.caret_moved(id, cx);
    }

    /// Tells the app where a field's cursor and selection are, if it listens and they moved.
    pub fn report_selection(&mut self, id: NodeId, cx: &mut Context<Self>) {
        let listens = self.svelte.tree.borrow().nodes.get(&id).is_some_and(|node| node.listeners.contains("select"));
        if !listens {
            return;
        }
        let now = {
            let mut fields = self.interaction.fields.borrow_mut();
            let Some(field) = fields.get_mut(&id) else {
                return;
            };
            let now = field.selection_utf16();
            if field.reported == Some(now) {
                return;
            }
            field.reported = Some(now);
            now
        };
        self.emit(id, "select", json!({ "start": now.0, "end": now.1 }).to_string(), cx);
    }

    /// Fires `commit` when the text differs from what it was on focus.
    pub fn commit_change(&mut self, id: NodeId, cx: &mut Context<Self>) {
        let value = {
            let mut fields = self.interaction.fields.borrow_mut();
            let Some(field) = fields.get_mut(&id) else {
                return;
            };
            if field.text == field.focus_value {
                return;
            }
            field.focus_value = field.text.clone();
            field.text.clone()
        };
        self.emit(id, "commit", json!({ "value": value }).to_string(), cx);
    }

    /// Remembers the text a field had on focus, for `commit`.
    pub fn field_focused(&mut self, id: NodeId) {
        if let Some(field) = self.interaction.fields.borrow_mut().get_mut(&id) {
            field.focus_value = field.text.clone();
        }
    }

    /// Applies a keystroke to a focused field. Returns whether the field used it.
    ///
    /// Typed characters normally reach a field through the input method, which is how
    /// accents and composed text arrive; `insert_text` makes the field take them from the
    /// key instead, for scripted typing.
    pub fn field_key(&mut self, id: NodeId, keystroke: &Keystroke, insert_text: bool, cx: &mut Context<Self>) -> bool {
        if !self.has_field(id) {
            return false;
        }
        let command = keystroke.modifiers.control || keystroke.modifiers.platform;
        let extend = keystroke.modifiers.shift;
        let multiline = self.is_multiline(id);
        let editable = self.editable(id);

        match keystroke.key.as_str() {
            "a" if command => {
                self.edit_field(id, |field| {
                    field.anchor = 0;
                    field.cursor = field.text.len();
                });
                return true;
            }
            "left" | "right" | "home" | "end" => {
                self.move_cursor(id, keystroke, extend);
                return true;
            }
            "up" | "down" if multiline => {
                let direction = if keystroke.key == "up" { -1.0 } else { 1.0 };
                self.edit_field(id, |field| {
                    if let Some(index) = field.vertical(direction) {
                        field.move_to(index, extend);
                    }
                });
                return true;
            }
            _ => {}
        }
        if !editable {
            return false;
        }
        match keystroke.key.as_str() {
            "backspace" => self.delete(id, true, command, cx),
            "delete" => self.delete(id, false, command, cx),
            "enter" if multiline => self.type_text(id, "\n", "insertLineBreak", cx),
            "enter" => {
                let value = self.interaction.fields.borrow().get(&id).map(|field| field.text.clone());
                self.commit_change(id, cx);
                if let Some(value) = value {
                    self.emit(id, "submit", json!({ "value": value }).to_string(), cx);
                }
            }
            "tab" => return true,
            _ => {
                let Some(text) = &keystroke.key_char else {
                    return false;
                };
                if command || keystroke.modifiers.alt || !insert_text {
                    return false;
                }
                self.type_text(id, text, "insertText", cx);
            }
        }
        true
    }

    fn is_multiline(&self, id: NodeId) -> bool {
        let tree = self.svelte.tree.borrow();
        tree.nodes.get(&id).is_some_and(is_multiline)
    }

    fn edit_field(&mut self, id: NodeId, change: impl FnOnce(&mut Field)) {
        if let Some(field) = self.interaction.fields.borrow_mut().get_mut(&id) {
            change(field);
        }
    }

    fn move_cursor(&mut self, id: NodeId, keystroke: &Keystroke, extend: bool) {
        let word = keystroke.modifiers.control || keystroke.modifiers.alt;
        let key = keystroke.key.clone();
        let multiline = self.is_multiline(id);
        self.edit_field(id, |field| {
            let collapsed = !field.has_selection() || extend;
            let target = match key.as_str() {
                "left" if !collapsed => field.selection().start,
                "right" if !collapsed => field.selection().end,
                "left" if word => field.previous_word(field.cursor),
                "right" if word => field.next_word(field.cursor),
                "left" => field.previous_boundary(field.cursor),
                "right" => field.next_boundary(field.cursor),
                "home" if multiline && !word => field.line_start(field.cursor),
                "end" if multiline && !word => field.line_end(field.cursor),
                "home" => 0,
                _ => field.text.len(),
            };
            field.move_to(target, extend);
        });
    }

    fn type_text(&mut self, id: NodeId, text: &str, input_type: &str, cx: &mut Context<Self>) {
        self.compose(id, None, text, false, None, input_type, cx);
    }

    /// Puts text into a field: over `range`, else over what an input method is composing,
    /// else over the selection. With `mark` the text stays provisional, drawn underlined,
    /// until it is committed or replaced; `selection` places the cursor inside it.
    pub fn compose(
        &mut self,
        id: NodeId,
        range: Option<Range<usize>>,
        text: &str,
        mark: bool,
        selection: Option<Range<usize>>,
        input_type: &str,
        cx: &mut Context<Self>,
    ) {
        let (accepted, was_composing) = {
            let tree = self.svelte.tree.borrow();
            let Some(node) = tree.nodes.get(&id) else {
                return;
            };
            let mut fields = self.interaction.fields.borrow_mut();
            let Some(field) = fields.get_mut(&id) else {
                return;
            };
            let multiline = is_multiline(node);
            let was_composing = field.marked.is_some();
            let mut text = text.to_string();
            if !multiline {
                if !mark && text.trim_matches(['\n', '\r', '\t']).is_empty() && !text.is_empty() {
                    return;
                }
                text = text.replace(['\n', '\r'], " ");
            }
            let target = match range.or_else(|| field.marked.clone()) {
                Some(range) => range.start.min(field.text.len())..range.end.min(field.text.len()),
                None => field.selection(),
            };
            if !mark {
                let limit: usize = attribute(node, "maxlength").parse().unwrap_or(usize::MAX);
                let kept = field.text.chars().count() - field.text[target.clone()].chars().count();
                text = text.chars().take(limit.saturating_sub(kept)).collect();
                if (text.is_empty() && !was_composing) || !accepts(node, &text) {
                    if text.is_empty() && !was_composing {
                        return;
                    }
                }
            }
            field.text.replace_range(target.clone(), &text);
            let end = target.start + text.len();
            field.marked = if mark { Some(target.start..end) } else { None };
            match (&selection, mark) {
                (Some(inside), true) => {
                    field.anchor = target.start + byte_offset(&text, inside.start);
                    field.cursor = target.start + byte_offset(&text, inside.end);
                }
                _ => {
                    field.cursor = end;
                    field.anchor = end;
                }
            }
            (text, was_composing)
        };
        if mark && !was_composing {
            self.emit(id, "compositionstart", json!({ "data": "" }).to_string(), cx);
        }
        if mark {
            self.emit(id, "compositionupdate", json!({ "data": accepted }).to_string(), cx);
        }
        let kind = if mark || was_composing { "insertCompositionText" } else { input_type };
        self.after_edit(id, kind, Some(&accepted), cx);
        if !mark && was_composing {
            self.emit(id, "compositionend", json!({ "data": accepted }).to_string(), cx);
        }
    }

    /// The field an input method is typing into: the focused one.
    pub(super) fn input_target(&self, window: &Window) -> Option<NodeId> {
        let id = self.focused_node(window)?;
        if !self.has_field(id) {
            return None;
        }
        Some(id)
    }

    fn delete(&mut self, id: NodeId, backwards: bool, word: bool, cx: &mut Context<Self>) {
        let changed = {
            let mut fields = self.interaction.fields.borrow_mut();
            let Some(field) = fields.get_mut(&id) else {
                return;
            };
            if !field.has_selection() {
                let edge = match (backwards, word) {
                    (true, true) => field.previous_word(field.cursor),
                    (true, false) => field.previous_boundary(field.cursor),
                    (false, true) => field.next_word(field.cursor),
                    (false, false) => field.next_boundary(field.cursor),
                };
                field.anchor = edge;
            }
            let any = field.has_selection();
            if any {
                field.insert("");
            }
            any
        };
        if changed {
            let kind = if backwards { "deleteContentBackward" } else { "deleteContentForward" };
            self.after_edit(id, kind, None, cx);
        }
    }

    /// Puts text into the focused field, as a paste does.
    pub fn paste_into(&mut self, id: NodeId, text: &str, cx: &mut Context<Self>) {
        if self.has_field(id) && self.editable(id) {
            self.type_text(id, text, "insertFromPaste", cx);
        }
    }

    /// The selected text of a field, if there is any.
    pub fn selected_text(&self, id: NodeId) -> Option<String> {
        let fields = self.interaction.fields.borrow();
        let field = fields.get(&id)?;
        if !field.has_selection() {
            return None;
        }
        Some(field.selected_text().to_string())
    }

    /// Removes the selection, as cut does after the text is on the clipboard.
    pub fn cut_selection(&mut self, id: NodeId, cx: &mut Context<Self>) {
        if !self.editable(id) {
            return;
        }
        self.delete(id, true, false, cx);
    }

    /// Mouse down inside a field: place the cursor, or extend from the anchor with shift.
    pub fn field_mouse_down(&mut self, id: NodeId, position: Point<Pixels>, extend: bool) {
        self.edit_field(id, |field| {
            let Some(layout) = field.layout.as_ref() else {
                return;
            };
            let index = match layout.index_for_position(position) {
                Ok(index) | Err(index) => index.min(field.text.len()),
            };
            field.move_to(index, extend);
            field.dragging = true;
        });
    }

    pub fn field_mouse_move(&mut self, id: NodeId, position: Point<Pixels>) {
        self.edit_field(id, |field| {
            if !field.dragging {
                return;
            }
            let Some(layout) = field.layout.as_ref() else {
                return;
            };
            let index = match layout.index_for_position(position) {
                Ok(index) | Err(index) => index.min(field.text.len()),
            };
            field.move_to(index, true);
        });
    }

    pub fn field_mouse_up(&mut self, id: NodeId) {
        self.edit_field(id, |field| field.dragging = false);
    }

    /// Selects a range the app asked for, in UTF-16 offsets.
    pub fn select_range(&mut self, id: NodeId, start: usize, end: usize) {
        self.edit_field(id, |field| {
            field.anchor = byte_offset(&field.text, start);
            field.cursor = byte_offset(&field.text, end);
        });
    }

    /// Writes text to the system clipboard and keeps the copy the app can read.
    pub fn write_clipboard(&mut self, text: String, cx: &mut Context<Self>) {
        if std::env::var("SVELTE_GPUI_TRACE").is_ok() {
            println!("clipboard write: {text:?}");
        }
        *self.svelte.clipboard.borrow_mut() = text.clone();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    /// Reads the system clipboard into the copy the app can read, and returns its text. An image
    /// on it goes to a file in the temporary directory, for `clipboard.readImage()`.
    pub fn read_clipboard(&mut self, cx: &mut Context<Self>) -> String {
        let item = cx.read_from_clipboard();
        let text = match item.as_ref().and_then(|item| item.text()) {
            Some(text) => text,
            None => String::new(),
        };
        let image = item.as_ref().and_then(|item| {
            item.entries().iter().find_map(|entry| match entry {
                ClipboardEntry::Image(image) => Some(image),
                ClipboardEntry::String(_) => None,
            })
        });
        let saved = image.and_then(|image| match save_image(&std::env::temp_dir(), image.format, &image.bytes) {
            Ok(saved) => Some(saved),
            Err(error) => {
                eprintln!("svelte-gpui: cannot keep the clipboard's image: {error}");
                None
            }
        });
        *self.svelte.clipboard.borrow_mut() = text.clone();
        *self.svelte.clipboard_image.borrow_mut() = saved;
        text
    }
}
