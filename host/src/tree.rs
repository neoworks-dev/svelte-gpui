//! The host's copy of the document: the nodes Svelte created, in the shape the
//! runtime's operations describe. The runtime sends only changes, so this is
//! what a frame is drawn from.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

pub type NodeId = u32;

pub const ROOT: NodeId = 1;

pub enum Kind {
    Element { tag: String },
    Text { data: String },
}

pub struct Node {
    pub kind: Kind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub attributes: HashMap<String, String>,
    /// Inline declarations, in the order they were set.
    pub style: Vec<(String, String)>,
    pub listeners: HashSet<String>,
}

impl Node {
    /// The app listens for this element's size and position, or only for its size.
    pub fn observed(&self) -> bool {
        self.listeners.contains("layout") || self.listeners.contains("resize")
    }
}

/// Something the app asked the host to do that is not a change to the tree.
#[derive(Debug, Clone)]
pub enum Command {
    Focus(NodeId),
    /// Drops focus from an element, if it has it.
    Blur(NodeId),
    ScrollTo(NodeId, f32, f32),
    /// Select a range of a field, in UTF-16 offsets.
    Select(NodeId, usize, usize),
    WriteClipboard(String),
    /// A method call on an app element (`Elements`). `call` is the number the app's promise waits on,
    /// and `args` the JSON array of the arguments.
    Call {
        node: NodeId,
        call: u64,
        method: String,
        args: String,
    },
}

pub struct Tree {
    pub nodes: HashMap<NodeId, Node>,
    /// The pixels of each `g-canvas`, by element.
    pub canvases: HashMap<NodeId, std::rc::Rc<std::cell::RefCell<crate::canvas::Surface>>>,
    /// Canvases that were dropped, whose images the window must still free.
    pub dropped_canvases: Vec<std::rc::Rc<std::cell::RefCell<crate::canvas::Surface>>>,
    /// Bumped by every change; a frame only rebuilds styles when it moved.
    pub version: u64,
    /// Requests from the app, taken by the window on its next tick.
    pub commands: Vec<Command>,
    /// Event types `document` or `window` listen for, which the app reports under id 0.
    pub global_listeners: std::collections::HashSet<String>,
    /// Nodes whose own content or children changed since the window last looked.
    pub dirty: HashSet<NodeId>,
    /// Nodes whose attributes or style changed, which can change how what is under them looks.
    pub dirty_deep: HashSet<NodeId>,
    /// App elements (`g-element`) created or given new attributes, which the window must bring in step
    /// with the Rust views behind them.
    pub changed_elements: HashSet<NodeId>,
    /// App elements removed for good, whose ids no longer name a node.
    pub dropped_elements: Vec<NodeId>,
}

/// The tag of the node an app's own GPUI view is drawn in.
pub const ELEMENT_TAG: &str = "g-element";

// Operation codes, generated from protocol.json, which runtime/host/bridge.ts reads too.
include!(concat!(env!("OUT_DIR"), "/protocol.rs"));

impl Tree {
    pub fn new() -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            canvases: HashMap::new(),
            dropped_canvases: Vec::new(),
            version: 0,
            commands: Vec::new(),
            global_listeners: std::collections::HashSet::new(),
            dirty: HashSet::new(),
            dirty_deep: HashSet::new(),
            changed_elements: HashSet::new(),
            dropped_elements: Vec::new(),
        };
        tree.nodes.insert(
            ROOT,
            Node {
                kind: Kind::Element { tag: "#root".into() },
                parent: None,
                children: Vec::new(),
                attributes: HashMap::new(),
                style: Vec::new(),
                listeners: HashSet::new(),
            },
        );
        tree
    }

    /// Applies one batch of operations, as the runtime serialised it.
    pub fn apply(&mut self, batch: &str) -> usize {
        let Ok(Value::Array(operations)) = serde_json::from_str::<Value>(batch) else {
            eprintln!("svelte-gpui: unreadable batch from the runtime");
            return 0;
        };
        for operation in &operations {
            let Value::Array(parts) = operation else {
                continue;
            };
            self.apply_one(parts);
        }
        self.version += 1;
        operations.len()
    }

    fn apply_one(&mut self, parts: &[Value]) {
        let code = parts.first().and_then(Value::as_i64).unwrap_or(-1);
        let id = parts.get(1).and_then(Value::as_u64).unwrap_or(0) as NodeId;
        match code {
            CREATE_ELEMENT => {
                let tag = text_at(parts, 2);
                self.create(id, Kind::Element { tag });
            }
            CREATE_TEXT => {
                let data = text_at(parts, 2);
                self.create(id, Kind::Text { data });
            }
            SET_TEXT => {
                if let Some(Node { kind: Kind::Text { data }, .. }) = self.nodes.get_mut(&id) {
                    *data = text_at(parts, 2);
                    self.dirty.insert(id);
                }
            }
            SET_ATTRIBUTE => self.set_attribute(id, parts),
            INSERT => {
                let child = parts.get(2).and_then(Value::as_u64).unwrap_or(0) as NodeId;
                let before = parts.get(3).and_then(Value::as_u64).unwrap_or(0) as NodeId;
                self.insert(id, child, before);
            }
            REMOVE => self.detach(id),
            LISTEN => {
                let event = text_at(parts, 2);
                if id == 0 {
                    self.global_listeners.insert(event);
                } else if let Some(node) = self.nodes.get_mut(&id) {
                    node.listeners.insert(event);
                    self.dirty_deep.insert(id);
                }
            }
            UNLISTEN => {
                let event = text_at(parts, 2);
                if id == 0 {
                    self.global_listeners.remove(&event);
                } else if let Some(node) = self.nodes.get_mut(&id) {
                    node.listeners.remove(&event);
                    self.dirty_deep.insert(id);
                }
            }
            SET_STYLE => self.set_style(id, parts),
            DROP => self.drop_subtree(id),
            FOCUS => self.commands.push(Command::Focus(id)),
            BLUR => self.commands.push(Command::Blur(id)),
            SCROLL_TO => {
                let left = parts.get(2).and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let top = parts.get(3).and_then(Value::as_f64).unwrap_or(0.0) as f32;
                self.commands.push(Command::ScrollTo(id, left, top));
            }
            SELECT => {
                let start = parts.get(2).and_then(Value::as_u64).unwrap_or(0) as usize;
                let end = parts.get(3).and_then(Value::as_u64).unwrap_or(0) as usize;
                self.commands.push(Command::Select(id, start, end));
            }
            DRAW => {
                if let Some(Value::Array(command)) = parts.get(2) {
                    self.canvases.entry(id).or_default().borrow_mut().command(command);
                    self.dirty.insert(id);
                }
            }
            WRITE_CLIPBOARD => self.commands.push(Command::WriteClipboard(text_at(parts, 2))),
            CALL => {
                let call = parts.get(2).and_then(Value::as_u64).unwrap_or(0);
                let method = text_at(parts, 3);
                let args = text_at(parts, 4);
                self.commands.push(Command::Call { node: id, call, method, args });
            }
            _ => eprintln!("svelte-gpui: unknown operation {code}"),
        }
    }

    fn create(&mut self, id: NodeId, kind: Kind) {
        self.nodes.insert(
            id,
            Node {
                kind,
                parent: None,
                children: Vec::new(),
                attributes: HashMap::new(),
                style: Vec::new(),
                listeners: HashSet::new(),
            },
        );
    }

    fn set_attribute(&mut self, id: NodeId, parts: &[Value]) {
        let name = text_at(parts, 2);
        let Some(node) = self.nodes.get_mut(&id) else {
            return;
        };
        self.dirty_deep.insert(id);
        if matches!(&node.kind, Kind::Element { tag } if tag == ELEMENT_TAG) {
            self.changed_elements.insert(id);
        }
        match parts.get(3) {
            Some(Value::Null) | None => {
                node.attributes.remove(&name);
            }
            Some(Value::String(value)) => {
                node.attributes.insert(name, value.clone());
            }
            Some(other) => {
                node.attributes.insert(name, other.to_string());
            }
        }
    }

    fn set_style(&mut self, id: NodeId, parts: &[Value]) {
        let property = text_at(parts, 2);
        let Some(node) = self.nodes.get_mut(&id) else {
            return;
        };
        self.dirty_deep.insert(id);
        node.style.retain(|(name, _)| *name != property);
        if let Some(Value::String(value)) = parts.get(3) {
            node.style.push((property, value.clone()));
        }
    }

    /// Puts `child` under `parent`, before `before` or at the end when that is 0.
    fn insert(&mut self, parent: NodeId, child: NodeId, before: NodeId) {
        self.detach(child);
        let Some(parent_node) = self.nodes.get_mut(&parent) else {
            return;
        };
        let position = parent_node
            .children
            .iter()
            .position(|candidate| *candidate == before)
            .unwrap_or(parent_node.children.len());
        parent_node.children.insert(position, child);
        self.dirty.insert(parent);
        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = Some(parent);
        }
    }

    fn detach(&mut self, id: NodeId) {
        let Some(parent) = self.nodes.get(&id).and_then(|node| node.parent) else {
            return;
        };
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.retain(|candidate| *candidate != id);
            self.dirty.insert(parent);
        }
        if let Some(node) = self.nodes.get_mut(&id) {
            node.parent = None;
        }
    }

    fn drop_subtree(&mut self, id: NodeId) {
        self.detach(id);
        let mut pending = vec![id];
        while let Some(next) = pending.pop() {
            if let Some(surface) = self.canvases.remove(&next) {
                self.dropped_canvases.push(surface);
            }
            if let Some(node) = self.nodes.remove(&next) {
                if matches!(&node.kind, Kind::Element { tag } if tag == ELEMENT_TAG) {
                    self.dropped_elements.push(next);
                    self.changed_elements.remove(&next);
                }
                pending.extend(node.children);
            }
        }
    }

    /// All the text under a node, for finding things by what they say.
    pub fn text_content(&self, id: NodeId) -> String {
        let Some(node) = self.nodes.get(&id) else {
            return String::new();
        };
        match &node.kind {
            Kind::Text { data } => data.clone(),
            Kind::Element { .. } => node.children.iter().map(|child| self.text_content(*child)).collect(),
        }
    }
}

fn text_at(parts: &[Value], index: usize) -> String {
    match parts.get(index) {
        Some(Value::String(value)) => value.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}
