//! Focus and the keyboard: keys go to the app first, up from what has focus, then to
//! the focused field or the clipboard.

use gpui::{Context, KeyDownEvent, KeyUpEvent, Keystroke, Window};

use super::Workbench;
use crate::tree::NodeId;

impl Workbench {
    /// Focuses an element the host made a focus handle for.
    pub fn focus_node(&self, id: NodeId, window: &mut Window) {
        if let Some(handle) = self.interaction.focus.borrow().get(&id) {
            window.focus(handle);
        }
    }

    /// The element that has keyboard focus, if it is one the app made.
    pub(super) fn focused_node(&self, window: &Window) -> Option<NodeId> {
        let handles = self.interaction.focus.borrow();
        for (id, handle) in handles.iter() {
            if handle.is_focused(window) {
                return Some(*id);
            }
        }
        None
    }

    /// A key press: first to the app, up from whatever has focus, and then, unless the
    /// app cancelled it, to the field that has focus or the clipboard shortcut it spells.
    pub(crate) fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.errors_visible() && event.keystroke.key == "escape" {
            self.dismiss_errors(cx);
            return true;
        }
        let target = self.focused_node(window).unwrap_or(0);
        self.handle_key(target, &event.keystroke, event.is_held, false, cx)
    }

    /// The part of a key press after the target is known; scripted steps come here too.
    /// Returns whether the key was used, so it is not also typed.
    pub(super) fn handle_key(
        &mut self,
        target: NodeId,
        keystroke: &Keystroke,
        held: bool,
        insert_text: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let command = keystroke.modifiers.control || keystroke.modifiers.platform;
        let clipboard_event = match (command, keystroke.key.as_str()) {
            (true, "c") => Some("copy"),
            (true, "x") => Some("cut"),
            (true, "v") => Some("paste"),
            _ => None,
        };
        let detail = key_detail(keystroke, held);
        if self.emit(target, "key", detail, cx).prevented {
            return true;
        }
        if let (Some(kind), true) = (clipboard_event, target != 0) {
            self.clipboard_shortcut(kind, target, cx);
            return true;
        }
        if target != 0 && self.field_key(target, keystroke, insert_text, cx) {
            self.flush_dirty(cx);
            self.caret_moved(target, cx);
            return true;
        }
        false
    }

    /// `copy`, `cut` and `paste` in a field.
    fn clipboard_shortcut(&mut self, kind: &str, target: NodeId, cx: &mut Context<Self>) {
        let clipboard = self.read_clipboard(cx);
        let selected = self.selected_text(target);
        match (kind, selected) {
            ("copy", Some(text)) => self.write_clipboard(text, cx),
            ("cut", Some(text)) => {
                self.write_clipboard(text, cx);
                self.cut_selection(target, cx);
            }
            ("paste", _) => self.paste_into(target, &clipboard, cx),
            _ => {}
        }
    }

    pub(crate) fn key_up(&mut self, event: &KeyUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        let target = self.focused_node(window).unwrap_or(0);
        let _ = target;
        let detail = key_detail(&event.keystroke, false);
        self.emit(0, "keyup", detail, cx);
    }
}

/// The DOM's name for a key: `Enter`, `ArrowLeft`, or the character typed.
fn dom_key(keystroke: &Keystroke) -> String {
    let named = match keystroke.key.as_str() {
        "enter" => Some("Enter"),
        "escape" => Some("Escape"),
        "backspace" => Some("Backspace"),
        "delete" => Some("Delete"),
        "tab" => Some("Tab"),
        "left" => Some("ArrowLeft"),
        "right" => Some("ArrowRight"),
        "up" => Some("ArrowUp"),
        "down" => Some("ArrowDown"),
        "home" => Some("Home"),
        "end" => Some("End"),
        "pageup" => Some("PageUp"),
        "pagedown" => Some("PageDown"),
        "space" => Some(" "),
        _ => None,
    };
    if let Some(name) = named {
        return name.to_string();
    }
    if let Some(number) =
        keystroke.key.strip_prefix('f').filter(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
    {
        return format!("F{number}");
    }
    let held = keystroke.modifiers.control || keystroke.modifiers.alt || keystroke.modifiers.platform;
    if let (Some(text), false) = (&keystroke.key_char, held) {
        return text.clone();
    }
    if keystroke.key.chars().count() == 1 && keystroke.modifiers.shift {
        return keystroke.key.to_uppercase();
    }
    keystroke.key.clone()
}

/// The DOM's `code` for a key, which names the physical key.
fn dom_code(keystroke: &Keystroke) -> String {
    let key = dom_key(keystroke);
    if key == " " {
        return "Space".to_string();
    }
    if keystroke.key.len() == 1 {
        let character = keystroke.key.chars().next().unwrap_or(' ');
        if character.is_ascii_alphabetic() {
            return format!("Key{}", character.to_ascii_uppercase());
        }
        if character.is_ascii_digit() {
            return format!("Digit{character}");
        }
    }
    key
}

fn key_detail(keystroke: &Keystroke, repeat: bool) -> String {
    let key = dom_key(keystroke).replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "{{\"key\":\"{key}\",\"code\":\"{}\",\"repeat\":{repeat},\"ctrl\":{},\"shift\":{},\"alt\":{},\"meta\":{}}}",
        dom_code(keystroke),
        keystroke.modifiers.control,
        keystroke.modifiers.shift,
        keystroke.modifiers.alt,
        keystroke.modifiers.platform
    )
}
