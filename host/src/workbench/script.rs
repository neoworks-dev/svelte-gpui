//! Scripted steps (`--do press:Save`), for testing where nobody can click.

use gpui::{Context, Keystroke, Window};

use super::Workbench;
use crate::render::{self};
use crate::tree::{Command, Kind, NodeId, ROOT};

/// A step taken apart: `ACTION:TARGET:ARGUMENT`.
#[derive(Debug, PartialEq)]
struct Parts {
    action: String,
    target: String,
    argument: String,
}

/// Steps whose argument is never free text, so it is the part after the last colon and the target
/// may contain colons as they are.
const LAST_COLON: [&str; 5] = ["key", "scroll", "down", "move", "up"];

/// Splits a step at its colons. `\:` is a colon that does not split, and `\\` is a backslash. Of the
/// steps that take an argument, `key`, `scroll`, `down`, `move` and `up` end in it, so a target
/// such as `echo #1: hello` needs no escape there; for the others (`type`, `compose`, `commit`,
/// `drop`, `dragover`) the target ends at its first colon and the rest is the argument, which
/// may hold colons.
fn split_step(step: &str) -> Parts {
    let mut fields = vec![String::new()];
    let mut characters = step.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\\' if matches!(characters.peek(), Some(':' | '\\')) => {
                fields.last_mut().unwrap().push(characters.next().unwrap());
            }
            ':' => fields.push(String::new()),
            other => fields.last_mut().unwrap().push(other),
        }
    }
    let mut fields = fields.into_iter();
    let action = fields.next().unwrap_or_default();
    let rest: Vec<String> = fields.collect();
    if LAST_COLON.contains(&action.as_str()) && rest.len() >= 2 {
        let (argument, target) = rest.split_last().unwrap();
        return Parts { action, target: target.join(":"), argument: argument.clone() };
    }
    let target = rest.first().cloned().unwrap_or_default();
    let argument = rest.iter().skip(1).cloned().collect::<Vec<_>>().join(":");
    Parts { action, target, argument }
}

impl Workbench {
    /// One scripted step: a click, a focus, or a scroll of the nearest scrollable ancestor.
    pub(crate) fn run_step(&mut self, step: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Parts { action, target: needle, argument } = split_step(step);
        let (action, needle, argument) = (action.as_str(), needle.as_str(), argument.as_str());
        if action == "wait" {
            return;
        }
        if action == "dismiss" {
            self.dismiss_errors(cx);
            return;
        }
        if action == "frames" {
            self.frame_times.clear();
            self.frame_target = needle.parse().unwrap_or(0);
            cx.notify();
            return;
        }
        let Some(id) = self.find_node(needle) else {
            eprintln!("nothing matches {needle:?}");
            return;
        };
        if action == "hover" || action == "unhover" {
            let target = self.listening_ancestor(id, "hover");
            self.hover_changed(target, action == "hover", cx);
            return;
        }
        if action == "poke" {
            self.poked = Some(id);
            return;
        }
        if action == "drop" || action == "dragover" {
            self.simulate_drag(id, argument, action == "drop", cx);
            return;
        }
        if matches!(action, "down" | "move" | "up") {
            if action == "down" {
                self.press_focus(id, window);
            }
            let id = if action == "down" {
                // A press goes to the nearest element that takes this button, as a real one does.
                let button = argument.split(',').nth(2).and_then(|button| button.trim().parse().ok()).unwrap_or(0);
                self.pressed_ancestor(id, button)
            } else {
                self.listening_ancestor(id, action)
            };
            self.run_pointer_step(action, id, if argument.is_empty() { "0,0" } else { argument }, cx);
            self.dirty_all_views(cx);
            cx.notify();
            return;
        }
        match action {
            "click" | "press" => {
                let target = self.listening_ancestor(id, "press");
                // A real click goes down first, which overlays opened elsewhere hear as an outside press.
                let middle = self.interaction.bounds.borrow().get(&target).map(|bounds| bounds.center());
                if let Some(middle) = middle {
                    self.simulate_press_outside(middle, 0, cx);
                }
                self.press_focus(id, window);
                self.emit(target, "press", "{\"x\":0,\"y\":0,\"button\":0,\"clicks\":1}".to_string(), cx);
            }
            "focus" => {
                let command = Command::Focus(self.focusable_ancestor(id));
                if !self.carry_out(&command, window, cx) {
                    self.queued.push((command, 20));
                }
            }
            "type" => {
                let command = Command::Focus(id);
                self.carry_out(&command, window, cx);
                for character in argument.chars() {
                    let text = character.to_string();
                    let keystroke =
                        Keystroke { modifiers: Default::default(), key: text.clone(), key_char: Some(text) };
                    self.handle_key(id, &keystroke, false, true, cx);
                }
            }
            "compose" | "commit" => {
                let command = Command::Focus(id);
                self.carry_out(&command, window, cx);
                let text = argument;
                if action == "compose" {
                    <Self as gpui::EntityInputHandler>::replace_and_mark_text_in_range(
                        self, None, text, None, window, cx,
                    );
                } else {
                    <Self as gpui::EntityInputHandler>::replace_text_in_range(self, None, text, window, cx);
                }
            }
            "key" => {
                let spelled = argument;
                match Keystroke::parse(spelled) {
                    Ok(keystroke) => {
                        self.handle_key(id, &keystroke, false, true, cx);
                    }
                    Err(_) => eprintln!("cannot read keystroke {spelled:?}"),
                }
            }
            "scroll" => {
                let pixels: f32 = argument.parse().unwrap_or(0.0);
                let target = self.scrollable_ancestor(id);
                self.queued.push((Command::ScrollTo(target, 0.0, pixels), 20));
            }
            _ => eprintln!("unknown step {step:?}"),
        }
        self.dirty_all_views(cx);
        cx.notify();
    }

    /// `down` presses on an element, which then holds the pointer; `move` and `up` go to the
    /// element holding it. The argument is the position in the window as `x,y`, and optionally the button after it as `x,y,2`.
    fn run_pointer_step(&mut self, action: &str, id: NodeId, at: &str, cx: &mut Context<Self>) {
        let mut numbers = at.split(',').map(|number| number.trim().parse::<f32>().unwrap_or(0.0));
        let (x, y) = (numbers.next().unwrap_or(0.0), numbers.next().unwrap_or(0.0));
        // A third number names the button: 0 left (the default), 1 middle, 2 right.
        let button = numbers.next().unwrap_or(0.0) as i32;
        let position = gpui::point(gpui::px(x), gpui::px(y));
        let target = self.captured.unwrap_or(id);
        let detail = render::pointer_detail(target, position, Default::default(), button, 1, self);
        match action {
            "down" => {
                self.simulate_press_outside(position, button, cx);
                self.captured = Some(id);
                self.emit(id, "down", detail, cx);
            }
            "move" => {
                self.emit(target, "move", detail, cx);
            }
            _ => {
                self.captured = None;
                self.emit(target, "up", detail, cx);
            }
        }
    }

    /// The closest element at or above `id` that listens for `event`, as a real click would find.
    fn listening_ancestor(&self, id: NodeId, event: &str) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            let node = tree.nodes.get(&candidate);
            if node.is_some_and(|node| node.listeners.contains(event)) {
                return candidate;
            }
            current = node.and_then(|node| node.parent);
        }
        id
    }

    /// The closest element at or above `id` that listens for `down` and takes this button.
    fn pressed_ancestor(&self, id: NodeId, button: i32) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            let node = tree.nodes.get(&candidate);
            if node.is_some_and(|node| node.listeners.contains("down") && render::takes_button(node, button)) {
                return candidate;
            }
            current = node.and_then(|node| node.parent);
        }
        id
    }

    /// The closest element at or above `id` that can take focus, as pressing it would give focus to.
    fn focusable_ancestor(&self, id: NodeId) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let handles = self.interaction.focus.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            if handles.contains_key(&candidate) {
                return candidate;
            }
            current = tree.nodes.get(&candidate).and_then(|node| node.parent);
        }
        id
    }

    /// The closest element at or above `id` that scrolls.
    fn scrollable_ancestor(&self, id: NodeId) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let handles = self.interaction.scroll.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            if handles.contains_key(&candidate) {
                return candidate;
            }
            current = tree.nodes.get(&candidate).and_then(|node| node.parent);
        }
        id
    }

    /// An element by its text, or by `@attribute=value` such as `@placeholder=Your name`.
    fn find_node(&self, needle: &str) -> Option<NodeId> {
        let Some(query) = needle.strip_prefix('@') else {
            return self.find_by_text(needle);
        };
        let (name, value) = query.split_once('=')?;
        let tree = self.svelte.tree.borrow();
        let mut matches: Vec<NodeId> = tree
            .nodes
            .iter()
            .filter(|(_, node)| match (&node.kind, name) {
                (Kind::Element { tag }, "tag") => tag == value,
                _ => node.attributes.get(name).map(String::as_str) == Some(value),
            })
            .map(|(id, _)| *id)
            .collect();
        matches.sort();
        matches.first().copied()
    }

    /// The deepest element whose text contains `needle`, for scripted clicks.
    fn find_by_text(&self, needle: &str) -> Option<NodeId> {
        let tree = self.svelte.tree.borrow();
        let mut current = ROOT;
        if !tree.text_content(current).contains(needle) {
            return None;
        }
        loop {
            let node = tree.nodes.get(&current)?;
            let next = node.children.iter().find(|child| {
                matches!(tree.nodes.get(child).map(|n| &n.kind), Some(Kind::Element { .. }))
                    && tree.text_content(**child).contains(needle)
            });
            match next {
                Some(child) => current = *child,
                None => return Some(current),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(step: &str) -> (String, String, String) {
        let Parts { action, target, argument } = split_step(step);
        (action, target, argument)
    }

    fn parts(action: &str, target: &str, argument: &str) -> (String, String, String) {
        (action.to_string(), target.to_string(), argument.to_string())
    }

    #[test]
    fn plain_steps_split_as_before() {
        assert_eq!(split("press:Save"), parts("press", "Save", ""));
        assert_eq!(split("wait"), parts("wait", "", ""));
        assert_eq!(split("type:@placeholder=Name:Ada"), parts("type", "@placeholder=Name", "Ada"));
        assert_eq!(split("type:Name:a:b"), parts("type", "Name", "a:b"));
        assert_eq!(split("down:Canvas:10,20,2"), parts("down", "Canvas", "10,20,2"));
        assert_eq!(split("frames:30"), parts("frames", "30", ""));
    }

    #[test]
    fn a_target_with_a_colon_splits_from_the_end_for_steps_that_end_in_their_argument() {
        assert_eq!(split("key:echo #1: hello:down"), parts("key", "echo #1: hello", "down"));
        assert_eq!(split("scroll:Time 12:30:400"), parts("scroll", "Time 12:30", "400"));
        assert_eq!(split("down:12:30:5,6"), parts("down", "12:30", "5,6"));
    }

    #[test]
    fn an_escaped_colon_does_not_split() {
        assert_eq!(split(r"press:echo #1\: hello"), parts("press", "echo #1: hello", ""));
        assert_eq!(split(r"type:Time 12\:30:ok"), parts("type", "Time 12:30", "ok"));
        assert_eq!(split(r"key:a\:b:enter"), parts("key", "a:b", "enter"));
        assert_eq!(split(r"press:@href=http\://x"), parts("press", "@href=http://x", ""));
    }

    #[test]
    fn a_backslash_before_a_backslash_or_colon_is_one_backslash() {
        assert_eq!(split(r"press:a\\b"), parts("press", r"a\b", ""));
        assert_eq!(split(r"press:a\\:b"), parts("press", r"a\", "b"));
        assert_eq!(split(r"press:C:\dir"), parts("press", "C", r"\dir"));
    }
}
