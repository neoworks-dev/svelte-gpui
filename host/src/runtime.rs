//! Runs the compiled Svelte app in QuickJS. The app talks to the host through
//! one function, `host.apply(batch)`, and the host talks back through two
//! globals the app installs: `__event` for input and `__pump` for the clock.

use std::cell::RefCell;
use std::rc::Rc;

use rquickjs::{CaughtError, Coerced, Context, Function, Object, Runtime, function::Rest};

use crate::tree::{NodeId, Tree};

pub struct App {
    runtime: Runtime,
    context: Context,
    pub tree: Rc<RefCell<Tree>>,
    /// Operations applied and batches received, for the stress report.
    pub counters: Rc<RefCell<Counters>>,
}

#[derive(Default)]
pub struct Counters {
    pub batches: u32,
    pub operations: u32,
}

impl App {
    /// Evaluates the bundle, which mounts the component and sends its first batch.
    pub fn start(bundle: &str) -> Self {
        let runtime = Runtime::new().expect("quickjs runtime");
        let context = Context::full(&runtime).expect("quickjs context");
        let tree = Rc::new(RefCell::new(Tree::new()));
        let counters = Rc::new(RefCell::new(Counters::default()));
        let app = Self {
            runtime,
            context,
            tree,
            counters,
        };
        app.install_host();
        app.context.with(|ctx| {
            if let Err(error) = ctx.eval::<(), _>(bundle) {
                eprintln!("svelte-gpui: the bundle failed: {}", CaughtError::from_error(&ctx, error));
            }
        });
        app.drain();
        app
    }

    fn install_host(&self) {
        let tree = self.tree.clone();
        let counters = self.counters.clone();
        self.context.with(|ctx| {
            let host = Object::new(ctx.clone()).expect("host object");
            let apply = Function::new(ctx.clone(), move |batch: String| {
                let operations = tree.borrow_mut().apply(&batch);
                let mut counters = counters.borrow_mut();
                counters.batches += 1;
                counters.operations += operations as u32;
            })
            .expect("apply function");
            host.set("apply", apply).expect("set apply");
            ctx.globals().set("host", host).expect("set host");

            let console = Object::new(ctx.clone()).expect("console object");
            for name in ["log", "info", "warn", "error", "debug"] {
                let print = Function::new(ctx.clone(), move |arguments: Rest<Coerced<String>>| {
                    let words: Vec<String> = arguments.0.into_iter().map(|word| word.0).collect();
                    eprintln!("[{name}] {}", words.join(" "));
                })
                .expect("console function");
                console.set(name, print).expect("set console");
            }
            ctx.globals().set("console", console).expect("set console");
        });
    }

    /// Runs queued promise jobs until there are none: Svelte schedules work as microtasks.
    fn drain(&self) {
        while self.runtime.execute_pending_job().unwrap_or(false) {}
    }

    /// What follows every call into the app: its microtasks, then a flush of what they changed.
    fn after_call(&self) {
        self.drain();
        self.settle();
    }

    /// Flushes whatever the microtasks changed, which the JS side only sends on request.
    fn settle(&self) {
        self.context.with(|ctx| {
            let function: Result<Function, _> = ctx.globals().get("__settle");
            if let Ok(function) = function {
                let _ = function.call::<_, ()>(());
            }
        });
    }

    /// Reports an event on a node to the app.
    pub fn event(&self, id: NodeId, kind: &str, detail: &str) {
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>("__event") else {
                return;
            };
            if let Err(error) = function.call::<_, ()>((id, kind.to_string(), detail.to_string())) {
                eprintln!("svelte-gpui: __event failed: {}", CaughtError::from_error(&ctx, error));
            }
        });
        self.after_call();
    }

    /// Advances the app's clock, which runs due timers and animation frames.
    pub fn pump(&self, now_milliseconds: f64) {
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>("__pump") else {
                return;
            };
            if let Err(error) = function.call::<_, ()>((now_milliseconds,)) {
                eprintln!("svelte-gpui: __pump failed: {}", CaughtError::from_error(&ctx, error));
            }
        });
        self.after_call();
    }
}
