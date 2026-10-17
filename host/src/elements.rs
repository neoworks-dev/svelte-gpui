//! GPUI views of the app's own, drawn inside the window like a primitive.
//!
//! An app whose crate draws something GPUI has to draw itself, such as a code editor, registers it in
//! [`Elements`] under a name, and gives the host that registry in `Options::elements`. The app's Svelte code
//! turns the name into a component with `element(name)`. The host then does three things for each such
//! component that is on screen:
//! - builds the view with the registered factory, and draws it inside the node's box, laid out from the
//!   node's `style` the way a `View` is;
//! - passes the component's props to [`AppElement::set_prop`] as JSON whenever they change, and runs the
//!   methods the app calls through [`AppElement::call`];
//! - reports the view's events to the app, which it emits through its [`NativeHost`].
//!
//! The view keeps its own [`FocusHandle`]. While it has focus, keys reach the view's own key listeners
//! first; what it leaves alone bubbles to the node's `onkey` and to `window`, as for any element.
//!
//! A hot reload restarts the app's script but not the views. A view whose component has a `key` stays
//! alive while the app starts again, and the new tree gets it back when it asks for the same name and key.

use std::collections::HashMap;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use gpui::{AnyView, App, Context, Entity, FocusHandle, Render};
use serde_json::Value;

use crate::io::{CommandError, Sink};
use crate::tree::NodeId;

/// How long a view with a `key` outlives the app that showed it, while the new run of the app has not
/// asked for it yet. Past this, the view is released.
const KEPT_AFTER_RELOAD: Duration = Duration::from_secs(5);

/// A view the app's crate draws in a node of the window.
///
/// The view is a GPUI [`Render`]: what `render` returns is drawn inside the node's box. Its root should fill
/// the box (`size_full()`), and track the view's focus handle if the view takes the keyboard. A view the app
/// shows is cached: it is drawn again only when it calls `cx.notify()`, which the host also does after every
/// `set_prop` and `call`. That is what makes a large view cheap to keep on screen.
pub trait AppElement: Render + Sized + 'static {
    /// The app set a prop, or removed it (`null`). It is called once for each prop that has a value when the
    /// view appears, and after that only for those that change.
    fn set_prop(&mut self, name: &str, value: Value, cx: &mut Context<Self>);

    /// The app called a method on the component. `args` is the JSON array of its arguments. The answer
    /// resolves the app's promise, and an error rejects it, as for a command.
    fn call(&mut self, method: &str, args: Value, cx: &mut Context<Self>) -> Result<Value, CommandError>;

    /// The handle the view takes keys and input-method text through, if it takes focus. The host moves focus
    /// to it when the node is pressed or the app calls `focus()`.
    fn focus_handle(&self, _cx: &App) -> Option<FocusHandle> {
        None
    }
}

/// What the view gets to reach the app with. Cloneable, and usable from any thread.
#[derive(Clone)]
pub struct NativeHost {
    node: Arc<AtomicU32>,
    hub: Arc<Hub>,
}

impl NativeHost {
    /// Fires the `on<name>` listener of the component this view is drawn for, with `detail` as the value the
    /// listener gets. Nothing happens when the app does not listen, and the call never blocks.
    pub fn emit(&self, name: &str, detail: Value) {
        let node = self.node.load(Ordering::Relaxed);
        if node == 0 {
            return;
        }
        let sink = self.hub.sink.lock().unwrap_or_else(PoisonError::into_inner).clone();
        if let Some(sink) = sink {
            sink.element_event(node, name, detail);
        }
    }

    fn attach(&self, node: NodeId) {
        self.node.store(node, Ordering::Relaxed);
    }

    /// The view is not in the tree for now, and its events are dropped.
    fn detach(&self) {
        self.node.store(0, Ordering::Relaxed);
    }
}

/// Where the window's current run of the app takes element events. A reload connects a new sink.
#[derive(Default)]
struct Hub {
    sink: Mutex<Option<Sink>>,
}

/// What the host does with a view, whatever its type.
pub(crate) trait ElementHandle {
    fn set_prop(&self, name: &str, value: Value, cx: &mut App);
    fn call(&self, method: &str, args: Value, cx: &mut App) -> Result<Value, CommandError>;
    fn focus_handle(&self, cx: &App) -> Option<FocusHandle>;
    fn view(&self) -> AnyView;
}

impl<T: AppElement> ElementHandle for Entity<T> {
    fn set_prop(&self, name: &str, value: Value, cx: &mut App) {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            self.update(cx, |element, cx| {
                element.set_prop(name, value, cx);
                cx.notify();
            })
        }));
        if outcome.is_err() {
            eprintln!("svelte-gpui: an element panicked setting the prop {name:?}");
        }
    }

    fn call(&self, method: &str, args: Value, cx: &mut App) -> Result<Value, CommandError> {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            self.update(cx, |element, cx| {
                let answer = element.call(method, args, cx);
                cx.notify();
                answer
            })
        }));
        match outcome {
            Ok(answer) => answer,
            Err(_) => Err(CommandError::with_code("Panicked", format!("the method {method:?} panicked"))),
        }
    }

    fn focus_handle(&self, cx: &App) -> Option<FocusHandle> {
        self.read(cx).focus_handle(cx)
    }

    fn view(&self) -> AnyView {
        self.clone().into()
    }
}

type Factory = Rc<dyn Fn(NativeHost, &mut App) -> Rc<dyn ElementHandle>>;

/// The views an app's crate offers, by name. Built before the app starts and given to the host in
/// `Options::elements`.
///
/// ```ignore
/// let elements = Elements::new().register("counter", |host, cx| cx.new(|cx| Counter::new(host, cx)));
/// svelte_gpui::run(options.elements(elements));
/// ```
#[derive(Clone, Default)]
pub struct Elements {
    factories: HashMap<String, Factory>,
    hub: Arc<Hub>,
}

impl fmt::Debug for Elements {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Elements").field("names", &self.names()).finish()
    }
}

impl Elements {
    pub fn new() -> Self {
        Elements::default()
    }

    /// Registers a view under `name`, which the app turns into a component with `element(name)`. The factory
    /// builds one view each time the app shows the component, and gets the [`NativeHost`] the view emits
    /// events through. A name registered twice, or empty, panics.
    pub fn register<T: AppElement>(
        mut self,
        name: &str,
        factory: impl Fn(NativeHost, &mut App) -> Entity<T> + 'static,
    ) -> Self {
        assert!(!name.is_empty(), "an element needs a name");
        let build: Factory = Rc::new(move |host, cx| {
            let entity: Rc<dyn ElementHandle> = Rc::new(factory(host, cx));
            entity
        });
        let previous = self.factories.insert(name.to_string(), build);
        assert!(previous.is_none(), "the element {name:?} is registered twice");
        self
    }

    /// The registered names, sorted.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.factories.keys().cloned().collect();
        names.sort();
        names
    }

    /// Connects the views' events to this run of the app. The views already alive keep working.
    pub(crate) fn attach(&self, sink: Sink) {
        *self.hub.sink.lock().unwrap_or_else(PoisonError::into_inner) = Some(sink);
    }

    /// A new view for the registered `name`, or `None` when there is no such element.
    pub(crate) fn create(&self, name: &str, key: Option<String>, cx: &mut App) -> Option<Live> {
        let factory = self.factories.get(name)?;
        let host = NativeHost { node: Arc::new(AtomicU32::new(0)), hub: self.hub.clone() };
        let handle = factory(host.clone(), cx);
        Some(Live { handle, host, name: name.to_string(), key, props: HashMap::new() })
    }
}

/// A view that exists, and which node of the tree it is drawn in.
pub(crate) struct Live {
    pub handle: Rc<dyn ElementHandle>,
    host: NativeHost,
    pub name: String,
    pub key: Option<String>,
    /// The props the view has been given, as the JSON text each was sent in.
    pub props: HashMap<String, String>,
}

impl Live {
    /// Draws this view in `node`, and lets its events reach it.
    pub fn attach(&self, node: NodeId) {
        self.host.attach(node);
    }
}

/// The views that exist: those in the tree, and those waiting for a tree to ask for them.
#[derive(Default)]
pub(crate) struct Instances {
    pub live: HashMap<NodeId, Live>,
    /// Views with a `key` that left the tree, with the moment they are released. A tree that asks for the
    /// same name and key before then gets the view back.
    kept: Vec<(Live, Instant)>,
}

impl Instances {
    /// The view in `node` is gone. A view with a key stays for the rest of this pass, so a component
    /// that is rebuilt in the same flush finds it again; one without is released.
    pub fn remove(&mut self, node: NodeId) {
        let live = self.live.remove(&node);
        self.remember(live);
    }

    /// Lets go of a view that left the tree, keeping it for now when it has a key.
    pub fn remember(&mut self, live: Option<Live>) {
        let Some(live) = live else {
            return;
        };
        live.host.detach();
        if live.key.is_some() {
            self.kept.push((live, Instant::now()));
        }
    }

    /// A reload starts: every view leaves the tree, and those with a key wait for the new one.
    pub fn stash_for_reload(&mut self) {
        let deadline = Instant::now() + KEPT_AFTER_RELOAD;
        for (_, live) in std::mem::take(&mut self.live) {
            live.host.detach();
            if live.key.is_some() {
                self.kept.push((live, deadline));
            }
        }
    }

    /// The kept view with this name and key, if there is one.
    pub fn claim(&mut self, name: &str, key: &Option<String>) -> Option<Live> {
        key.as_ref()?;
        let position = self.kept.iter().position(|(live, _)| live.name == name && live.key == *key)?;
        Some(self.kept.remove(position).0)
    }

    /// Releases the kept views whose time is up.
    pub fn expire(&mut self) {
        let now = Instant::now();
        self.kept.retain(|(_, deadline)| *deadline > now);
    }
}
