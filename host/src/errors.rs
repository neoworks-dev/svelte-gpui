//! Errors the app runs into: what threw, where it came from in the app's own files, and the
//! list the window shows them from (see `Workbench::error_overlay`) or logs them to.

use std::path::{Path, PathBuf};

use crate::sourcemap::SourceMap;

/// How many lines of code to show around where an error was thrown.
const CONTEXT_LINES: usize = 2;

/// What the app was doing when the error came out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorKind {
    /// The build failed, so there is no new bundle.
    Build,
    /// The bundle did not compile or threw while it ran, so the app never started.
    Bundle,
    /// An event handler, or the effects the event set off.
    Event,
    /// A timer or an animation frame.
    Timer,
    /// The result of a network, socket, file or command call arriving.
    Io,
    /// A microtask: an effect or something an `await` continued with.
    Task,
    /// A promise that rejected and nothing handled.
    Rejection,
    /// The window asking whether it may close.
    Window,
}

impl ErrorKind {
    pub fn label(self) -> &'static str {
        match self {
            ErrorKind::Build => "The build failed",
            ErrorKind::Bundle => "The app did not start",
            ErrorKind::Event => "Uncaught error while handling an event",
            ErrorKind::Timer => "Uncaught error in a timer or animation frame",
            ErrorKind::Io => "Uncaught error while handling a result",
            ErrorKind::Task => "Uncaught error in an effect or a microtask",
            ErrorKind::Rejection => "Unhandled promise rejection",
            ErrorKind::Window => "Uncaught error in a close handler",
        }
    }
}

/// An error as QuickJS gave it: the message, and the stack with the bundle's positions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub message: String,
    pub stack: String,
}

/// One line of a stack.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub function: String,
    /// `src/App.svelte:12:5`, or the bundle's own position when there is no map for it.
    pub place: String,
    /// Whether it is the app's own code, as opposed to Svelte's or this library's runtime.
    pub in_app: bool,
}

/// A line of the source around where an error was thrown.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeLine {
    pub number: usize,
    pub text: String,
    pub current: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub frames: Vec<Frame>,
    /// The component whose code the error came from, when a frame says.
    pub component: Option<String>,
    /// The source lines around the first frame of the app's own code.
    pub code: Vec<CodeLine>,
    /// How many times this same error happened.
    pub count: u32,
}

impl AppError {
    /// Explains a report: its stack mapped back to the sources when there is a map, the
    /// component, and the code. `base` is what paths are shown relative to.
    pub fn explain(kind: ErrorKind, report: Report, map: Option<&SourceMap>, base: &Path) -> Self {
        let mut frames = Vec::new();
        let mut code = Vec::new();
        let mut component = None;
        for line in report.stack.lines() {
            let Some(frame) = parse_frame(line) else {
                continue;
            };
            let origin = map.zip(frame.position).and_then(|(map, (line, column))| map.origin(line, column));
            let (place, in_app) = match &origin {
                Some(origin) => {
                    (format!("{}:{}:{}", shown(&origin.path, base), origin.line, origin.column), is_app(&origin.path))
                }
                None => (frame.file, false),
            };
            if let (Some(origin), Some(map)) = (&origin, map) {
                if in_app && code.is_empty() {
                    code = code_around(map, origin);
                }
                if component.is_none() && origin.path.extension().is_some_and(|extension| extension == "svelte") {
                    component = origin.path.file_name().map(|name| name.to_string_lossy().to_string());
                }
            }
            frames.push(Frame { function: frame.function, place, in_app });
        }
        AppError { kind, message: report.message, frames, component, code, count: 1 }
    }

    /// Whether `other` is the same error happening again, which is counted instead of listed.
    pub fn repeats(&self, other: &AppError) -> bool {
        self.kind == other.kind && self.message == other.message && self.frames.first() == other.frames.first()
    }

    /// The error as plain text, for the log file.
    pub fn text(&self) -> String {
        let mut text = format!("{}: {}\n", self.kind.label(), self.message);
        if let Some(component) = &self.component {
            text.push_str(&format!("  in {component}\n"));
        }
        for frame in &self.frames {
            text.push_str(&format!("    at {} ({})\n", frame.function, frame.place));
        }
        text
    }
}

/// Every different error so far, with how often each happened.
#[derive(Default)]
pub struct ErrorList {
    pub errors: Vec<AppError>,
    /// How many errors have been added in all, repeats included; it moves whenever the list changes.
    pub total: u64,
    /// Errors not yet written to the log file, one per occurrence.
    pub unlogged: Vec<AppError>,
}

impl ErrorList {
    pub fn add(&mut self, error: AppError) {
        self.total += 1;
        self.unlogged.push(error.clone());
        match self.errors.iter_mut().find(|known| known.repeats(&error)) {
            Some(known) => known.count += 1,
            None => self.errors.push(error),
        }
    }
}

/// A stack line as QuickJS writes it: `    at name (file:line:column)`, or without a name.
struct ParsedFrame {
    function: String,
    file: String,
    position: Option<(usize, usize)>,
}

fn parse_frame(line: &str) -> Option<ParsedFrame> {
    let rest = line.trim().strip_prefix("at ")?;
    let (function, location) = match rest.rsplit_once(" (") {
        Some((function, location)) => (function.to_string(), location.trim_end_matches(')')),
        None => ("<anonymous>".to_string(), rest),
    };
    let mut parts = location.rsplitn(3, ':');
    let column = parts.next().and_then(|part| part.parse::<usize>().ok());
    let line_number = parts.next().and_then(|part| part.parse::<usize>().ok());
    let file = parts.next().unwrap_or(location).to_string();
    let position = line_number.zip(column);
    let file = if position.is_some() { file } else { location.to_string() };
    // A bundle kept as bytecode has no line tables, so its frames name the function and no place.
    let file = if file == "<null>" { "bundle without line numbers".to_string() } else { file };
    Some(ParsedFrame { function, file, position })
}

/// Whether a source file is the app's own, as opposed to a package or this library's runtime.
fn is_app(path: &Path) -> bool {
    let text = path.to_string_lossy();
    !text.contains("/node_modules/") && !text.contains("/svelte-gpui/runtime/")
}

/// A path as the overlay shows it: relative to the directory the host was started in when it
/// is under it, and from the last `node_modules` on when it is a package's.
fn shown(path: &Path, base: &Path) -> String {
    let text = path.to_string_lossy();
    if let Some((_, package)) = text.rsplit_once("/node_modules/") {
        return format!("node_modules/{package}");
    }
    match path.strip_prefix(base) {
        Ok(relative) => relative.to_string_lossy().to_string(),
        Err(_) => text.to_string(),
    }
}

fn code_around(map: &SourceMap, origin: &crate::sourcemap::Origin) -> Vec<CodeLine> {
    let first = origin.line.saturating_sub(CONTEXT_LINES).max(1);
    (first..=origin.line + CONTEXT_LINES)
        .filter_map(|number| {
            let text = map.source_line(&origin.path, number)?;
            Some(CodeLine { number, text, current: number == origin.line })
        })
        .collect()
}

/// Where errors are logged when the window does not show them: a file in the cache directory,
/// named after the app.
pub fn log_path(app: &str) -> PathBuf {
    let cache = match std::env::var_os("XDG_CACHE_HOME") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache"),
    };
    cache.join("svelte-gpui").join(app).join("errors.log")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_frame_with_and_without_a_name() {
        let named = parse_frame("    at press (bundle:12:34)").expect("a frame");
        assert_eq!((named.function.as_str(), named.file.as_str(), named.position), ("press", "bundle", Some((12, 34))));
        let anonymous = parse_frame("    at bundle:5:6").expect("a frame");
        assert_eq!((anonymous.function.as_str(), anonymous.position), ("<anonymous>", Some((5, 6))));
        let native = parse_frame("    at map (native)").expect("a frame");
        assert_eq!((native.file.as_str(), native.position), ("native", None));
        assert!(parse_frame("not a frame").is_none());
    }

    #[test]
    fn counts_the_same_error_once() {
        let report = Report { message: "boom".into(), stack: "    at a (bundle:1:1)".into() };
        let first = AppError::explain(ErrorKind::Event, report, None, Path::new("/"));
        let mut list = ErrorList::default();
        list.add(first.clone());
        list.add(first);
        assert_eq!(list.errors.len(), 1);
        assert_eq!((list.errors[0].count, list.total, list.unlogged.len()), (2, 2, 2));
    }
}
