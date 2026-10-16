//! The text of one field, and where its cursor and selection are. The host keeps it,
//! because it owns the keyboard and the layout; the app owns the `value` attribute.

use std::ops::Range;

use gpui::{Point, TextLayout};

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
    /// The text an input method is still composing, as a byte range inside `text`.
    pub marked: Option<Range<usize>>,
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
            marked: None,
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

    pub(crate) fn previous_boundary(&self, from: usize) -> usize {
        match self.text[..from].char_indices().next_back() {
            Some((index, _)) => index,
            None => 0,
        }
    }

    pub(crate) fn next_boundary(&self, from: usize) -> usize {
        match self.text[from..].chars().next() {
            Some(character) => from + character.len_utf8(),
            None => self.text.len(),
        }
    }

    /// The start of the word before `from`, as Ctrl+Left finds it.
    pub(crate) fn previous_word(&self, from: usize) -> usize {
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

    pub(crate) fn next_word(&self, from: usize) -> usize {
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

    pub(crate) fn line_start(&self, from: usize) -> usize {
        match self.text[..from].rfind('\n') {
            Some(index) => index + 1,
            None => 0,
        }
    }

    pub(crate) fn line_end(&self, from: usize) -> usize {
        match self.text[from..].find('\n') {
            Some(index) => from + index,
            None => self.text.len(),
        }
    }

    pub(crate) fn move_to(&mut self, position: usize, extend: bool) {
        self.cursor = position;
        if !extend {
            self.anchor = position;
        }
    }

    /// The index on the line above or below, in the same column on screen.
    pub(crate) fn vertical(&self, direction: f32) -> Option<usize> {
        let layout = self.layout.as_ref()?;
        let origin = layout.position_for_index(self.cursor)?;
        let target = Point { x: origin.x, y: origin.y + layout.line_height() * direction + layout.line_height() * 0.5 };
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
