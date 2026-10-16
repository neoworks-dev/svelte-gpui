//! Scripted steps (`--do press:Save`), for testing where nobody can click.

use gpui::{Context, Keystroke, Window};

use super::Workbench;
use crate::render::{self};
use crate::tree::{Command, Kind, NodeId, ROOT};

impl Workbench {
    /// One scripted step: a click, a focus, or a scroll of the nearest scrollable ancestor.
    pub(crate) fn run_step(&mut self, step: &str, window: &mut Window, cx: &mut Context<Self>) {
        let mut parts = step.splitn(3, ':');
        let action = parts.next().unwrap_or("");
        let needle = parts.next().unwrap_or("");
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
            self.simulate_drag(id, parts.next().unwrap_or(""), action == "drop", cx);
            return;
        }
        if matches!(action, "down" | "move" | "up") {
            let id = if action == "down" {
                // A press goes to the nearest element that takes this button, as a real one does.
                let button = parts.clone().next().and_then(|at| at.split(',').nth(2)?.trim().parse().ok()).unwrap_or(0);
                self.pressed_ancestor(id, button)
            } else {
                self.listening_ancestor(id, action)
            };
            self.run_pointer_step(action, id, parts.next().unwrap_or("0,0"), cx);
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
                for character in parts.next().unwrap_or("").chars() {
                    let text = character.to_string();
                    let keystroke =
                        Keystroke { modifiers: Default::default(), key: text.clone(), key_char: Some(text) };
                    self.handle_key(id, &keystroke, false, true, cx);
                }
            }
            "compose" | "commit" => {
                let command = Command::Focus(id);
                self.carry_out(&command, window, cx);
                let text = parts.next().unwrap_or("");
                if action == "compose" {
                    <Self as gpui::EntityInputHandler>::replace_and_mark_text_in_range(
                        self, None, text, None, window, cx,
                    );
                } else {
                    <Self as gpui::EntityInputHandler>::replace_text_in_range(self, None, text, window, cx);
                }
            }
            "key" => {
                let spelled = parts.next().unwrap_or("");
                match Keystroke::parse(spelled) {
                    Ok(keystroke) => {
                        self.handle_key(id, &keystroke, false, true, cx);
                    }
                    Err(_) => eprintln!("cannot read keystroke {spelled:?}"),
                }
            }
            "scroll" => {
                let pixels: f32 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
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
