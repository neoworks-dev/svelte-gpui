//! The app's own GPUI views (`Elements`): bringing the ones in the tree up to date, and running the methods the
//! app calls on them.

use std::collections::HashSet;

use gpui::Context;
use serde_json::Value;

use super::Workbench;
use crate::elements::Live;
use crate::tree::{Command, ELEMENT_TAG, Kind, NodeId};

/// The attribute prefix the runtime sends a component's props under: `prop:path` holds the JSON of `path`.
const PROP_PREFIX: &str = "prop:";

/// What the tree says about an element node: which view it shows and the props it was given.
struct Wanted {
    name: String,
    key: Option<String>,
    /// Each prop's name and the JSON text it was sent as.
    props: Vec<(String, String)>,
}

impl Workbench {
    /// What the tree says node `id` should show, or `None` when it is gone or names no element yet.
    fn wanted_element(&self, id: NodeId) -> Option<Wanted> {
        let tree = self.svelte.tree.borrow();
        let node = tree.nodes.get(&id)?;
        if !matches!(&node.kind, Kind::Element { tag } if tag == ELEMENT_TAG) {
            return None;
        }
        let name = node.attributes.get("name")?.clone();
        let key = node.attributes.get("key").cloned();
        let mut props: Vec<(String, String)> = node
            .attributes
            .iter()
            .filter_map(|(attribute, json)| Some((attribute.strip_prefix(PROP_PREFIX)?.to_string(), json.clone())))
            .collect();
        props.sort();
        Some(Wanted { name, key, props })
    }

    /// Brings the views behind the element nodes in step with the tree: builds the ones that appeared, releases
    /// those that went, and hands the ones that stayed the props that changed. Views kept for a reload that
    /// nobody asked for in time are released.
    pub(super) fn sync_elements(&mut self, cx: &mut Context<Self>) {
        let (changed, dropped) = {
            let mut tree = self.svelte.tree.borrow_mut();
            (std::mem::take(&mut tree.changed_elements), std::mem::take(&mut tree.dropped_elements))
        };
        for id in dropped {
            self.interaction.instances.borrow_mut().remove(id);
            self.interaction.focus.borrow_mut().remove(&id);
        }
        let mut sorted: Vec<NodeId> = changed.into_iter().collect();
        sorted.sort();
        for id in sorted {
            self.update_element(id, cx);
        }
        self.interaction.instances.borrow_mut().expire();
    }

    /// Makes node `id` show the view the tree names for it, with the props the tree gives it.
    fn update_element(&mut self, id: NodeId, cx: &mut Context<Self>) {
        let Some(wanted) = self.wanted_element(id) else {
            return;
        };
        let existing = self.interaction.instances.borrow_mut().live.remove(&id);
        let mut live = match existing {
            Some(live) if live.name == wanted.name && live.key == wanted.key => live,
            replaced => {
                if replaced.is_some() {
                    self.interaction.focus.borrow_mut().remove(&id);
                }
                // A view the new run of the app asked for by name and key is taken back, not built again.
                self.interaction.instances.borrow_mut().remember(replaced);
                match self.bring_in_view(id, &wanted, cx) {
                    Some(live) => live,
                    None => return,
                }
            }
        };
        apply_props(&mut live, &wanted.props, cx);
        self.interaction.instances.borrow_mut().live.insert(id, live);
        self.touch(id, cx);
    }

    /// A view for node `id`: one kept from the last run of the app when the key matches, otherwise a new one.
    fn bring_in_view(&mut self, id: NodeId, wanted: &Wanted, cx: &mut Context<Self>) -> Option<Live> {
        let kept = self.interaction.instances.borrow_mut().claim(&wanted.name, &wanted.key);
        let live = match kept {
            Some(live) => live,
            None => match self.elements.create(&wanted.name, wanted.key.clone(), cx) {
                Some(live) => live,
                None => {
                    eprintln!(
                        "svelte-gpui: the app uses element({:?}), but registers {:?}",
                        wanted.name,
                        self.elements.names()
                    );
                    return None;
                }
            },
        };
        live.attach(id);
        if let Some(handle) = live.handle.focus_handle(cx) {
            self.interaction.focus.borrow_mut().insert(id, handle);
        }
        Some(live)
    }

    /// The window starts the app again: every view leaves the tree, and those with a key wait for the new tree.
    pub(super) fn stash_elements(&mut self) {
        self.interaction.instances.borrow_mut().stash_for_reload();
    }

    /// Whether a view of the app's own is drawn in node `id`.
    pub(super) fn is_element(&self, id: NodeId) -> bool {
        self.interaction.instances.borrow().live.contains_key(&id)
    }

    /// Runs the method calls the app just made on views that exist, before anything is drawn. A call on a view
    /// that is not there yet waits for the next tick.
    pub(super) fn run_element_calls(&mut self, cx: &mut Context<Self>) {
        self.sync_elements(cx);
        let ready: Vec<Command> = {
            let instances = self.interaction.instances.borrow();
            let mut ready = Vec::new();
            self.svelte.tree.borrow_mut().commands.retain(|command| match command {
                Command::Call { node, .. } if instances.live.contains_key(node) => {
                    ready.push(command.clone());
                    false
                }
                _ => true,
            });
            ready
        };
        for command in ready {
            self.carry_out_call(&command, cx);
        }
    }

    /// Runs one method call, and settles the app's promise with the answer. False when the view is not there.
    pub(super) fn carry_out_call(&mut self, command: &Command, cx: &mut Context<Self>) -> bool {
        let Command::Call { node, call, method, args } = command else {
            return true;
        };
        let handle = self.interaction.instances.borrow().live.get(node).map(|live| live.handle.clone());
        let Some(handle) = handle else {
            return false;
        };
        let args: Value = serde_json::from_str(args).unwrap_or(Value::Array(Vec::new()));
        let sink = self.svelte.io.sink();
        match handle.call(method, args, cx) {
            Ok(answer) => sink.resolve(*call, answer),
            Err(error) => match error.code {
                Some(code) => sink.reject_with_code(*call, &code, error.message),
                None => sink.reject(*call, error.message),
            },
        }
        self.touch(*node, cx);
        true
    }

    /// Settles a method call that never found its view.
    pub(super) fn abandon_call(&mut self, command: &Command) {
        if let Command::Call { call, method, .. } = command {
            let message = format!("cannot call {method:?}: the element is not on screen");
            self.svelte.io.sink().reject_with_code(*call, "NoElement", message);
        }
    }

    /// An app element's view emitted an event: hands it to the app when the component listens for it. The
    /// runtime listens under the name `app:` and the event's name, so none of the window's own events collide.
    pub(super) fn deliver_element_event(&mut self, node: NodeId, payload: &Value, cx: &mut Context<Self>) {
        let Some(name) = payload["name"].as_str() else {
            return;
        };
        let event = format!("app:{name}");
        let listened = {
            let tree = self.svelte.tree.borrow();
            tree.nodes.get(&node).is_some_and(|found| found.listeners.contains(&event))
        };
        if !listened {
            return;
        }
        let detail = serde_json::json!({ "detail": payload["detail"] }).to_string();
        self.emit(node, &event, detail, cx);
    }
}

/// Gives the view the props that changed since it last heard, and `null` for those that went.
fn apply_props(live: &mut Live, wanted: &[(String, String)], cx: &mut gpui::App) {
    let mut seen: HashSet<&str> = HashSet::new();
    for (name, json) in wanted {
        seen.insert(name);
        if live.props.get(name) == Some(json) {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(json) else {
            eprintln!("svelte-gpui: the prop {name:?} is not JSON: {json}");
            continue;
        };
        live.handle.set_prop(name, value, cx);
        live.props.insert(name.clone(), json.clone());
    }
    let gone: Vec<String> = live.props.keys().filter(|name| !seen.contains(name.as_str())).cloned().collect();
    for name in gone {
        live.handle.set_prop(&name, Value::Null, cx);
        live.props.remove(&name);
    }
}
