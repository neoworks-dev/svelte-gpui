//! Editing for `input` and `textarea`. The text of a field lives here, because the
//! host owns the keyboard and the layout; the app owns the `value` attribute. A key
//! changes the text and tells the app with an `input` event, and the app writing
//! `value` replaces the text. Checkboxes and radios are toggled the same way.

use gpui::{ClipboardItem, Context, Keystroke, Pixels, Point, TextLayout, Window};
use serde_json::json;

use crate::Workbench;
use crate::tree::{Kind, Node, NodeId};

/// The text of one field and where its cursor and selection are, in byte offsets.
pub struct Field {
    pub text: String,
    pub cursor: usize,
    pub anchor: usize,
    /// The `value` attribute this text was last taken from, so an unchanged
    /// attribute does not reset what the user typed.
    pub applied: Option<String>,
    /// The text when the field got focus, which `change` is measured against.
    pub focus_value: String,
    pub layout: Option<TextLayout>,
    pub dragging: bool,
    /// How far a single-line field has scrolled sideways to keep the cursor in view.
    pub scroll_x: f32,
}

impl Field {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            anchor: 0,
            applied: None,
            focus_value: String::new(),
            layout: None,
            dragging: false,
            scroll_x: 0.0,
        }
    }

    pub fn selection(&self) -> std::ops::Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }

    pub fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    /// Replaces the selection, or inserts at the cursor, and puts the cursor after it.
    pub fn insert(&mut self, text: &str) {
        let range = self.selection();
        self.text.replace_range(range.clone(), text);
        self.cursor = range.start + text.len();
        self.anchor = self.cursor;
    }

    fn previous_boundary(&self, from: usize) -> usize {
        match self.text[..from].char_indices().next_back() {
            Some((index, _)) => index,
            None => 0,
        }
    }

    fn next_boundary(&self, from: usize) -> usize {
        match self.text[from..].chars().next() {
            Some(character) => from + character.len_utf8(),
            None => self.text.len(),
        }
    }

    /// The start of the word before `from`, as Ctrl+Left finds it.
    fn previous_word(&self, from: usize) -> usize {
        let mut position = from;
        while position > 0 {
            let before = self.previous_boundary(position);
            if !self.text[before..position].chars().all(char::is_whitespace) {
                break;
            }
            position = before;
        }
        while position > 0 {
            let before = self.previous_boundary(position);
            if self.text[before..position].chars().all(char::is_whitespace) {
                break;
            }
            position = before;
        }
        position
    }

    fn next_word(&self, from: usize) -> usize {
        let mut position = from;
        while position < self.text.len() {
            let after = self.next_boundary(position);
            if self.text[position..after].chars().all(char::is_whitespace) {
                break;
            }
            position = after;
        }
        while position < self.text.len() {
            let after = self.next_boundary(position);
            if !self.text[position..after].chars().all(char::is_whitespace) {
                break;
            }
            position = after;
        }
        position
    }

    fn line_start(&self, from: usize) -> usize {
        match self.text[..from].rfind('\n') {
            Some(index) => index + 1,
            None => 0,
        }
    }

    fn line_end(&self, from: usize) -> usize {
        match self.text[from..].find('\n') {
            Some(index) => from + index,
            None => self.text.len(),
        }
    }

    fn move_to(&mut self, position: usize, extend: bool) {
        self.cursor = position;
        if !extend {
            self.anchor = position;
        }
    }

    /// The index on the line above or below, in the same column on screen.
    fn vertical(&self, direction: f32) -> Option<usize> {
        let layout = self.layout.as_ref()?;
        let origin = layout.position_for_index(self.cursor)?;
        let target = Point {
            x: origin.x,
            y: origin.y + layout.line_height() * direction + layout.line_height() * 0.5,
        };
        match layout.index_for_position(target) {
            Ok(index) | Err(index) => Some(index.min(self.text.len())),
        }
    }
}

/// UTF-16 offset of a byte offset, which is what a script's string indices count.
pub fn utf16_offset(text: &str, byte: usize) -> usize {
    text[..byte.min(text.len())].encode_utf16().count()
}

/// The byte offset of a UTF-16 offset, clamped to the text.
pub fn byte_offset(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (index, character) in text.char_indices() {
        if units >= utf16 {
            return index;
        }
        units += character.len_utf16();
    }
    text.len()
}

fn attribute<'a>(node: &'a Node, name: &str) -> &'a str {
    match node.attributes.get(name) {
        Some(value) => value.as_str(),
        None => "",
    }
}

/// What kind of editable thing a node is, if it is one.
#[derive(Clone, Copy, PartialEq)]
pub enum Control {
    Text,
    Multiline,
    Checkbox,
    Radio,
}

pub fn control_of(node: &Node) -> Option<Control> {
    let Kind::Element { tag } = &node.kind else {
        return None;
    };
    match tag.as_str() {
        "textarea" => Some(Control::Multiline),
        "input" => match attribute(node, "type") {
            "checkbox" => Some(Control::Checkbox),
            "radio" => Some(Control::Radio),
            "button" | "submit" | "reset" | "file" | "color" | "range" | "image" | "hidden" => None,
            _ => Some(Control::Text),
        },
        _ => None,
    }
}

fn is_true(node: &Node, name: &str) -> bool {
    match node.attributes.get(name) {
        Some(value) => value != "false",
        None => false,
    }
}

/// What may be typed into a field of this kind.
fn accepts(node: &Node, text: &str) -> bool {
    if attribute(node, "type") != "number" {
        return true;
    }
    text.chars()
        .all(|character| character.is_ascii_digit() || matches!(character, '.' | '-' | '+' | 'e'))
}

impl Workbench {
    /// The field behind a node, once it has been drawn.
    fn has_field(&self, id: NodeId) -> bool {
        self.interaction.fields.borrow().contains_key(&id)
    }

    fn editable(&self, id: NodeId) -> bool {
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
            (
                field.text.clone(),
                utf16_offset(&field.text, range.start),
                utf16_offset(&field.text, range.end),
            )
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
        self.emit(id, "input", detail.to_string(), cx);
    }

    /// Fires `change` when the text differs from what it was on focus.
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
        self.emit(id, "change", json!({ "value": value }).to_string(), cx);
    }

    /// Remembers the text a field had on focus, for `change`.
    pub fn field_focused(&mut self, id: NodeId) {
        if let Some(field) = self.interaction.fields.borrow_mut().get_mut(&id) {
            field.focus_value = field.text.clone();
        }
    }

    /// Applies a keystroke to a focused field. Returns whether the field used it.
    pub fn field_key(&mut self, id: NodeId, keystroke: &Keystroke, cx: &mut Context<Self>) -> bool {
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
            "enter" => self.commit_change(id, cx),
            "tab" => return false,
            _ => {
                let Some(text) = &keystroke.key_char else {
                    return false;
                };
                if command || keystroke.modifiers.alt {
                    return false;
                }
                self.type_text(id, text, "insertText", cx);
            }
        }
        true
    }

    fn is_multiline(&self, id: NodeId) -> bool {
        let tree = self.svelte.tree.borrow();
        matches!(tree.nodes.get(&id).and_then(control_of), Some(Control::Multiline))
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
        let allowed = {
            let tree = self.svelte.tree.borrow();
            let Some(node) = tree.nodes.get(&id) else {
                return;
            };
            let mut fields = self.interaction.fields.borrow_mut();
            let Some(field) = fields.get_mut(&id) else {
                return;
            };
            let single_line = text.replace(['\n', '\r'], " ");
            let text = if control_of(node) == Some(Control::Multiline) { text } else { single_line.as_str() };
            let limit: usize = attribute(node, "maxlength").parse().unwrap_or(usize::MAX);
            let room = limit.saturating_sub(field.text.chars().count() - field.selected_text().chars().count());
            let clipped: String = text.chars().take(room).collect();
            if clipped.is_empty() || !accepts(node, &clipped) {
                false
            } else {
                field.insert(&clipped);
                true
            }
        };
        if allowed {
            self.after_edit(id, input_type, Some(text), cx);
        }
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

    /// The default action of a click on a checkbox, a radio, or a label that points at one.
    pub fn click_default(&mut self, id: NodeId, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(select) = self.select_for_click(id) {
            self.toggle_select(select, cx);
            if let Some(handle) = self.interaction.focus.borrow().get(&select) {
                window.focus(handle);
            }
            return;
        }
        let control = self.control_for_click(id);
        let Some(target) = control else {
            return;
        };
        let kind = {
            let tree = self.svelte.tree.borrow();
            tree.nodes.get(&target).and_then(control_of)
        };
        match kind {
            Some(Control::Checkbox) => self.toggle_checkbox(target, cx),
            Some(Control::Radio) => self.choose_radio(target, cx),
            Some(Control::Text) | Some(Control::Multiline) => {
                if let Some(handle) = self.interaction.focus.borrow().get(&target) {
                    window.focus(handle);
                }
            }
            None => {}
        }
    }

    /// The select a click landed in, if any.
    fn select_for_click(&self, id: NodeId) -> Option<NodeId> {
        let tree = self.svelte.tree.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            let node = tree.nodes.get(&candidate)?;
            if matches!(&node.kind, Kind::Element { tag } if tag == "select") {
                return Some(candidate);
            }
            current = node.parent;
        }
        None
    }

    /// The control a click lands on: the node itself, or the one a `label` stands for.
    fn control_for_click(&self, id: NodeId) -> Option<NodeId> {
        let tree = self.svelte.tree.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            let node = tree.nodes.get(&candidate)?;
            if control_of(node).is_some() {
                return Some(candidate);
            }
            if matches!(&node.kind, Kind::Element { tag } if tag == "label") {
                return label_target(&tree, candidate);
            }
            current = node.parent;
        }
        None
    }

    fn toggle_checkbox(&mut self, id: NodeId, cx: &mut Context<Self>) {
        let checked = {
            let mut tree = self.svelte.tree.borrow_mut();
            let Some(node) = tree.nodes.get_mut(&id) else {
                return;
            };
            if is_true(node, "disabled") {
                return;
            }
            let next = !is_true(node, "checked");
            node.attributes.insert("checked".to_string(), next.to_string());
            next
        };
        let detail = json!({ "checked": checked }).to_string();
        self.emit(id, "input", detail.clone(), cx);
        self.emit(id, "change", detail, cx);
    }

    fn choose_radio(&mut self, id: NodeId, cx: &mut Context<Self>) {
        {
            let mut tree = self.svelte.tree.borrow_mut();
            let group = match tree.nodes.get(&id) {
                Some(node) if !is_true(node, "disabled") && !is_true(node, "checked") => {
                    attribute(node, "name").to_string()
                }
                _ => return,
            };
            let others: Vec<NodeId> = tree
                .nodes
                .iter()
                .filter(|(other, node)| {
                    **other != id
                        && control_of(node) == Some(Control::Radio)
                        && !group.is_empty()
                        && attribute(node, "name") == group
                })
                .map(|(other, _)| *other)
                .collect();
            for other in others {
                if let Some(node) = tree.nodes.get_mut(&other) {
                    node.attributes.insert("checked".to_string(), "false".to_string());
                }
            }
            if let Some(node) = tree.nodes.get_mut(&id) {
                node.attributes.insert("checked".to_string(), "true".to_string());
            }
        }
        let detail = json!({ "checked": true }).to_string();
        self.emit(id, "input", detail.clone(), cx);
        self.emit(id, "change", detail, cx);
    }

    /// Writes text to the system clipboard and keeps the copy the app can read.
    pub fn write_clipboard(&mut self, text: String, cx: &mut Context<Self>) {
        *self.svelte.clipboard.borrow_mut() = text.clone();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    /// Reads the system clipboard into the copy the app can read, and returns it.
    pub fn read_clipboard(&mut self, cx: &mut Context<Self>) -> String {
        let text = match cx.read_from_clipboard().and_then(|item| item.text()) {
            Some(text) => text,
            None => String::new(),
        };
        *self.svelte.clipboard.borrow_mut() = text.clone();
        text
    }
}

/// The control a `label` is for: the element its `for` names, or the first one inside it.
fn label_target(tree: &crate::tree::Tree, label: NodeId) -> Option<NodeId> {
    let node = tree.nodes.get(&label)?;
    if let Some(target) = node.attributes.get("for") {
        return tree.nodes.iter().find_map(|(id, candidate)| {
            let named = candidate.attributes.get("id").map(String::as_str) == Some(target.as_str());
            if named && control_of(candidate).is_some() {
                return Some(*id);
            }
            None
        });
    }
    let mut pending: Vec<NodeId> = node.children.clone();
    while let Some(next) = pending.pop() {
        let candidate = tree.nodes.get(&next)?;
        if control_of(candidate).is_some() {
            return Some(next);
        }
        pending.extend(candidate.children.iter().copied());
    }
    None
}
