//! The file dialogs of a host that has no window: `files.pickFiles` and `files.pickSavePath`, which the
//! window shows with GPUI, shown by the desktop instead. On Linux that is the file chooser of
//! xdg-desktop-portal, over D-Bus, through `ashpd`, which GPUI uses for its own dialogs. It needs no window of
//! ours: the portal's backend (GTK, KDE, …) draws it. Elsewhere there is no chooser to ask yet, and a call
//! says so.
//!
//! A dialog runs on a thread of its own and answers through the [`Sink`], like any other `Io` work, so the
//! session never waits for the user. What the user picks is allowed for the rest of the session, as it is in the
//! window, and a dialog the user cancels resolves with `null`, as it does there.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::io::{Io, MainJob, Sink};

/// What `files.pickFiles` asked for.
#[derive(Debug, PartialEq)]
pub(super) struct OpenRequest {
    pub directories: bool,
    pub multiple: bool,
    /// The text of the confirm button.
    pub prompt: Option<String>,
    pub filters: Vec<Filter>,
}

/// What `files.pickSavePath` asked for.
#[derive(Debug, PartialEq)]
pub(super) struct SaveRequest {
    /// Where the dialog starts, as the app wrote it (it may start with `~` or a placeholder).
    pub directory: Option<String>,
    /// The file name it suggests.
    pub name: Option<String>,
    pub filters: Vec<Filter>,
}

/// A group of files the dialog offers to show, such as images: a label and the extensions in it.
#[derive(Debug, PartialEq)]
pub(super) struct Filter {
    pub name: String,
    /// Glob patterns, such as `*.png`.
    pub patterns: Vec<String>,
}

impl OpenRequest {
    pub fn from_args(args: &Value) -> Self {
        OpenRequest {
            directories: args["directories"].as_bool().unwrap_or(false),
            multiple: args["multiple"].as_bool().unwrap_or(false),
            prompt: args["prompt"].as_str().map(str::to_string),
            filters: filters_of(&args["filters"]),
        }
    }

    pub fn title(&self) -> &'static str {
        if self.directories { "Open Folder" } else { "Open File" }
    }
}

impl SaveRequest {
    pub fn from_args(args: &Value) -> Self {
        SaveRequest {
            directory: args["directory"].as_str().map(str::to_string),
            name: args["name"].as_str().map(str::to_string),
            filters: filters_of(&args["filters"]),
        }
    }
}

/// `{ name, extensions }` entries as filters. One without extensions, or whose extensions are not usable, is left out.
fn filters_of(value: &Value) -> Vec<Filter> {
    let mut filters = Vec::new();
    for entry in value.as_array().into_iter().flatten() {
        let patterns: Vec<String> = entry["extensions"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|ext| ext.as_str().and_then(glob_of))
            .collect();
        if patterns.is_empty() {
            continue;
        }
        let name = entry["name"].as_str().filter(|name| !name.is_empty()).map(str::to_string);
        let name = name.unwrap_or_else(|| patterns.join(", "));
        filters.push(Filter { name, patterns });
    }
    filters
}

/// The pattern for an extension written as `png`, `.png` or `*.png`. `None` for text that is not one.
fn glob_of(extension: &str) -> Option<String> {
    let bare = extension.trim().trim_start_matches('*').trim_start_matches('.');
    let usable = !bare.is_empty() && bare.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'));
    usable.then(|| format!("*.{bare}"))
}

/// How a dialog ended: with what the user chose, or by being cancelled; or with a message for why it could not be shown.
type Chosen<T> = Result<Option<T>, String>;

/// Starts the dialogs the session queued. Each answers its call from a thread of its own.
pub(super) fn run(io: &Io) {
    let jobs = std::mem::take(&mut *io.main_jobs.borrow_mut());
    for job in jobs {
        let call = job.call;
        let sink = io.sink();
        let files = io.files.clone();
        let started = std::thread::Builder::new().name(format!("dialog-{call}")).spawn({
            let sink = sink.clone();
            move || {
                let directories = files.directories().clone();
                answer(&sink, job, &directories.home, &|path| directories.expand(path), &|path| {
                    files.grant_picked(path)
                });
            }
        });
        if started.is_err() {
            sink.reject(call, "could not start a thread for the dialog");
        }
    }
}

/// Shows the dialog a job asked for, and settles its call with what came of it. `expand` makes a path the app
/// wrote absolute, and `grant` allows what the user picked, which the app may read from then on.
fn answer(sink: &Sink, job: MainJob, home: &Path, expand: &dyn Fn(&str) -> PathBuf, grant: &dyn Fn(&Path)) {
    let MainJob { op, call, args } = job;
    match op.as_str() {
        "fs.pickFiles" => match portal::open(&OpenRequest::from_args(&args)) {
            Ok(Some(paths)) => {
                paths.iter().for_each(|path| grant(path));
                let paths: Vec<_> = paths.iter().map(|path| path.to_string_lossy()).collect();
                sink.resolve(call, json!({ "paths": paths }));
            }
            Ok(None) => sink.resolve(call, json!({ "paths": null })),
            Err(message) => sink.reject_with_code(call, "Other", message),
        },
        "fs.pickSavePath" => {
            let request = SaveRequest::from_args(&args);
            let start = match &request.directory {
                Some(directory) => expand(directory),
                None => home.to_path_buf(),
            };
            match portal::save(&request, &start) {
                Ok(Some(path)) => {
                    grant(&path);
                    sink.resolve(call, json!({ "path": path.to_string_lossy() }));
                }
                Ok(None) => sink.resolve(call, json!({ "path": null })),
                Err(message) => sink.reject_with_code(call, "Other", message),
            }
        }
        _ => sink.reject(call, format!("svelte-gpui: unknown operation {op}")),
    }
}

#[cfg(target_os = "linux")]
mod portal {
    use std::path::{Path, PathBuf};

    use ashpd::desktop::file_chooser::{FileFilter, OpenFileRequest, SaveFileRequest};

    use super::{Chosen, Filter, OpenRequest, SaveRequest};

    fn filter_of(filter: &Filter) -> FileFilter {
        filter.patterns.iter().fold(FileFilter::new(&filter.name), |built, pattern| built.glob(pattern))
    }

    fn failed(error: ashpd::Error) -> String {
        match error {
            ashpd::Error::PortalNotFound(_) => "this desktop has no file chooser to show the dialog with: install \
                 xdg-desktop-portal and a backend for your desktop (xdg-desktop-portal-gtk, -gnome, -kde)"
                .to_string(),
            other => format!("the desktop's file chooser could not be used: {other}"),
        }
    }

    /// Whether a response is the user's cancel rather than a failure. The portal answers a cancel as an error.
    fn cancelled(error: &ashpd::Error) -> bool {
        matches!(error, ashpd::Error::Response(_))
    }

    fn paths_of(uris: &[ashpd::url::Url]) -> Vec<PathBuf> {
        uris.iter().filter_map(|uri| uri.to_file_path().ok()).collect()
    }

    pub fn open(request: &OpenRequest) -> Chosen<Vec<PathBuf>> {
        futures::executor::block_on(async {
            let sent = OpenFileRequest::default()
                .modal(false)
                .title(request.title())
                .accept_label(request.prompt.as_deref())
                .multiple(request.multiple)
                .directory(request.directories)
                .filters(request.filters.iter().map(filter_of))
                .send()
                .await
                .map_err(failed)?;
            match sent.response() {
                Ok(response) => Ok(Some(paths_of(response.uris()))),
                Err(error) if cancelled(&error) => Ok(None),
                Err(error) => Err(failed(error)),
            }
        })
    }

    pub fn save(request: &SaveRequest, start: &Path) -> Chosen<PathBuf> {
        futures::executor::block_on(async {
            let mut builder = SaveFileRequest::default()
                .modal(false)
                .title("Save File")
                .filters(request.filters.iter().map(filter_of))
                .current_folder(start)
                .map_err(|error| format!("cannot start the dialog in {}: {error}", start.display()))?;
            if let Some(name) = &request.name {
                builder = builder.current_name(name.as_str());
            }
            let sent = builder.send().await.map_err(failed)?;
            match sent.response() {
                Ok(response) => Ok(paths_of(response.uris()).into_iter().next()),
                Err(error) if cancelled(&error) => Ok(None),
                Err(error) => Err(failed(error)),
            }
        })
    }
}

#[cfg(not(target_os = "linux"))]
mod portal {
    use std::path::{Path, PathBuf};

    use super::{Chosen, OpenRequest, SaveRequest};

    fn missing() -> String {
        "this host shows file dialogs only through xdg-desktop-portal, which is Linux's".to_string()
    }

    pub fn open(_: &OpenRequest) -> Chosen<Vec<PathBuf>> {
        Err(missing())
    }

    pub fn save(_: &SaveRequest, _: &Path) -> Chosen<PathBuf> {
        Err(missing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_request_reads_what_the_app_asked_for() {
        let request = OpenRequest::from_args(&json!({
            "multiple": true,
            "prompt": "Import",
            "filters": [{ "name": "Images", "extensions": ["png", ".jpg", "*.webp"] }],
        }));
        assert_eq!(
            request,
            OpenRequest {
                directories: false,
                multiple: true,
                prompt: Some("Import".into()),
                filters: vec![Filter {
                    name: "Images".into(),
                    patterns: vec!["*.png".into(), "*.jpg".into(), "*.webp".into()]
                }],
            }
        );
        assert_eq!(request.title(), "Open File");
    }

    #[test]
    fn folders_are_asked_for_with_directories_and_titled_so() {
        let request = OpenRequest::from_args(&json!({ "directories": true }));
        assert!(request.directories);
        assert!(!request.multiple);
        assert_eq!(request.prompt, None);
        assert!(request.filters.is_empty());
        assert_eq!(request.title(), "Open Folder");
    }

    #[test]
    fn nothing_asked_means_one_file_and_no_filter() {
        assert_eq!(
            OpenRequest::from_args(&json!({})),
            OpenRequest { directories: false, multiple: false, prompt: None, filters: vec![] }
        );
        assert_eq!(OpenRequest::from_args(&Value::Null).filters, vec![]);
    }

    #[test]
    fn a_save_request_reads_where_to_start_and_what_to_suggest() {
        let request = SaveRequest::from_args(
            &json!({ "directory": "~/Documents", "name": "notes.md", "filters": [{ "extensions": ["md"] }] }),
        );
        assert_eq!(request.directory.as_deref(), Some("~/Documents"));
        assert_eq!(request.name.as_deref(), Some("notes.md"));
        // A filter with no name is called by what it shows.
        assert_eq!(request.filters, vec![Filter { name: "*.md".into(), patterns: vec!["*.md".into()] }]);
        let bare = SaveRequest::from_args(&json!({}));
        assert_eq!((bare.directory, bare.name), (None, None));
    }

    #[test]
    fn an_extension_is_written_as_a_pattern_whichever_way_it_was_given() {
        assert_eq!(glob_of("png").as_deref(), Some("*.png"));
        assert_eq!(glob_of(".png").as_deref(), Some("*.png"));
        assert_eq!(glob_of("*.png").as_deref(), Some("*.png"));
        assert_eq!(glob_of(" tar ").as_deref(), Some("*.tar"));
        assert_eq!(glob_of("c++").as_deref(), Some("*.c++"));
    }

    #[test]
    fn what_is_not_an_extension_is_left_out() {
        assert_eq!(glob_of(""), None);
        assert_eq!(glob_of("*"), None);
        assert_eq!(glob_of("a/b"), None);
        assert_eq!(glob_of("p ng"), None);
        let filters = filters_of(&json!([
            { "name": "Nothing", "extensions": [] },
            { "name": "Bad", "extensions": ["", "a/b", 3] },
            { "extensions": ["txt", "bad ext"] },
            "not an object",
        ]));
        assert_eq!(filters, vec![Filter { name: "*.txt".into(), patterns: vec!["*.txt".into()] }]);
    }
}
