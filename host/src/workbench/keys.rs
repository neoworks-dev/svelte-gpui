//! Focus and the keyboard: keys go to the app first, up from what has focus, then to
//! the focused field or the clipboard.

use gpui::{Context, KeyDownEvent, KeyUpEvent, Keystroke, Window};

use super::Workbench;
use crate::tree::{NodeId, Tree};

/// What a press does to focus.
#[derive(Debug, PartialEq)]
pub(super) enum Pressed {
    /// Focus stays where it is.
    Keep,
    /// This element takes it.
    Focus(NodeId),
    /// No element has it any more.
    Blur,
}

/// What a press on `id` does to focus: the nearest element at or above it that takes focus gets it,
/// and when there is none, nothing has focus. An element with `keepfocus` ends the search first and
/// leaves focus where it is, even if it could take focus itself.
pub(super) fn press_outcome(tree: &Tree, takes_focus: impl Fn(NodeId) -> bool, id: NodeId) -> Pressed {
    let mut current = Some(id);
    while let Some(candidate) = current {
        let node = tree.nodes.get(&candidate);
        if node.is_some_and(|node| node.attributes.contains_key("keepfocus")) {
            return Pressed::Keep;
        }
        if takes_focus(candidate) {
            return Pressed::Focus(candidate);
        }
        current = node.and_then(|node| node.parent);
    }
    Pressed::Blur
}

impl Workbench {
    /// Focuses an element the host made a focus handle for.
    pub fn focus_node(&self, id: NodeId, window: &mut Window) {
        if let Some(handle) = self.interaction.focus.borrow().get(&id) {
            window.focus(handle);
        }
    }

    /// Takes focus away from an element, if it has it.
    pub fn blur_node(&self, id: NodeId, window: &mut Window) {
        if self.focused_node(window) == Some(id) {
            self.blur_focused(window);
        }
    }

    /// Takes focus from whatever has it. Focus goes to the window itself, so no element of the
    /// app has it, as when a press lands on empty space.
    pub fn blur_focused(&self, window: &mut Window) {
        window.focus(&self.interaction.root_focus);
    }

    /// What a press on `id` does to focus, for a press the window did not already settle.
    pub fn press_focus(&self, id: NodeId, window: &mut Window) {
        let outcome = {
            let handles = self.interaction.focus.borrow();
            press_outcome(&self.svelte.tree.borrow(), |candidate| handles.contains_key(&candidate), id)
        };
        match outcome {
            Pressed::Keep => {}
            Pressed::Focus(target) => self.focus_node(target, window),
            Pressed::Blur => self.blur_focused(window),
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
        // A view of the app's own has its clipboard keys to itself.
        if let (Some(kind), true) = (clipboard_event, target != 0 && !self.is_element(target)) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{CREATE_ELEMENT, INSERT, ROOT, SET_ATTRIBUTE};

    /// A tree of `g-view`s: 2 holds 3 and 4, and 3 holds 5. Node 3 keeps focus; 2 and 5 can take it.
    fn tree() -> Tree {
        let mut tree = Tree::new();
        let mut batch: Vec<String> = (2..=5).map(|id| format!("[{CREATE_ELEMENT},{id},\"g-view\"]")).collect();
        for (parent, child) in [(ROOT, 2), (2, 3), (2, 4), (3, 5)] {
            batch.push(format!("[{INSERT},{parent},{child},0]"));
        }
        batch.push(format!("[{SET_ATTRIBUTE},3,\"keepfocus\",\"\"]"));
        tree.apply(&format!("[{}]", batch.join(",")));
        tree
    }

    #[test]
    fn a_press_where_nothing_takes_focus_drops_it() {
        assert_eq!(press_outcome(&tree(), |_| false, 4), Pressed::Blur);
    }

    #[test]
    fn the_nearest_element_that_takes_focus_gets_it() {
        assert_eq!(press_outcome(&tree(), |id| id == 2 || id == 4, 4), Pressed::Focus(4));
        assert_eq!(press_outcome(&tree(), |id| id == 2, 4), Pressed::Focus(2));
    }

    #[test]
    fn keepfocus_leaves_focus_alone_for_everything_inside_it() {
        assert_eq!(press_outcome(&tree(), |_| false, 3), Pressed::Keep);
        assert_eq!(press_outcome(&tree(), |_| false, 5), Pressed::Keep);
        // Even an element that could take focus, or has one around it, does not.
        assert_eq!(press_outcome(&tree(), |id| id == 3 || id == 2, 5), Pressed::Keep);
    }

    #[test]
    fn keepfocus_does_not_reach_outside_the_element() {
        assert_eq!(press_outcome(&tree(), |id| id == 2, 4), Pressed::Focus(2));
    }
}
