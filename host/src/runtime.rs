//! Runs the compiled Svelte app in QuickJS. The app talks to the host through
//! one function, `host.apply(batch)`, and the host talks back through two
//! globals the app installs: `__event` for input and `__pump` for the clock.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rquickjs::{
    ArrayBuffer, CaughtError, Coerced, Context, Function, Object, Runtime, function::Opt, function::Rest, qjs,
};

use crate::clipboard::ClipboardImage;
use crate::errors::{AppError, ErrorKind, ErrorList, Report};
use crate::field::Field;
use crate::io::{Commands, Completion, Io};
use crate::sourcemap::SourceMap;
use crate::tree::{NodeId, Tree};
use crate::window_control::{WindowCommand, WindowSettings, WindowState};

/// What the app did with an event: whether it cancelled the default action.
#[derive(Default)]
pub struct Outcome {
    pub prevented: bool,
}

pub struct App {
    /// The moment the app started; its clock counts milliseconds from here.
    pub started: std::time::Instant,
    runtime: Runtime,
    context: Context,
    pub tree: Rc<RefCell<Tree>>,
    /// What the system clipboard held when the host last looked, for `navigator.clipboard`.
    pub clipboard: Rc<RefCell<String>>,
    /// The image the system clipboard held then, if any, for `clipboard.readImage()`.
    pub clipboard_image: Rc<RefCell<Option<ClipboardImage>>>,
    /// The text fields in the window, which the window keeps up to date and `host.fieldState` reads.
    pub fields: Rc<RefCell<HashMap<NodeId, Field>>>,
    /// Operations applied and batches received, for the stress report.
    pub counters: Rc<RefCell<Counters>>,
    /// When each recent frame was drawn and how long building it took, for `frameStats()`.
    pub frames: Rc<RefCell<std::collections::VecDeque<(std::time::Instant, f64)>>>,
    /// Network, sockets and files, which run off the main thread.
    pub io: Rc<Io>,
    /// What the app can call in Rust, kept so a reload starts the new run with the same.
    pub commands: Commands,
    /// Promises that rejected with no handler yet, reported once the microtasks have run.
    rejections: Rc<RefCell<Vec<(usize, Report)>>>,
    /// Why the bundle did not run to the end, when it did not: the error it threw, or
    /// that it could not be compiled.
    pub load_error: Option<String>,
    /// Every error the app has run into, for the window to show or log.
    pub errors: Rc<RefCell<ErrorList>>,
    /// The map from the bundle back to the app's sources, read when the first error needs it.
    source_map: RefCell<Option<Option<Rc<SourceMap>>>>,
    /// Where the map for this bundle is, beside it.
    map_path: std::path::PathBuf,
    /// What the app asked for with `start(..., { window })`, read before the window opens.
    pub window_settings: Rc<RefCell<Option<WindowSettings>>>,
    /// Requests the app made of the window, taken by `Workbench::run_window_commands`.
    pub window_commands: Rc<RefCell<Vec<WindowCommand>>>,
}

#[derive(Default)]
pub struct Counters {
    pub batches: u32,
    pub operations: u32,
}

/// Where the app's script comes from.
#[derive(Clone, Debug)]
pub enum Script {
    /// A bundle file. It is compiled once, without its source text and line tables, and kept
    /// beside the bundle as bytecode, which is loaded from then on. That skips parsing and
    /// keeps tens of megabytes of text out of memory; stack traces then lack line numbers,
    /// so set `SVELTE_GPUI_DEBUG` to run the text itself.
    File(String),
    /// A bundle file run from its text, which keeps the line tables, so an error's stack names
    /// positions in the bundle that a source map can take back to the app's files. It is not
    /// kept as bytecode: this is for development, where the bundle changes all the time.
    Development(String),
    /// Source text, such as an embedded bundle packed without compiling it.
    Source(&'static [u8]),
    /// Bytecode made by [`compile`], such as an embedded bundle.
    Bytecode(&'static [u8]),
}

fn run_bundle(ctx: &rquickjs::Ctx<'_>, script: &Script) -> rquickjs::Result<()> {
    match script {
        Script::File(path) => run_file(ctx, path),
        Script::Development(path) => run_text(ctx, std::fs::read(path).expect("read the bundle")),
        Script::Source(bytes) => ctx.eval::<(), _>(*bytes),
        Script::Bytecode(bytes) => load_bytecode(ctx, bytes),
    }
}

fn run_file(ctx: &rquickjs::Ctx<'_>, bundle_path: &str) -> rquickjs::Result<()> {
    if std::env::var("SVELTE_GPUI_DEBUG").is_ok() {
        return run_text(ctx, std::fs::read(bundle_path).expect("read the bundle"));
    }
    let cache_path = format!("{bundle_path}.qbc");
    let fresh = match (std::fs::metadata(bundle_path), std::fs::metadata(&cache_path)) {
        (Ok(source), Ok(cache)) => match (source.modified(), cache.modified()) {
            (Ok(source), Ok(cache)) => cache >= source,
            _ => false,
        },
        _ => false,
    };
    if fresh {
        if let Ok(bytes) = std::fs::read(&cache_path) {
            return load_bytecode(ctx, &bytes);
        }
    }
    let source = std::fs::read_to_string(bundle_path).expect("read the bundle");
    let compiled = compile_source(ctx, &source);
    // The text is done with; the bundle can be tens of megabytes.
    drop(source);
    let Some(bytes) = compiled else {
        return Err(rquickjs::Error::Exception);
    };
    if std::fs::write(&cache_path, &bytes).is_err() {
        eprintln!("svelte-gpui: cannot keep the compiled bundle at {cache_path}");
    }
    // Run what was written, so a stripped bundle behaves the same on every start.
    load_bytecode(ctx, &bytes)
}

/// Runs a bundle from its text. The script is named `bundle` in stack traces.
fn run_text(ctx: &rquickjs::Ctx<'_>, source: Vec<u8>) -> rquickjs::Result<()> {
    let mut options = rquickjs::context::EvalOptions::default();
    options.filename = Some("bundle".to_string());
    ctx.eval_with_options::<(), _>(source, options)
}

/// Compiles a script to bytecode without its source text and line tables. `None` if it does
/// not compile; the error is then pending in `ctx`.
fn compile_source(ctx: &rquickjs::Ctx<'_>, source: &str) -> Option<Vec<u8>> {
    let raw = ctx.as_raw().as_ptr();
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
    let compiled = unsafe { rquickjs::Value::from_raw(ctx.clone(), function) };
    if compiled.is_exception() {
        return None;
    }
    let flags = qjs::JS_WRITE_OBJ_BYTECODE | qjs::JS_WRITE_OBJ_STRIP_SOURCE | qjs::JS_WRITE_OBJ_STRIP_DEBUG;
    let mut length = 0;
    let written = unsafe { qjs::JS_WriteObject(raw, &mut length, compiled.as_raw(), flags as i32) };
    if written.is_null() {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(written, length as usize) }.to_vec();
    unsafe { qjs::js_free(raw, written as *mut std::os::raw::c_void) };
    Some(bytes)
}

/// Reads bytecode made by [`compile_source`] and runs it.
fn load_bytecode(ctx: &rquickjs::Ctx<'_>, bytes: &[u8]) -> rquickjs::Result<()> {
    let raw = ctx.as_raw().as_ptr();
    let flags = qjs::JS_READ_OBJ_BYTECODE as i32;
    let function = unsafe { qjs::JS_ReadObject(raw, bytes.as_ptr(), bytes.len() as _, flags) };
    run_function(ctx, function)
}

/// Compiles the bundle at `source` to the bytecode `Script::Bytecode` runs, and writes it to
/// `output`. This is what packaging embeds; it needs no window.
pub fn compile(source: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    let text = std::fs::read_to_string(source).map_err(|error| format!("cannot read {}: {error}", source.display()))?;
    let runtime = Runtime::new().map_err(|error| error.to_string())?;
    let context = Context::full(&runtime).map_err(|error| error.to_string())?;
    let bytes = context.with(|ctx| {
        let failed = || format!("{} does not compile: {}", source.display(), pending_error(&ctx));
        compile_source(&ctx, &text).ok_or_else(failed)
    })?;
    std::fs::write(output, bytes).map_err(|error| format!("cannot write {}: {error}", output.display()))
}

/// QuickJS's own limit, 256 KB, which is all the app gets where the thread's stack is unknown.
const DEFAULT_STACK_SIZE: usize = 256 * 1024;

/// Stack left to the host and GPUI below the deepest JavaScript frame: what was used before the
/// runtime started, and the host functions the app calls.
const HOST_STACK_RESERVE: usize = 1024 * 1024;

/// How much stack JavaScript may use. Svelte mounts a component's children while the parent's
/// effect is still running, so each level of nesting costs about 60 KB of stack: QuickJS's 256 KB
/// fit only 15. The app runs on the main thread, whose stack is the process's stack limit (8 MB
/// on most systems, about 110 levels), so it gets all of that but `HOST_STACK_RESERVE`.
fn javascript_stack_size() -> usize {
    const MOST: usize = 64 * 1024 * 1024;
    #[cfg(unix)]
    {
        let mut limit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        if unsafe { libc::getrlimit(libc::RLIMIT_STACK, &mut limit) } != 0 {
            return DEFAULT_STACK_SIZE;
        }
        let stack = if limit.rlim_cur == libc::RLIM_INFINITY { MOST } else { (limit.rlim_cur as usize).min(MOST) };
        stack.saturating_sub(HOST_STACK_RESERVE).max(DEFAULT_STACK_SIZE)
    }
    // Windows gives the main thread 1 MB, so QuickJS's default is already a quarter of it.
    #[cfg(not(unix))]
    DEFAULT_STACK_SIZE
}

fn pending_error(ctx: &rquickjs::Ctx<'_>) -> String {
    CaughtError::from_error(ctx, rquickjs::Error::Exception).to_string()
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
    /// Evaluates the script, which mounts the component and sends its first batch.
    /// `bundle_path` is a file in the bundle's directory (the script's own, for a file).
    /// `commands` is what the app can call in Rust.
    pub fn start_with(bundle_path: &str, script: Script, allow_files: &[String], commands: Commands) -> Self {
        let runtime = Runtime::new().expect("quickjs runtime");
        runtime.set_max_stack_size(javascript_stack_size());
        // A promise nobody catches would otherwise fail without a word. QuickJS reports a
        // rejection the moment it happens, and again as handled if a handler attaches
        // later, so rejections are only reported once the microtasks have run (`report`).
        let rejections: Rc<RefCell<Vec<(usize, Report)>>> = Rc::new(RefCell::new(Vec::new()));
        let tracked = rejections.clone();
        runtime.set_host_promise_rejection_tracker(Some(Box::new(move |_ctx, promise, reason, handled| {
            let key = unsafe { qjs::JS_VALUE_GET_PTR(promise.as_raw()) } as usize;
            if handled {
                tracked.borrow_mut().retain(|(pending, _)| *pending != key);
                return;
            }
            let message =
                reason.get::<Coerced<String>>().map(|value| value.0).unwrap_or_else(|_| "an error".to_string());
            let stack = reason
                .as_exception()
                .and_then(|exception| exception.stack())
                .or_else(|| {
                    reason
                        .as_object()
                        .and_then(|object| object.get::<_, Coerced<String>>("stack").ok())
                        .map(|value| value.0)
                })
                .unwrap_or_default();
            tracked.borrow_mut().push((key, Report { message, stack }));
        })));
        let context = Context::full(&runtime).expect("quickjs context");
        let tree = Rc::new(RefCell::new(Tree::new()));
        let counters = Rc::new(RefCell::new(Counters::default()));
        let clipboard = Rc::new(RefCell::new(String::new()));
        let mut app = Self {
            frames: Rc::new(RefCell::new(std::collections::VecDeque::new())),
            started: std::time::Instant::now(),
            runtime,
            context,
            tree,
            clipboard,
            clipboard_image: Rc::new(RefCell::new(None)),
            fields: Rc::new(RefCell::new(HashMap::new())),
            counters,
            io: Rc::new(Io::new(Some(bundle_path), allow_files, commands.clone())),
            commands,
            rejections,
            load_error: None,
            errors: Rc::new(RefCell::new(ErrorList::default())),
            source_map: RefCell::new(None),
            map_path: std::path::PathBuf::from(format!("{bundle_path}.map")),
            window_settings: Rc::new(RefCell::new(None)),
            window_commands: Rc::new(RefCell::new(Vec::new())),
        };
        app.install_host();
        let load_error = app.context.with(|ctx| match run_bundle(&ctx, &script) {
            Ok(()) => None,
            Err(error) => Some(caught(&ctx, error)),
        });
        app.load_error = load_error.as_ref().map(|report| report.message.clone());
        if let Some(report) = load_error {
            app.record(ErrorKind::Bundle, report);
        }
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
        let fields = self.fields.clone();
        let started = self.started;
        self.context.with(|ctx| {
            let host = Object::new(ctx.clone()).expect("host object");
            let measure =
                Function::new(ctx.clone(), |size: f64, text: String| -> Vec<f64> { measure_text(size as f32, &text) })
                    .expect("measureText function");
            host.set("measureText", measure).expect("set measureText");
            let now =
                Function::new(ctx.clone(), move || started.elapsed().as_secs_f64() * 1000.0).expect("now function");
            host.set("now", now).expect("set now");
            let frames = self.frames.clone();
            let frame_stats = Function::new(ctx.clone(), move || -> Vec<f64> {
                let mut frames = frames.borrow_mut();
                let now = std::time::Instant::now();
                while frames.front().is_some_and(|(at, _)| now.duration_since(*at).as_secs_f64() > 1.0) {
                    frames.pop_front();
                }
                let count = frames.len() as f64;
                let building = frames.iter().map(|(_, milliseconds)| milliseconds).sum::<f64>() / count.max(1.0);
                vec![count, building]
            })
            .expect("frameStats function");
            host.set("frameStats", frame_stats).expect("set frameStats");
            let read_clipboard =
                Function::new(ctx.clone(), move || clipboard.borrow().clone()).expect("clipboard function");
            host.set("clipboardText", read_clipboard).expect("set clipboardText");
            let clipboard_image = self.clipboard_image.clone();
            let read_image = Function::new(ctx.clone(), move || -> Vec<String> {
                match &*clipboard_image.borrow() {
                    Some(image) => vec![image.path.clone(), image.mime.to_string()],
                    None => Vec::new(),
                }
            })
            .expect("clipboardImage function");
            host.set("clipboardImage", read_image).expect("set clipboardImage");
            // A field's selection and where its cursor sits among the rows, as it is right now:
            // start, end, row and row count, or nothing for an element that is not a drawn field.
            let field_state = Function::new(ctx.clone(), move |id: f64| -> Vec<f64> {
                let Ok(fields) = fields.try_borrow() else {
                    return Vec::new();
                };
                let Some(field) = fields.get(&(id as NodeId)) else {
                    return Vec::new();
                };
                let (start, end) = field.selection_utf16();
                vec![start as f64, end as f64, field.caret_row.0 as f64, field.caret_row.1 as f64]
            })
            .expect("fieldState function");
            host.set("fieldState", field_state).expect("set fieldState");
            let trace = std::env::var_os("SVELTE_GPUI_TRACE_OPS").is_some();
            let apply = Function::new(ctx.clone(), move |batch: String| {
                if trace {
                    eprintln!("batch: {batch}");
                }
                let operations = tree.borrow_mut().apply(&batch);
                let mut counters = counters.borrow_mut();
                counters.batches += 1;
                counters.operations += operations as u32;
            })
            .expect("apply function");
            host.set("apply", apply).expect("set apply");
            let io = self.io.clone();
            let request =
                Function::new(ctx.clone(), move |op: String, call: f64, args: String, bytes: Opt<rquickjs::Value>| {
                    let args = serde_json::from_str(&args).unwrap_or(serde_json::Value::Null);
                    // Copied before any script runs again, which could detach the buffer.
                    let bytes = bytes
                        .0
                        .and_then(|value| ArrayBuffer::from_value(value))
                        .and_then(|buffer| unsafe { buffer.as_bytes().map(<[u8]>::to_vec) });
                    io.request(&op, call as u64, args, bytes);
                })
                .expect("io function");
            host.set("io", request).expect("set io");
            let settings = self.window_settings.clone();
            let window_options = Function::new(ctx.clone(), move |json: String| {
                let parsed = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
                *settings.borrow_mut() = Some(WindowSettings::from_json(&parsed));
            })
            .expect("windowOptions function");
            host.set("windowOptions", window_options).expect("set windowOptions");
            let commands = self.window_commands.clone();
            let window_command = Function::new(ctx.clone(), move |op: String, args: String| {
                let parsed = serde_json::from_str(&args).unwrap_or(serde_json::Value::Null);
                match WindowCommand::parse(&op, &parsed) {
                    Some(command) => commands.borrow_mut().push(command),
                    None => eprintln!("svelte-gpui: unknown window operation {op:?}"),
                }
            })
            .expect("window function");
            host.set("window", window_command).expect("set window");
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
        loop {
            match self.runtime.execute_pending_job() {
                Ok(true) => {}
                Ok(false) => break,
                // A job that threw is not the end of the others.
                Err(job) => {
                    let report = job.0.with(|ctx| caught(&ctx, rquickjs::Error::Exception));
                    self.record(ErrorKind::Task, report);
                }
            }
        }
        let rejected: Vec<_> = self.rejections.borrow_mut().drain(..).collect();
        for (_, report) in rejected {
            self.record(ErrorKind::Rejection, report);
        }
    }

    /// Notes an error the app ran into: it is printed, and added to the list the window shows
    /// or logs, with its stack mapped back to the app's sources.
    fn record(&self, kind: ErrorKind, report: Report) {
        let map = self.source_map();
        let base = std::env::current_dir().unwrap_or_default();
        let error = AppError::explain(kind, report, map.as_deref(), &base);
        eprintln!("svelte-gpui: {}", error.text());
        self.errors.borrow_mut().add(error);
    }

    /// The map for the bundle, if the build made one beside it.
    fn source_map(&self) -> Option<Rc<SourceMap>> {
        let mut loaded = self.source_map.borrow_mut();
        loaded
            .get_or_insert_with(|| {
                let text = std::fs::read_to_string(&self.map_path).ok()?;
                let directory = self.map_path.parent().unwrap_or(std::path::Path::new("."));
                SourceMap::parse(&text, directory).map(Rc::new)
            })
            .clone()
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

    /// Calls into the app. An error the call throws is recorded as `kind`, not passed on, and
    /// the call then answers `None`, as it does when the app has not installed `name`.
    fn call_app<R>(
        &self,
        kind: ErrorKind,
        name: &str,
        call: impl for<'js> FnOnce(Function<'js>, &rquickjs::Ctx<'js>) -> rquickjs::Result<R>,
    ) -> Option<R> {
        let outcome = self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>(name) else {
                return Ok(None);
            };
            match call(function, &ctx) {
                Ok(value) => Ok(Some(value)),
                Err(error) => Err(caught(&ctx, error)),
            }
        });
        match outcome {
            Ok(value) => value,
            Err(report) => {
                self.record(kind, report);
                None
            }
        }
    }

    /// Reports an event on a node to the app, and says what the app made of it.
    pub fn event(&self, id: NodeId, kind: &str, detail: &str) -> Outcome {
        let json = self.call_app(ErrorKind::Event, "__event", |function, _| {
            function.call::<_, String>((id, kind.to_string(), detail.to_string()))
        });
        self.after_call();
        json.map(|json| parse_outcome(&json)).unwrap_or_default()
    }

    /// Settles a call the app made through `host.io`, or passes on what a stream produced.
    pub fn deliver(&self, completion: Completion) {
        self.call_app(ErrorKind::Io, "__io", |function, ctx| {
            let bytes = match completion.bytes {
                Some(bytes) => match ArrayBuffer::new(ctx.clone(), bytes) {
                    Ok(buffer) => buffer.into_value(),
                    Err(_) => rquickjs::Value::new_undefined(ctx.clone()),
                },
                None => rquickjs::Value::new_undefined(ctx.clone()),
            };
            let arguments = (completion.call as f64, completion.settle.name(), completion.json, bytes);
            function.call::<_, ()>(arguments)
        });
        self.after_call();
    }

    /// Tells the app the window's state: its size, which `window.innerWidth` and `resize`
    /// report, and what `window` from the library keeps current.
    pub fn window(&self, state: &WindowState) {
        let arguments = (
            state.width as f64,
            state.height as f64,
            state.scale as f64,
            state.focused,
            state.maximized,
            state.fullscreen,
        );
        self.call_app(ErrorKind::Event, "__window", |function, _| function.call::<_, ()>(arguments));
        self.after_call();
    }

    /// Asks the app whether the window may close now. `false` when a handler refused, or
    /// when one is still deciding: the app closes the window itself once it has said yes.
    pub fn close_requested(&self) -> bool {
        let answer = self.call_app(ErrorKind::Window, "__closeRequested", |function, _| function.call::<_, bool>(()));
        self.after_call();
        answer.unwrap_or(true)
    }

    /// Advances the app's clock, which runs due timers and, on a display frame, the
    /// animation frames. Says when the clock next has work.
    pub fn pump(&self, now_milliseconds: f64, frame: bool) -> Wake {
        let answer =
            self.call_app(ErrorKind::Timer, "__pump", |function, _| function.call::<_, f64>((now_milliseconds, frame)));
        self.after_call();
        Wake::from_milliseconds(answer.unwrap_or(-1.0))
    }
}

/// What a thrown value says: its name and message, and the stack.
fn caught(ctx: &rquickjs::Ctx<'_>, error: rquickjs::Error) -> Report {
    match CaughtError::from_error(ctx, error) {
        CaughtError::Exception(exception) => {
            let name = exception.as_object().get::<_, Option<String>>("name").ok().flatten();
            let name = name.unwrap_or_else(|| "Error".to_string());
            let message = exception.message().unwrap_or_default();
            let message = if message.is_empty() { name } else { format!("{name}: {message}") };
            Report { message, stack: exception.stack().unwrap_or_default() }
        }
        CaughtError::Value(value) => {
            let message = value.get::<Coerced<String>>().map(|text| text.0).unwrap_or_else(|_| "an error".to_string());
            Report { message: format!("Uncaught {message}"), stack: String::new() }
        }
        CaughtError::Error(error) => Report { message: error.to_string(), stack: String::new() },
    }
}

/// When the app's clock next has work.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Wake {
    /// On the display's next frame: an animation frame was requested or an animation runs.
    Frame,
    /// After this long, when the first timer is due.
    After(std::time::Duration),
    /// Not until something happens.
    Idle,
}

impl Wake {
    fn from_milliseconds(milliseconds: f64) -> Wake {
        if milliseconds < 0.0 {
            Wake::Idle
        } else if milliseconds == 0.0 {
            Wake::Frame
        } else {
            Wake::After(std::time::Duration::from_secs_f64(milliseconds / 1000.0))
        }
    }
}

fn parse_outcome(json: &str) -> Outcome {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Outcome::default();
    };
    Outcome { prevented: value["prevented"].as_bool().unwrap_or(false) }
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
        let advance =
            face.glyph_index(character).and_then(|glyph| face.glyph_hor_advance(glyph)).unwrap_or((units * 0.6) as u16);
        width += advance as f32 / units * size;
    }
    let ascent = face.ascender() as f32 / units * size;
    let descent = -(face.descender() as f32) / units * size;
    vec![width as f64, ascent as f64, descent as f64]
}
