//! An app packed into its executable: the bundle (as bytecode, when it was compiled), the
//! files beside it, and who the app is. `scripts/package.mjs` produces the pieces and
//! [`embed!`](crate::embed) includes them in the binary.
//!
//! The bundle runs straight from the binary's memory. The files (images, mostly) are
//! unpacked once to a cache directory, because GPUI reads images from files. The
//! directory is named after a hash of the files, so each version of an app unpacks
//! once, a start finds it ready, and the old versions' directories are removed.
//!
//! The files travel as an archive: for each file, its path (UTF-8, `/` between parts,
//! relative) as a `u32` length and the bytes, then its size as a `u64` and the bytes,
//! all little-endian.

use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::runtime::Script;

/// Marks a directory whose unpacking finished.
const COMPLETE: &str = ".unpacked";

/// What `scripts/package.mjs` made for an app, included in the binary. Build it with
/// [`embed!`](crate::embed).
#[derive(Clone, Copy)]
pub struct Embedded {
    metadata: &'static str,
    script: &'static [u8],
    files: &'static [u8],
}

impl std::fmt::Debug for Embedded {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.metadata().map(|metadata| metadata.name).unwrap_or_default();
        write!(formatter, "Embedded({name:?}, script {} bytes, files {} bytes)", self.script.len(), self.files.len())
    }
}

/// Who an embedded app is.
#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    /// The name people see.
    pub name: String,
    /// A reverse-domain identifier, which names the unpacking directory and the window's
    /// application id (so the desktop matches the window to its `.desktop` file).
    pub id: String,
    pub version: String,
    /// The window's title, when it differs from the name.
    pub title: Option<String>,
    /// Whether the script is bytecode (otherwise it is source).
    pub bytecode: bool,
    /// A hash of the files, naming the directory they are unpacked to.
    pub hash: String,
}

/// An embedded app made ready to run.
pub struct Unpacked {
    pub metadata: Metadata,
    /// Where the files are. The bundle's directory for this app: `files.paths().bundle`
    /// and relative image paths start here. The executable's directory if the app has no files.
    pub directory: PathBuf,
    pub script: Script,
}

impl Embedded {
    /// Used by [`embed!`](crate::embed); the arguments are the packager's three files.
    pub const fn new(metadata: &'static str, script: &'static [u8], files: &'static [u8]) -> Self {
        Embedded { metadata, script, files }
    }

    pub fn metadata(&self) -> Result<Metadata, String> {
        let json: Value = serde_json::from_str(self.metadata).map_err(|error| format!("bad metadata: {error}"))?;
        let text = |key: &str| json[key].as_str().map(str::to_string);
        let name = text("name").ok_or("the metadata has no name")?;
        Ok(Metadata {
            id: text("id").ok_or("the metadata has no id")?,
            version: text("version").unwrap_or_default(),
            title: text("title"),
            bytecode: json["bytecode"].as_bool().unwrap_or(false),
            hash: text("hash").unwrap_or_default(),
            name,
        })
    }

    /// Unpacks the files if this version has not been yet, and says where the app is.
    pub fn unpack(&self) -> Result<Unpacked, String> {
        let metadata = self.metadata()?;
        let script = if metadata.bytecode { Script::Bytecode(self.script) } else { Script::Source(self.script) };
        let directory = if self.files.is_empty() { executable_directory() } else { self.unpack_files(&metadata)? };
        Ok(Unpacked { metadata, directory, script })
    }

    fn unpack_files(&self, metadata: &Metadata) -> Result<PathBuf, String> {
        let mut failures = Vec::new();
        for parent in [cache_home().join(&metadata.id), std::env::temp_dir().join(&metadata.id)] {
            match self.unpack_into(&parent, &metadata.hash) {
                Ok(directory) => return Ok(directory),
                Err(error) => failures.push(format!("{}: {error}", parent.display())),
            }
        }
        Err(format!("cannot unpack the app's files ({})", failures.join("; ")))
    }

    fn unpack_into(&self, parent: &Path, hash: &str) -> std::io::Result<PathBuf> {
        let directory = parent.join(format!("files-{hash}"));
        if directory.join(COMPLETE).is_file() {
            return Ok(directory);
        }
        // Unpack beside the final directory and rename it into place, so a start that is
        // interrupted, or two that race, never see a half-written directory.
        let staging = parent.join(format!(".unpack-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(&staging)?;
        let written = write_archive(self.files, &staging).and_then(|()| std::fs::write(staging.join(COMPLETE), hash));
        if let Err(error) = written {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(error);
        }
        if std::fs::rename(&staging, &directory).is_err() {
            // Someone else finished first.
            let _ = std::fs::remove_dir_all(&staging);
            if !directory.join(COMPLETE).is_file() {
                return Err(std::io::Error::other("the files could not be moved into place"));
            }
        }
        remove_other_versions(parent, &directory);
        Ok(directory)
    }
}

/// Includes what `scripts/package.mjs` produced. The packager sets `SVELTE_GPUI_EMBED_DIR`
/// to the directory it wrote them to when it builds the app's crate:
///
/// ```ignore
/// let options = svelte_gpui::Options::embedded(svelte_gpui::embed!());
/// ```
#[macro_export]
macro_rules! embed {
    () => {
        $crate::Embedded::new(
            include_str!(concat!(env!("SVELTE_GPUI_EMBED_DIR"), "/metadata.json")),
            include_bytes!(concat!(env!("SVELTE_GPUI_EMBED_DIR"), "/script.bin")),
            include_bytes!(concat!(env!("SVELTE_GPUI_EMBED_DIR"), "/files.bin")),
        )
    };
}

fn executable_directory() -> PathBuf {
    std::env::current_exe().ok().and_then(|path| path.parent().map(Path::to_path_buf)).unwrap_or_default()
}

fn cache_home() -> PathBuf {
    match std::env::var_os("XDG_CACHE_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache"),
    }
}

/// Removes the other versions' directories, which nothing will start from again.
fn remove_other_versions(parent: &Path, keep: &Path) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_old_version = entry.file_name().to_string_lossy().starts_with("files-");
        if is_old_version && path != keep {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

/// Writes every file of an archive below `root`.
fn write_archive(archive: &[u8], root: &Path) -> std::io::Result<()> {
    let invalid = |reason: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, reason.to_string());
    let mut rest = archive;
    while !rest.is_empty() {
        let (path, after) = take_chunk(rest, 4).ok_or_else(|| invalid("the archive is cut short"))?;
        let (data, after) = take_chunk(after, 8).ok_or_else(|| invalid("the archive is cut short"))?;
        rest = after;
        let relative = Path::new(std::str::from_utf8(path).map_err(|_| invalid("a path is not UTF-8"))?);
        if !relative.components().all(|part| matches!(part, Component::Normal(_))) {
            return Err(invalid("a path leaves the directory"));
        }
        let target = root.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, data)?;
    }
    Ok(())
}

/// Splits `bytes` into a chunk and what follows it. The chunk's length is the little-endian
/// number in the first `width` bytes (4 or 8).
fn take_chunk(bytes: &[u8], width: usize) -> Option<(&[u8], &[u8])> {
    let (prefix, rest) = bytes.split_at_checked(width)?;
    let mut length = [0u8; 8];
    length[..width].copy_from_slice(prefix);
    let length = usize::try_from(u64::from_le_bytes(length)).ok()?;
    rest.split_at_checked(length)
}
