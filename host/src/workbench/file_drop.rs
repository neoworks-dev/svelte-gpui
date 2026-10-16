//! Files dragged onto the window from other programs. The element under the pointer that
//! listens for `dragenter`, `dragleave`, `dragover` or `drop` hears about them, with the paths.
//! Dropped paths are allowed through the file limit from then on, as paths chosen in a dialog are.

use std::path::PathBuf;

use gpui::{Context, Modifiers, Pixels, Point};

use super::Workbench;
use crate::render::{listens_for_drag, pointer_detail};
use crate::tree::{Node, NodeId};

/// Which element the dragged files are over.
#[derive(Default)]
pub(super) struct ExternalDrag {
    target: Option<NodeId>,
}

impl Workbench {
    /// Files are being dragged over the window, now at `position`. The element they are over
    /// hears `dragover`, and `dragenter` and `dragleave` when that changes.
    pub(crate) fn external_drag_over(&mut self, position: Point<Pixels>, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let target = self.drag_target_at(position);
        let previous = self.external_drag.target;
        if target != previous {
            if let Some(left) = previous {
                self.tell_drag(left, "dragleave", position, &paths, cx);
            }
            self.external_drag.target = target;
            if let Some(entered) = target {
                self.tell_drag(entered, "dragenter", position, &paths, cx);
            }
        }
        if let Some(over) = target {
            self.tell_drag(over, "dragover", position, &paths, cx);
        }
    }

    /// The dragged files left the window without being dropped.
    pub(crate) fn external_drag_left(&mut self, cx: &mut Context<Self>) {
        if let Some(left) = self.external_drag.target.take() {
            self.tell_drag(left, "dragleave", Point::default(), &[], cx);
        }
    }

    /// The files were dropped at `position`. The element there that listens for `drop` hears the
    /// paths, which the app may read from then on.
    pub(crate) fn external_drop(&mut self, position: Point<Pixels>, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.external_drag_over(position, paths.clone(), cx);
        let Some(target) = self.external_drag.target.take() else {
            return;
        };
        let takes_drop =
            self.svelte.tree.borrow().nodes.get(&target).is_some_and(|node| node.listeners.contains("drop"));
        if !takes_drop {
            return;
        }
        for path in &paths {
            self.svelte.io.files.grant_picked(path);
        }
        self.tell_drag(target, "drop", position, &paths, cx);
    }

    /// Reports one of the drag events to an element, when it listens for it.
    fn tell_drag(
        &mut self,
        id: NodeId,
        kind: &str,
        position: Point<Pixels>,
        paths: &[PathBuf],
        cx: &mut Context<Self>,
    ) {
        let listens = self.svelte.tree.borrow().nodes.get(&id).is_some_and(|node| node.listeners.contains(kind));
        if !listens {
            return;
        }
        let mut detail = pointer_detail(id, position, Modifiers::default(), -1, 0, self);
        detail.pop();
        let names: Vec<String> = paths.iter().map(|path| path.to_string_lossy().into_owned()).collect();
        let names = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        detail.push_str(&format!(",\"paths\":{names},\"count\":{}}}", paths.len()));
        self.emit(id, kind, detail, cx);
    }

    /// The deepest element that listens for drag events and contains the point.
    fn drag_target_at(&self, position: Point<Pixels>) -> Option<NodeId> {
        let tree = self.svelte.tree.borrow();
        let depth = |mut id: NodeId| {
            let mut depth = 0;
            while let Some(parent) = tree.nodes.get(&id).and_then(|node| node.parent) {
                depth += 1;
                id = parent;
            }
            depth
        };
        self.interaction
            .bounds
            .borrow()
            .iter()
            .filter(|(id, bounds)| {
                bounds.contains(&position) && tree.nodes.get(id).is_some_and(|node: &Node| listens_for_drag(node))
            })
            .map(|(id, _)| (depth(*id), *id))
            .max()
            .map(|(_, id)| id)
    }

    /// The scripted steps `dragover:TARGET:PATH,PATH` and `drop:TARGET:PATH,PATH`: files are dragged to
    /// the middle of the first element at or above the target that listens for `drop`, and for
    /// `drop` released there, through the same code a real drag takes.
    pub(super) fn simulate_drag(&mut self, id: NodeId, paths: &str, release: bool, cx: &mut Context<Self>) {
        let target = self.drop_listener_at_or_above(id);
        let center = self.interaction.bounds.borrow().get(&target).map(|bounds| bounds.center()).unwrap_or_default();
        let paths: Vec<PathBuf> = paths.split(',').filter(|path| !path.is_empty()).map(PathBuf::from).collect();
        if release {
            self.external_drop(center, paths, cx);
        } else {
            self.external_drag_over(center, paths, cx);
        }
    }

    fn drop_listener_at_or_above(&self, id: NodeId) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            let node = tree.nodes.get(&candidate);
            if node.is_some_and(|node| node.listeners.contains("drop")) {
                return candidate;
            }
            current = node.and_then(|node| node.parent);
        }
        id
    }
}
