//! Runs the compiled Svelte app in QuickJS. The app talks to the host through
//! one function, `host.apply(batch)`, and the host talks back through two
//! globals the app installs: `__event` for input and `__pump` for the clock.

use std::cell::RefCell;
use std::rc::Rc;

use rquickjs::{CaughtError, Coerced, Context, Function, Object, Runtime, function::Rest, qjs};

use crate::tree::{NodeId, Tree};

/// What the app did with an event: whether it cancelled the default action, and
/// any text a clipboard handler asked to be copied.
#[derive(Default)]
pub struct Outcome {
    pub prevented: bool,
    pub clipboard: Option<String>,
}

pub struct App {
    /// The link to Grove's main process, when there is one.
    pub bridge: Option<crate::bridge::Bridge>,
    /// The moment the app started; its clock counts milliseconds from here.
    pub started: std::time::Instant,
    runtime: Runtime,
    context: Context,
    pub tree: Rc<RefCell<Tree>>,
    /// What the system clipboard held when the host last looked, for `navigator.clipboard`.
    pub clipboard: Rc<RefCell<String>>,
    /// Operations applied and batches received, for the stress report.
    pub counters: Rc<RefCell<Counters>>,
}

#[derive(Default)]
pub struct Counters {
    pub batches: u32,
    pub operations: u32,
}

/// Runs the bundle. It is compiled once, without its source text and line tables, and kept
/// beside the bundle as bytecode, which is loaded from then on. That skips parsing and keeps
/// tens of megabytes of text out of memory; stack traces then lack line numbers, so set
/// `SVELTE_GPUI_DEBUG` to run the text itself.
fn run_bundle(ctx: &rquickjs::Ctx<'_>, bundle_path: &str) -> rquickjs::Result<()> {
    if std::env::var("SVELTE_GPUI_DEBUG").is_ok() {
        let source = std::fs::read_to_string(bundle_path).expect("read the bundle");
        return ctx.eval::<(), _>(source);
    }
    let cache_path = format!("{bundle_path}.qbc");
    let fresh = match (std::fs::metadata(bundle_path), std::fs::metadata(&cache_path)) {
        (Ok(source), Ok(cache)) => match (source.modified(), cache.modified()) {
            (Ok(source), Ok(cache)) => cache >= source,
            _ => false,
        },
        _ => false,
    };
    let raw = ctx.as_raw().as_ptr();
    if fresh {
        if let Ok(bytes) = std::fs::read(&cache_path) {
            let function = unsafe { qjs::JS_ReadObject(raw, bytes.as_ptr(), bytes.len() as _, qjs::JS_READ_OBJ_BYTECODE as i32) };
            return run_function(ctx, function);
        }
    }
    let source = std::fs::read_to_string(bundle_path).expect("read the bundle");
    let name = std::ffi::CString::new("bundle").expect("name");
    let function = unsafe {
        qjs::JS_Eval(
            raw,
            source.as_ptr() as *const std::os::raw::c_char,
            source.len() as _,
            name.as_ptr(),
            (qjs::JS_EVAL_TYPE_GLOBAL | qjs::JS_EVAL_FLAG_STRICT | qjs::JS_EVAL_FLAG_COMPILE_ONLY) as i32,
        )
    };
    drop(source);
    let compiled = unsafe { rquickjs::Value::from_raw(ctx.clone(), function) };
    if compiled.is_exception() {
        return Err(rquickjs::Error::Exception);
    }
    let flags = qjs::JS_WRITE_OBJ_BYTECODE | qjs::JS_WRITE_OBJ_STRIP_SOURCE | qjs::JS_WRITE_OBJ_STRIP_DEBUG;
    let mut length = 0;
    let written = unsafe { qjs::JS_WriteObject(raw, &mut length, compiled.as_raw(), flags as i32) };
    if !written.is_null() {
        let bytes = unsafe { std::slice::from_raw_parts(written, length as usize) };
        if std::fs::write(&cache_path, bytes).is_err() {
            eprintln!("svelte-gpui: cannot keep the compiled bundle at {cache_path}");
        }
        unsafe { qjs::js_free(raw, written as *mut std::os::raw::c_void) };
    }
    // Run what was written, so a stripped bundle behaves the same on every start.
    drop(compiled);
    match std::fs::read(&cache_path) {
        Ok(bytes) => {
            let function = unsafe { qjs::JS_ReadObject(raw, bytes.as_ptr(), bytes.len() as _, qjs::JS_READ_OBJ_BYTECODE as i32) };
            run_function(ctx, function)
        }
        Err(_) => {
            let source = std::fs::read_to_string(bundle_path).expect("read the bundle");
            ctx.eval::<(), _>(source)
        }
    }
}

/// Runs a compiled script; `function` is consumed.
fn run_function(ctx: &rquickjs::Ctx<'_>, function: qjs::JSValue) -> rquickjs::Result<()> {
    if unsafe { qjs::JS_IsException(function) } {
        return Err(rquickjs::Error::Exception);
    }
    let result = unsafe { qjs::JS_EvalFunction(ctx.as_raw().as_ptr(), function) };
    let finished = unsafe { rquickjs::Value::from_raw(ctx.clone(), result) };
    if finished.is_exception() {
        return Err(rquickjs::Error::Exception);
    }
    Ok(())
}

impl App {
    /// Evaluates the bundle, which mounts the component and sends its first batch.
    pub fn start(bundle_path: &str, bridge: Option<crate::bridge::Bridge>) -> Self {
        let runtime = Runtime::new().expect("quickjs runtime");
        // A promise nobody catches would otherwise fail without a word.
        runtime.set_host_promise_rejection_tracker(Some(Box::new(|_ctx, _promise, reason, handled| {
            if handled {
                return;
            }
            let text = reason
                .get::<Coerced<String>>()
                .map(|value| value.0)
                .unwrap_or_else(|_| "an error".to_string());
            let stack = reason
                .as_exception()
                .and_then(|exception| exception.stack())
                .unwrap_or_default();
            let keys: Vec<String> = reason
                .as_object()
                .map(|object| {
                    object
                        .keys::<String>()
                        .filter_map(|key| key.ok())
                        .collect()
                })
                .unwrap_or_default();
            let fallback = reason
                .as_object()
                .and_then(|object| object.get::<_, Coerced<String>>("stack").ok())
                .map(|value| value.0)
                .unwrap_or_default();
            eprintln!("svelte-gpui: unhandled rejection: {text} keys={keys:?}\n{stack}{fallback}");
        })));
        let context = Context::full(&runtime).expect("quickjs context");
        let tree = Rc::new(RefCell::new(Tree::new()));
        let counters = Rc::new(RefCell::new(Counters::default()));
        let clipboard = Rc::new(RefCell::new(String::new()));
        let app = Self {
            bridge,
            started: std::time::Instant::now(),
            runtime,
            context,
            tree,
            clipboard,
            counters,
        };
        app.install_host();
        app.context.with(|ctx| {
            if let Err(error) = run_bundle(&ctx, bundle_path) {
                eprintln!("svelte-gpui: the bundle failed: {}", CaughtError::from_error(&ctx, error));
            }
        });
        app.drain();
        // The parser and the bundle's text are done with; give the memory back.
        unsafe {
            libc::malloc_trim(0);
        }
        app
    }

    fn install_host(&self) {
        let tree = self.tree.clone();
        let counters = self.counters.clone();
        let clipboard = self.clipboard.clone();
        let started = self.started;
        let bridge = self.bridge.clone();
        self.context.with(|ctx| {
            let host = Object::new(ctx.clone()).expect("host object");
            let send = Function::new(ctx.clone(), move |line: String| {
                if let Some(bridge) = &bridge {
                    bridge.send(&line);
                }
            })
            .expect("ipc function");
            host.set("ipc", send).expect("set ipc");
            let measure = Function::new(ctx.clone(), |size: f64, text: String| -> Vec<f64> {
                measure_text(size as f32, &text)
            })
            .expect("measureText function");
            host.set("measureText", measure).expect("set measureText");
            let now = Function::new(ctx.clone(), move || started.elapsed().as_secs_f64() * 1000.0)
                .expect("now function");
            host.set("now", now).expect("set now");
            let read_clipboard = Function::new(ctx.clone(), move || clipboard.borrow().clone())
                .expect("clipboard function");
            host.set("clipboardText", read_clipboard).expect("set clipboardText");
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

    /// Where the app's memory went, by kind, for the report.
    pub fn memory_breakdown(&self) -> String {
        let usage = self.runtime.memory_usage();
        let megabytes = |bytes: i64| bytes as f64 / 1_048_576.0;
        format!(
            "quickjs: {:.1} MB held; functions {:.1} MB (code {:.1}, line tables {:.1}), strings {:.1}, objects {:.1}, properties {:.1}, shapes {:.1}, atoms {:.1}",
            megabytes(usage.malloc_size),
            megabytes(usage.js_func_size),
            megabytes(usage.js_func_code_size),
            megabytes(usage.js_func_pc2line_size),
            megabytes(usage.str_size),
            megabytes(usage.obj_size),
            megabytes(usage.prop_size),
            megabytes(usage.shape_size),
            megabytes(usage.atom_size),
        )
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

    /// Reports an event on a node to the app, and says what the app made of it.
    pub fn event(&self, id: NodeId, kind: &str, detail: &str) -> Outcome {
        let mut outcome = Outcome::default();
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>("__event") else {
                return;
            };
            let result = function.call::<_, String>((id, kind.to_string(), detail.to_string()));
            match result {
                Ok(json) => outcome = parse_outcome(&json),
                Err(error) => {
                    eprintln!("svelte-gpui: __event failed: {}", CaughtError::from_error(&ctx, error));
                }
            }
        });
        self.after_call();
        outcome
    }

    /// Hands the app a line from the main process: a reply, or an event.
    pub fn ipc(&self, line: &str) {
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>("__ipc") else {
                return;
            };
            if let Err(error) = function.call::<_, ()>((line.to_string(),)) {
                eprintln!("svelte-gpui: __ipc failed: {}", CaughtError::from_error(&ctx, error));
            }
        });
        self.after_call();
    }

    /// Tells the app how big the host measured an element it asked about.
    pub fn size(&self, id: NodeId, left: f32, top: f32, width: f32, height: f32) {
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>("__size") else {
                return;
            };
            if let Err(error) = function.call::<_, ()>((id, left as f64, top as f64, width as f64, height as f64)) {
                eprintln!("svelte-gpui: __size failed: {}", CaughtError::from_error(&ctx, error));
            }
        });
        self.after_call();
    }

    /// Tells the app the window's size, which `window.innerWidth` and `resize` report.
    pub fn window(&self, width: f32, height: f32) {
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>("__window") else {
                return;
            };
            if let Err(error) = function.call::<_, ()>((width as f64, height as f64)) {
                eprintln!("svelte-gpui: __window failed: {}", CaughtError::from_error(&ctx, error));
            }
        });
        self.after_call();
    }

    /// Advances the app's clock, which runs due timers and animation frames. Says whether
    /// anything is still waiting on the clock.
    pub fn pump(&self, now_milliseconds: f64) -> bool {
        let mut busy = false;
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>("__pump") else {
                return;
            };
            match function.call::<_, bool>((now_milliseconds,)) {
                Ok(value) => busy = value,
                Err(error) => {
                    eprintln!("svelte-gpui: __pump failed: {}", CaughtError::from_error(&ctx, error));
                }
            }
        });
        self.after_call();
        busy
    }
}

fn parse_outcome(json: &str) -> Outcome {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Outcome::default();
    };
    Outcome {
        prevented: value["prevented"].as_bool().unwrap_or(false),
        clipboard: value["clipboard"].as_str().map(String::from),
    }
}

/// Width, ascent and descent of some text in the bundled monospace font, for code that
/// measures a cell before it draws. The same face the grid is drawn with.
fn measure_text(size: f32, text: &str) -> Vec<f64> {
    let bytes: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
    let Ok(face) = ttf_parser::Face::parse(bytes, 0) else {
        return vec![(size * 0.6 * text.chars().count() as f32) as f64, (size * 0.8) as f64, (size * 0.2) as f64];
    };
    let units = face.units_per_em() as f32;
    let mut width = 0.0;
    for character in text.chars() {
        let advance = face
            .glyph_index(character)
            .and_then(|glyph| face.glyph_hor_advance(glyph))
            .unwrap_or((units * 0.6) as u16);
        width += advance as f32 / units * size;
    }
    let ascent = face.ascender() as f32 / units * size;
    let descent = -(face.descender() as f32) / units * size;
    vec![width as f64, ascent as f64, descent as f64]
}
