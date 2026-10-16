//! What the app may touch. Nothing is limited until the app (or the operator, with
//! `--allow-files`) says so; after that a path must be inside the declared roots, unless
//! the user picked it in a dialog. Paths are compared in their canonical form, so `..`
//! and symbolic links cannot lead out.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

/// The access limit. Each list holds canonical paths, and each path grants itself and
/// everything below it.
#[derive(Default)]
pub struct Limit {
    /// What the app declared with `fs.limit`. `None` until it does, and then it stays.
    declared: Option<Vec<PathBuf>>,
    /// What the operator allowed with `--allow-files`. `None` when the flag is absent.
    operator: Option<Vec<PathBuf>>,
    /// Paths the user picked in a dialog. They count even outside the other two.
    picked: Vec<PathBuf>,
}

impl Limit {
    pub fn new(operator: Option<Vec<PathBuf>>) -> Self {
        Limit { declared: None, operator, picked: Vec::new() }
    }

    /// Sets the app's limit. It can be set once; a second attempt fails, so code that runs
    /// later cannot widen it.
    pub fn declare(&mut self, roots: Vec<PathBuf>) -> Result<(), String> {
        if self.declared.is_some() {
            return Err("the file access limit is already set and cannot be changed".to_string());
        }
        self.declared = Some(roots);
        Ok(())
    }

    /// Grants a path the user picked, for the rest of the session.
    pub fn grant(&mut self, path: PathBuf) {
        if !self.picked.contains(&path) {
            self.picked.push(path);
        }
    }

    /// Whether a canonical path is allowed: inside something picked, or inside both the
    /// app's roots and the operator's, wherever those exist.
    pub fn permits(&self, path: &Path) -> bool {
        let inside = |roots: &Vec<PathBuf>| roots.iter().any(|root| path.starts_with(root));
        self.picked.iter().any(|root| path.starts_with(root))
            || (self.declared.as_ref().is_none_or(inside) && self.operator.as_ref().is_none_or(inside))
    }
}

/// One step of a path being resolved.
enum Step {
    /// A prefix or the root: starts the path over.
    Start(OsString),
    Parent,
    Name(OsString),
}

fn steps(path: &Path) -> VecDeque<Step> {
    path.components()
        .filter_map(|component| match component {
            Component::Prefix(_) | Component::RootDir => Some(Step::Start(component.as_os_str().to_owned())),
            Component::CurDir => None,
            Component::ParentDir => Some(Step::Parent),
            Component::Normal(name) => Some(Step::Name(name.to_owned())),
        })
        .collect()
}

/// More links than this in one path are taken to be a loop.
const MOST_LINKS: usize = 40;

/// The path with every `.`, `..` and symbolic link resolved, one step at a time. Unlike
/// `std::fs::canonicalize`, it also works for a path that does not exist yet: the part
/// that is missing is kept as written, after the real location of the part that is there.
/// `path` must be absolute. When `follow_last` is false, a link in the last position stays
/// a link, which is what removing or renaming it needs.
pub fn canonical(path: &Path, follow_last: bool) -> PathBuf {
    let mut result = PathBuf::new();
    let mut remaining = steps(path);
    let mut links = 0;
    while let Some(step) = remaining.pop_front() {
        match step {
            Step::Start(text) => result.push(text),
            Step::Parent => {
                result.pop();
            }
            Step::Name(name) => {
                let candidate = result.join(&name);
                if remaining.is_empty() && !follow_last {
                    result = candidate;
                    continue;
                }
                match link_target(&candidate, links) {
                    Some(target) => {
                        links += 1;
                        if target.is_absolute() {
                            result = PathBuf::new();
                        }
                        for step in steps(&target).into_iter().rev() {
                            remaining.push_front(step);
                        }
                    }
                    None => result = candidate,
                }
            }
        }
    }
    result
}

/// Where `path` points, if it is a symbolic link that may still be followed.
fn link_target(path: &Path, links: usize) -> Option<PathBuf> {
    if links >= MOST_LINKS || !std::fs::symlink_metadata(path).ok()?.file_type().is_symlink() {
        return None;
    }
    std::fs::read_link(path).ok()
}
