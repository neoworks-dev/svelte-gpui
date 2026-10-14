//! `select` elements: the closed control shows the chosen option, and a click opens
//! a list of them. Choosing one sets `selected` on the options and tells the app with
//! `input` and `change`, which is what `bind:value` listens for.

use gpui::{Context, Keystroke};
use serde_json::json;

use crate::Workbench;
use crate::tree::{Kind, Node, NodeId, Tree};

/// One option of a select, flattened out of any `optgroup`.
#[derive(Clone)]
pub struct Choice {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub selected: bool,
    pub disabled: bool,
    /// The `optgroup` this option sits under, shown as a heading before its first option.
    pub group: Option<String>,
}

fn attribute<'a>(node: &'a Node, name: &str) -> &'a str {
    match node.attributes.get(name) {
        Some(value) => value.as_str(),
        None => "",
    }
}

fn flag(node: &Node, name: &str) -> bool {
    match node.attributes.get(name) {
        Some(value) => value != "false",
        None => false,
    }
}

/// The options of a select in document order. The chosen one is `selected`, or the
/// first when none says so, as a browser shows it.
pub fn choices(tree: &Tree, select: NodeId) -> Vec<Choice> {
    let mut found = Vec::new();
    collect(tree, select, None, &mut found);
    if !found.iter().any(|choice| choice.selected) {
        if let Some(first) = found.first_mut() {
            first.selected = true;
        }
    }
    found
}

fn collect(tree: &Tree, parent: NodeId, group: Option<&str>, found: &mut Vec<Choice>) {
    let Some(node) = tree.nodes.get(&parent) else {
        return;
    };
    for child in &node.children {
        let Some(child_node) = tree.nodes.get(child) else {
            continue;
        };
        let Kind::Element { tag } = &child_node.kind else {
            continue;
        };
        match tag.as_str() {
            "option" => {
                let text = tree.text_content(*child).trim().to_string();
                let label = match attribute(child_node, "label") {
                    "" => text.clone(),
                    label => label.to_string(),
                };
                let value = match child_node.attributes.get("value") {
                    Some(value) => value.clone(),
                    None => text,
                };
                found.push(Choice {
                    id: *child,
                    label,
                    value,
                    selected: flag(child_node, "selected"),
                    disabled: flag(child_node, "disabled"),
                    group: group.map(String::from),
                });
            }
            "optgroup" => {
                let label = attribute(child_node, "label").to_string();
                collect(tree, *child, Some(&label), found);
            }
            _ => {}
        }
    }
}

impl Workbench {
    /// Opens the list under a select, or closes it.
    pub fn toggle_select(&mut self, select: NodeId, cx: &mut Context<Self>) {
        let disabled = {
            let tree = self.svelte.tree.borrow();
            match tree.nodes.get(&select) {
                Some(node) => flag(node, "disabled"),
                None => true,
            }
        };
        if disabled {
            return;
        }
        let mut open = self.interaction.open_select.borrow_mut();
        *open = if *open == Some(select) { None } else { Some(select) };
        drop(open);
        self.touch(select, cx);
    }

    pub fn close_select(&mut self, cx: &mut Context<Self>) {
        let was_open = self.interaction.open_select.borrow_mut().take();
        match was_open {
            Some(select) => self.touch(select, cx),
            None => cx.notify(),
        }
    }

    /// Makes the option at `index` the chosen one, and tells the app.
    pub fn choose_option(&mut self, select: NodeId, index: usize, cx: &mut Context<Self>) {
        let value = {
            let mut tree = self.svelte.tree.borrow_mut();
            let options = choices(&tree, select);
            let Some(chosen) = options.get(index) else {
                return;
            };
            if chosen.disabled {
                return;
            }
            for option in &options {
                if let Some(node) = tree.nodes.get_mut(&option.id) {
                    node.attributes
                        .insert("selected".to_string(), (option.id == chosen.id).to_string());
                }
            }
            chosen.value.clone()
        };
        *self.interaction.open_select.borrow_mut() = None;
        let detail = json!({ "selectedIndex": index, "value": value }).to_string();
        self.emit(select, "input", detail.clone(), cx);
        self.emit(select, "change", detail, cx);
    }

    /// Arrow keys step through the options of a focused select; Enter and Space open it.
    pub fn select_key(&mut self, select: NodeId, keystroke: &Keystroke, cx: &mut Context<Self>) -> bool {
        let is_select = {
            let tree = self.svelte.tree.borrow();
            matches!(tree.nodes.get(&select).map(|node| &node.kind), Some(Kind::Element { tag }) if tag == "select")
        };
        if !is_select {
            return false;
        }
        let (options, current) = {
            let tree = self.svelte.tree.borrow();
            let options = choices(&tree, select);
            let current = options.iter().position(|option| option.selected).unwrap_or(0);
            (options, current)
        };
        match keystroke.key.as_str() {
            "down" | "up" => {
                let step: isize = if keystroke.key == "down" { 1 } else { -1 };
                let mut next = current as isize + step;
                while next >= 0 && (next as usize) < options.len() && options[next as usize].disabled {
                    next += step;
                }
                if next >= 0 && (next as usize) < options.len() {
                    self.choose_option(select, next as usize, cx);
                }
                true
            }
            "enter" | "space" => {
                self.toggle_select(select, cx);
                true
            }
            "escape" => {
                self.close_select(cx);
                true
            }
            _ => false,
        }
    }
}
