//! Watching a file or directory with the operating system's notifications (inotify on
//! Linux, FSEvents on macOS, ReadDirectoryChanges on Windows), so a change is reported
//! as it happens and an idle watch costs nothing.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

use notify::event::{EventKind, ModifyKind, RenameMode};
use notify::{RecursiveMode, Watcher};
use serde_json::json;

use crate::io::Sink;

/// How often the watch looks at its stop flag while nothing changes.
const STOP_CHECK: Duration = Duration::from_millis(100);

/// The kinds of change the app hears about, for one notification. A rename is a
/// `remove` of the old path and a `create` of the new one; reads and metadata-only
/// changes are left out.
fn changes(kind: &EventKind, paths: &[PathBuf]) -> Vec<(&'static str, PathBuf)> {
    let all = |name: &'static str| paths.iter().map(|path| (name, path.clone())).collect();
    match kind {
        EventKind::Create(_) => all("create"),
        EventKind::Remove(_) => all("remove"),
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => all("remove"),
        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => all("create"),
        // Linux reports a rename as `From` and `To`, and then once more as `Both`; the
        // first two are enough.
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => Vec::new(),
        // Systems that cannot tell which end of a rename a path is: whether it is still
        // there says.
        EventKind::Modify(ModifyKind::Name(_)) => {
            paths.iter().map(|path| (if path.exists() { "create" } else { "remove" }, path.clone())).collect()
        }
        EventKind::Modify(ModifyKind::Metadata(_)) => Vec::new(),
        EventKind::Modify(_) => all("modify"),
        _ => Vec::new(),
    }
}

/// A path the system reported, written under `shown`, the way the app named the watch.
fn as_shown(path: &Path, target: &Path, shown: &Path) -> PathBuf {
    let relative = path.strip_prefix(target).unwrap_or(path);
    if relative.as_os_str().is_empty() { shown.to_path_buf() } else { shown.join(relative) }
}

/// Watches `target` until `stop` is set, sending one event per change and closing the
/// stream at the end. Events name paths under `shown`, the way the app wrote the path,
/// though `target` (the canonical path) is what is watched. A watch that cannot start
/// rejects the call, which the app hears through `onerror`.
pub fn watch(sink: &Sink, call: u64, target: PathBuf, shown: PathBuf, recursive: bool, stop: Arc<AtomicBool>) {
    let (sender, receiver) = channel();
    let started = notify::recommended_watcher(sender).and_then(|mut watcher| {
        let mode = if recursive { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };
        watcher.watch(&target, mode).map(|_| watcher)
    });
    let _watcher = match started {
        Ok(watcher) => watcher,
        Err(error) => {
            let code = if matches!(error.kind, notify::ErrorKind::PathNotFound) { "NotFound" } else { "Other" };
            sink.reject_with_code(call, code, format!("cannot watch {}: {error}", shown.display()));
            return;
        }
    };
    while !stop.load(Ordering::Relaxed) {
        match receiver.recv_timeout(STOP_CHECK) {
            Ok(Ok(event)) => {
                for (kind, path) in changes(&event.kind, &event.paths) {
                    let path = as_shown(&path, &target, &shown);
                    sink.event(call, json!({ "kind": kind, "path": path.to_string_lossy() }), None);
                }
            }
            Ok(Err(_)) | Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    sink.close(call, json!({}));
}
