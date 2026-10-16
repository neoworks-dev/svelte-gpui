//! The well-known directories, and how a path the app wrote becomes an absolute one.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// Where things live on this machine, found once when the host starts.
#[derive(Clone, Debug)]
pub struct Directories {
    pub home: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
    pub temp: PathBuf,
    /// The directory of the running bundle (the `--bundle` flag).
    pub bundle: PathBuf,
    /// The process's current directory when it started; relative paths resolve against it.
    pub cwd: PathBuf,
}

impl Directories {
    /// Finds the directories. `bundle_path` is the `--bundle` flag, when there is one.
    pub fn find(bundle_path: Option<&str>) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let home = home_directory();
        let bundle = bundle_path
            .map(|path| absolute(Path::new(path), &cwd))
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| cwd.clone());
        let (config, data, cache) = application_directories(&home);
        Directories { home, config, data, cache, temp: std::env::temp_dir(), bundle, cwd }
    }

    /// What `files.paths()` answers with.
    pub fn to_json(&self) -> Value {
        json!({
            "home": text(&self.home),
            "config": text(&self.config),
            "data": text(&self.data),
            "cache": text(&self.cache),
            "temp": text(&self.temp),
            "bundle": text(&self.bundle),
            "cwd": text(&self.cwd),
        })
    }

    /// The directory a placeholder such as `$data` stands for.
    fn placeholder(&self, name: &str) -> Option<&Path> {
        Some(match name {
            "home" => &self.home,
            "config" => &self.config,
            "data" => &self.data,
            "cache" => &self.cache,
            "temp" => &self.temp,
            "bundle" => &self.bundle,
            "cwd" => &self.cwd,
            _ => return None,
        })
    }

    /// Makes a path absolute: `~` is the home directory, and a relative path starts at the
    /// current directory. Nothing is looked up on disk, so `..` stays where it is.
    pub fn expand(&self, raw: &str) -> PathBuf {
        match raw.strip_prefix('~') {
            Some("") => self.home.clone(),
            Some(rest) if rest.starts_with('/') || rest.starts_with(std::path::MAIN_SEPARATOR) => {
                self.home.join(&rest[1..])
            }
            _ => absolute(Path::new(raw), &self.cwd),
        }
    }

    /// Like [`Directories::expand`], and a leading placeholder such as `$data/notes` stands
    /// for that directory. Used for the entries of an allow list.
    pub fn expand_allowed(&self, raw: &str) -> Result<PathBuf, String> {
        let Some(rest) = raw.strip_prefix('$') else {
            return Ok(self.expand(raw));
        };
        let (name, tail) = match rest.find(['/', std::path::MAIN_SEPARATOR]) {
            Some(index) => (&rest[..index], &rest[index + 1..]),
            None => (rest, ""),
        };
        match self.placeholder(name) {
            Some(directory) => Ok(directory.join(tail)),
            None => Err(format!("unknown placeholder ${name} in {raw}")),
        }
    }
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn absolute(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { cwd.join(path) }
}

fn home_directory() -> PathBuf {
    let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(variable).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

/// The directories for settings, data and caches, by each system's convention: the XDG
/// directories on Linux, `Library` on macOS, and `AppData` on Windows.
fn application_directories(home: &Path) -> (PathBuf, PathBuf, PathBuf) {
    if cfg!(target_os = "macos") {
        let support = home.join("Library/Application Support");
        return (support.clone(), support, home.join("Library/Caches"));
    }
    if cfg!(windows) {
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| home.join("AppData/Local"));
        let roaming = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| home.join("AppData/Roaming"));
        return (roaming, local.clone(), local.join("cache"));
    }
    let xdg = |variable: &str, fallback: &str| match std::env::var_os(variable).map(PathBuf::from) {
        Some(path) if path.is_absolute() => path,
        _ => home.join(fallback),
    };
    (xdg("XDG_CONFIG_HOME", ".config"), xdg("XDG_DATA_HOME", ".local/share"), xdg("XDG_CACHE_HOME", ".cache"))
}
