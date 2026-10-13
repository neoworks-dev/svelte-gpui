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

pub struct Tree {
    pub nodes: HashMap<NodeId, Node>,
    /// Bumped by every change; a frame only rebuilds styles when it moved.
    pub version: u64,
}

// Operation codes; runtime/host/bridge.ts sends the same numbers.
const CREATE_ELEMENT: i64 = 0;
const CREATE_TEXT: i64 = 1;
const SET_TEXT: i64 = 2;
const SET_ATTRIBUTE: i64 = 3;
const INSERT: i64 = 4;
const REMOVE: i64 = 5;
const LISTEN: i64 = 6;
const SET_STYLE: i64 = 7;
const DROP: i64 = 8;

impl Tree {
    pub fn new() -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            version: 0,
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
                if let Some(node) = self.nodes.get_mut(&id) {
                    node.listeners.insert(event);
                }
            }
            SET_STYLE => self.set_style(id, parts),
            DROP => self.drop_subtree(id),
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
        }
        if let Some(node) = self.nodes.get_mut(&id) {
            node.parent = None;
        }
    }

    fn drop_subtree(&mut self, id: NodeId) {
        self.detach(id);
        let mut pending = vec![id];
        while let Some(next) = pending.pop() {
            if let Some(node) = self.nodes.remove(&next) {
                pending.extend(node.children);
            }
        }
    }

    pub fn element_tag(&self, id: NodeId) -> Option<&str> {
        match &self.nodes.get(&id)?.kind {
            Kind::Element { tag } => Some(tag),
            Kind::Text { .. } => None,
        }
    }

    /// All the text under a node, for finding things by what they say.
    pub fn text_content(&self, id: NodeId) -> String {
        let Some(node) = self.nodes.get(&id) else {
            return String::new();
        };
        match &node.kind {
            Kind::Text { data } => data.clone(),
            Kind::Element { .. } => node
                .children
                .iter()
                .map(|child| self.text_content(*child))
                .collect(),
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
