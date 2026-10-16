//! Files: reading, writing and listing, on threads, within what the app allowed.
//!
//! Every path the app names is made absolute, resolved to its canonical form and checked
//! against the limit before anything is touched (see `limit`). A refusal rejects the call
//! with the code `NotAllowed`. Each failure carries a code the app can tell apart, which
//! becomes `FileError.code` in `runtime/io/files.ts`.
//!
//! The operations, by name: `fs.readText`, `fs.readBytes`, `fs.writeText`, `fs.writeBytes`,
//! `fs.appendText`, `fs.exists`, `fs.stat`, `fs.list`, `fs.mkdir`, `fs.remove`, `fs.rename`,
//! `fs.copy`, `fs.watch` (a stream, ended by `fs.unwatch`), `fs.paths`, and `fs.limit`.
//! The dialogs, `fs.pickFiles` and `fs.pickSavePath`, run on the main thread.

mod limit;
mod paths;
mod watch;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use super::{MainJob, Sink};
use limit::{Limit, canonical};
pub use paths::Directories;

/// Why an operation failed: a short code the app can test, and a message for people.
struct FileError {
    code: &'static str,
    message: String,
}

impl FileError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        FileError { code, message: message.into() }
    }

    /// An operating system error, as `cannot read /some/path: No such file (os error 2)`.
    fn system(action: &str, path: &str, error: std::io::Error) -> Self {
        use std::io::ErrorKind;
        let code = match error.kind() {
            ErrorKind::NotFound => "NotFound",
            ErrorKind::PermissionDenied => "PermissionDenied",
            ErrorKind::AlreadyExists => "Exists",
            ErrorKind::DirectoryNotEmpty => "NotEmpty",
            ErrorKind::NotADirectory => "NotADirectory",
            ErrorKind::IsADirectory => "IsADirectory",
            ErrorKind::InvalidData => "InvalidData",
            ErrorKind::InvalidInput => "InvalidArgument",
            _ => "Other",
        };
        FileError::new(code, format!("cannot {action} {path}: {error}"))
    }

    fn not_allowed(path: &str) -> Self {
        FileError::new("NotAllowed", format!("access to {path} is not allowed"))
    }

    fn argument(message: impl Into<String>) -> Self {
        FileError::new("InvalidArgument", message)
    }
}

/// What a finished operation hands back.
enum Reply {
    Json(Value),
    Bytes(Value, Vec<u8>),
}

type Outcome = Result<Reply, FileError>;

/// File access, and the limit on it. Cloning gives another handle on the same state.
#[derive(Clone)]
pub struct Files {
    directories: Arc<Directories>,
    limit: Arc<Mutex<Limit>>,
    /// The running watches by call number; setting the flag ends the watch.
    watches: Arc<Mutex<HashMap<u64, Arc<AtomicBool>>>>,
}

impl Files {
    /// File access for an app whose bundle is at `bundle`, which is where `$bundle` points.
    /// Each path in `operator` narrows access to it (and everything below it), on top of any
    /// limit the app declares; an empty list narrows nothing.
    pub fn new(bundle: Option<&str>, operator: &[String]) -> Self {
        let directories = Directories::find(bundle);
        let operator = (!operator.is_empty())
            .then(|| operator.iter().map(|path| canonical(&directories.expand(path), true)).collect());
        Files {
            directories: Arc::new(directories),
            limit: Arc::new(Mutex::new(Limit::new(operator))),
            watches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn directories(&self) -> &Directories {
        &self.directories
    }

    /// Allows a path the user picked in a dialog, and everything below it, for the rest of
    /// the session, whatever the limit says.
    pub fn grant_picked(&self, path: &Path) {
        let path = canonical(&self.directories.expand(&path.to_string_lossy()), true);
        self.limit().grant(path);
    }

    fn limit(&self) -> MutexGuard<'_, Limit> {
        self.limit.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn watches(&self) -> MutexGuard<'_, HashMap<u64, Arc<AtomicBool>>> {
        self.watches.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Ends every running watch, for an app that is going away.
    pub fn stop_watches(&self) {
        for stop in self.watches().values() {
            stop.store(true, Ordering::Relaxed);
        }
    }

    /// `fs.*` operations. Ones that need GPUI, such as dialogs, are pushed onto `main_jobs`.
    pub fn request(
        &self,
        sink: &Sink,
        main_jobs: &std::cell::RefCell<Vec<MainJob>>,
        op: &str,
        call: u64,
        args: Value,
        bytes: Option<Vec<u8>>,
    ) {
        match op {
            "fs.limit" => self.settle(sink, call, self.declare_limit(&args)),
            "fs.paths" => sink.resolve(call, self.directories.to_json()),
            "fs.unwatch" => self.settle(sink, call, self.unwatch(&args)),
            "fs.pickFiles" | "fs.pickSavePath" => {
                main_jobs.borrow_mut().push(MainJob { op: op.to_string(), call, args });
            }
            "fs.watch" => self.start_watch(sink, call, args),
            _ => {
                let (files, sink, op) = (self.clone(), sink.clone(), op.to_string());
                std::thread::spawn(move || {
                    let outcome = files.execute(&op, &args, bytes);
                    files.settle(&sink, call, outcome);
                });
            }
        }
    }

    fn settle(&self, sink: &Sink, call: u64, outcome: Outcome) {
        match outcome {
            Ok(Reply::Json(json)) => sink.resolve(call, json),
            Ok(Reply::Bytes(json, bytes)) => sink.resolve_bytes(call, json, bytes),
            Err(error) => reject(sink, call, error),
        }
    }

    /// Resolves a path the app wrote to the canonical path the operation works on, or fails
    /// if the limit does not allow it. With `follow_last` false a link in last position is
    /// not followed, for the operations that act on the link itself.
    fn check(&self, raw: &str, follow_last: bool) -> Result<PathBuf, FileError> {
        let target = canonical(&self.directories.expand(raw), follow_last);
        if self.limit().permits(&target) { Ok(target) } else { Err(FileError::not_allowed(raw)) }
    }

    fn declare_limit(&self, args: &Value) -> Outcome {
        let mut roots = Vec::new();
        for entry in array(args, "allow")? {
            let raw = entry.as_str().ok_or_else(|| FileError::argument("allow holds only paths"))?;
            let path = self.directories.expand_allowed(raw).map_err(FileError::argument)?;
            roots.push(canonical(&path, true));
        }
        self.limit().declare(roots).map_err(|message| FileError::new("NotAllowed", message))?;
        Ok(Reply::Json(json!({})))
    }

    fn execute(&self, op: &str, args: &Value, bytes: Option<Vec<u8>>) -> Outcome {
        match op {
            "fs.readText" => self.read_text(args),
            "fs.readBytes" => self.read_bytes(args),
            "fs.writeText" => self.write(args, string(args, "text")?.as_bytes(), false),
            "fs.writeBytes" => self.write(args, &bytes.ok_or_else(|| FileError::argument("no bytes to write"))?, false),
            "fs.appendText" => self.write(args, string(args, "text")?.as_bytes(), true),
            "fs.exists" => self.exists(args),
            "fs.stat" => self.stat(args),
            "fs.list" => self.list(args),
            "fs.mkdir" => self.mkdir(args),
            "fs.remove" => self.remove(args),
            "fs.rename" => self.rename(args),
            "fs.copy" => self.copy(args),
            _ => Err(FileError::argument(format!("svelte-gpui: unknown operation {op}"))),
        }
    }

    fn read_text(&self, args: &Value) -> Outcome {
        let raw = string(args, "path")?;
        let bytes = fs::read(self.check(raw, true)?).map_err(|error| FileError::system("read", raw, error))?;
        let text = String::from_utf8(bytes)
            .map_err(|_| FileError::new("InvalidData", format!("cannot read {raw}: it is not valid UTF-8 text")))?;
        Ok(Reply::Json(json!({ "text": text })))
    }

    fn read_bytes(&self, args: &Value) -> Outcome {
        let raw = string(args, "path")?;
        let bytes = fs::read(self.check(raw, true)?).map_err(|error| FileError::system("read", raw, error))?;
        Ok(Reply::Bytes(json!({}), bytes))
    }

    /// Writes (or appends) `data`, creating the file when it is not there. The directory
    /// has to exist already.
    fn write(&self, args: &Value, data: &[u8], append: bool) -> Outcome {
        let raw = string(args, "path")?;
        let target = self.check(raw, true)?;
        let mut options = fs::OpenOptions::new();
        options.create(true);
        if append {
            options.append(true);
        } else {
            options.write(true).truncate(true);
        }
        let mut file = options.open(target).map_err(|error| FileError::system("write", raw, error))?;
        file.write_all(data).map_err(|error| FileError::system("write", raw, error))?;
        Ok(Reply::Json(json!({})))
    }

    fn exists(&self, args: &Value) -> Outcome {
        let target = self.check(string(args, "path")?, true)?;
        Ok(Reply::Json(json!({ "exists": target.exists() })))
    }

    /// Describes the path itself: a link is reported as a link, not as what it points to.
    fn stat(&self, args: &Value) -> Outcome {
        let raw = string(args, "path")?;
        let metadata =
            fs::symlink_metadata(self.check(raw, false)?).map_err(|error| FileError::system("stat", raw, error))?;
        Ok(Reply::Json(describe(&metadata)))
    }

    fn list(&self, args: &Value) -> Outcome {
        let raw = string(args, "path")?;
        let shown = self.directories.expand(raw);
        let entries = fs::read_dir(self.check(raw, true)?).map_err(|error| FileError::system("list", raw, error))?;
        let mut listed = Vec::new();
        for item in entries {
            let item = item.map_err(|error| FileError::system("list", raw, error))?;
            let Ok(metadata) = item.metadata() else {
                continue;
            };
            let name = item.file_name().to_string_lossy().into_owned();
            let mut entry = describe(&metadata);
            entry["path"] = json!(shown.join(&name).to_string_lossy());
            entry["name"] = json!(name);
            listed.push(entry);
        }
        listed.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        Ok(Reply::Json(json!({ "entries": listed })))
    }

    fn mkdir(&self, args: &Value) -> Outcome {
        let raw = string(args, "path")?;
        let target = self.check(raw, true)?;
        let made = if flag(args, "recursive") { fs::create_dir_all(target) } else { fs::create_dir(target) };
        made.map_err(|error| FileError::system("create the directory", raw, error))?;
        Ok(Reply::Json(json!({})))
    }

    /// Removes a file or link, or a directory (with everything in it when `recursive`).
    fn remove(&self, args: &Value) -> Outcome {
        let raw = string(args, "path")?;
        let target = self.check(raw, false)?;
        let metadata = fs::symlink_metadata(&target).map_err(|error| FileError::system("remove", raw, error))?;
        let removed = match (metadata.is_dir(), flag(args, "recursive")) {
            (true, true) => fs::remove_dir_all(&target),
            (true, false) => fs::remove_dir(&target),
            (false, _) => fs::remove_file(&target),
        };
        removed.map_err(|error| FileError::system("remove", raw, error))?;
        Ok(Reply::Json(json!({})))
    }

    /// Moves a file or directory. Both ends are checked. It cannot cross file systems.
    fn rename(&self, args: &Value) -> Outcome {
        let (from, to) = (string(args, "from")?, string(args, "to")?);
        let (source, destination) = (self.check(from, false)?, self.check(to, false)?);
        fs::rename(source, destination)
            .map_err(|error| FileError::system("rename", &format!("{from} to {to}"), error))?;
        Ok(Reply::Json(json!({})))
    }

    /// Copies a file, or a directory with what is in it. Links inside a directory are left
    /// out, so a copy never reads from somewhere the app could not have named.
    fn copy(&self, args: &Value) -> Outcome {
        let (from, to) = (string(args, "from")?, string(args, "to")?);
        let (source, destination) = (self.check(from, true)?, self.check(to, true)?);
        let failed = |error| FileError::system("copy", &format!("{from} to {to}"), error);
        if source.is_dir() {
            if destination.starts_with(&source) {
                return Err(FileError::argument(format!("cannot copy {from} into itself ({to})")));
            }
            copy_directory(&source, &destination).map_err(failed)?;
        } else {
            fs::copy(&source, &destination).map_err(failed)?;
        }
        Ok(Reply::Json(json!({})))
    }

    /// Starts watching on a thread. The stream ends when `fs.unwatch` names this call.
    fn start_watch(&self, sink: &Sink, call: u64, args: Value) {
        let stop = Arc::new(AtomicBool::new(false));
        self.watches().insert(call, stop.clone());
        let (files, sink) = (self.clone(), sink.clone());
        std::thread::spawn(move || {
            let raw = string(&args, "path");
            match raw.and_then(|raw| Ok((files.check(raw, true)?, files.directories.expand(raw)))) {
                Ok((target, shown)) => watch::watch(&sink, call, target, shown, flag(&args, "recursive"), stop),
                Err(error) => reject(&sink, call, error),
            }
            files.watches().remove(&call);
        });
    }

    fn unwatch(&self, args: &Value) -> Outcome {
        let call = args["watch"].as_u64().ok_or_else(|| FileError::argument("no watch to stop"))?;
        if let Some(stop) = self.watches().get(&call) {
            stop.store(true, Ordering::Relaxed);
        }
        Ok(Reply::Json(json!({})))
    }
}

/// Fails the call with the error's code, which the app reads as `FileError.code`.
fn reject(sink: &Sink, call: u64, error: FileError) {
    sink.reject_with_code(call, error.code, error.message);
}

fn string<'a>(args: &'a Value, name: &str) -> Result<&'a str, FileError> {
    args[name].as_str().ok_or_else(|| FileError::argument(format!("{name} must be a string")))
}

fn array<'a>(args: &'a Value, name: &str) -> Result<&'a Vec<Value>, FileError> {
    args[name].as_array().ok_or_else(|| FileError::argument(format!("{name} must be a list")))
}

fn flag(args: &Value, name: &str) -> bool {
    args[name].as_bool().unwrap_or(false)
}

/// The kind, size and times of something on disk, as the app reads them.
fn describe(metadata: &fs::Metadata) -> Value {
    let kind = match metadata.file_type() {
        kind if kind.is_symlink() => "symlink",
        kind if kind.is_dir() => "directory",
        _ => "file",
    };
    let milliseconds = |time: std::io::Result<SystemTime>| {
        time.ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok()).map(|elapsed| elapsed.as_millis() as u64)
    };
    let mut described = json!({ "kind": kind, "size": metadata.len(), "modified": milliseconds(metadata.modified()) });
    if let Some(created) = milliseconds(metadata.created()) {
        described["created"] = json!(created);
    }
    described
}

fn copy_directory(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for item in fs::read_dir(from)? {
        let item = item?;
        let (source, destination) = (item.path(), to.join(item.file_name()));
        let kind = item.file_type()?;
        if kind.is_dir() {
            copy_directory(&source, &destination)?;
        } else if kind.is_file() {
            fs::copy(&source, &destination)?;
        }
    }
    Ok(())
}
