//! Where an `<img>` gets its pixels. A file path is used as it is, a `data:` URL
//! is decoded to a cache file, and an `http(s)` URL is fetched on a thread and
//! cached; GPUI only reads files, so every source ends as one. The page learns
//! the outcome as `load` and `error` events, which is how Grove's avatars fall
//! back to an initial.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub enum ImageState {
    Loading,
    Ready(PathBuf),
    Failed,
}

#[derive(Clone)]
pub struct Images {
    states: Arc<Mutex<HashMap<String, ImageState>>>,
    /// Pixel sizes read from file headers, which cost a read each.
    sizes: Arc<Mutex<HashMap<PathBuf, Option<(u32, u32)>>>>,
    /// Set when a background job finishes, so the window draws again.
    pub updated: Arc<AtomicBool>,
    base: PathBuf,
    cache: PathBuf,
}

impl Images {
    /// `base` is where relative paths are looked up.
    pub fn new(base: PathBuf) -> Self {
        let cache_home = match std::env::var_os("XDG_CACHE_HOME") {
            Some(path) => PathBuf::from(path),
            None => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache"),
        };
        let cache = cache_home.join("svelte-gpui").join("images");
        let _ = std::fs::create_dir_all(&cache);
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            sizes: Arc::new(Mutex::new(HashMap::new())),
            updated: Arc::new(AtomicBool::new(false)),
            base,
            cache,
        }
    }

    /// The state of a source, starting to load it if this is the first time it is asked for.
    pub fn state(&self, source: &str) -> ImageState {
        if let Some(state) = self.states.lock().expect("image states").get(source) {
            return state.clone();
        }
        let state = self.start(source);
        self.states
            .lock()
            .expect("image states")
            .insert(source.to_string(), state.clone());
        state
    }

    fn start(&self, source: &str) -> ImageState {
        if let Some(rest) = source.strip_prefix("data:") {
            return self.from_data_url(source, rest);
        }
        if source.starts_with("http://") || source.starts_with("https://") {
            return self.fetch(source);
        }
        let path = if source.starts_with('/') {
            PathBuf::from(source)
        } else {
            self.base.join(source)
        };
        if path.is_file() {
            return ImageState::Ready(path);
        }
        ImageState::Failed
    }

    fn cache_path(&self, key: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        self.cache.join(format!("{:016x}", hasher.finish()))
    }

    fn from_data_url(&self, source: &str, rest: &str) -> ImageState {
        let Some((header, payload)) = rest.split_once(',') else {
            return ImageState::Failed;
        };
        let bytes = if header.ends_with(";base64") {
            match decode_base64(payload) {
                Some(bytes) => bytes,
                None => return ImageState::Failed,
            }
        } else {
            payload.as_bytes().to_vec()
        };
        let path = self.cache_path(source);
        match std::fs::write(&path, bytes) {
            Ok(()) => ImageState::Ready(path),
            Err(_) => ImageState::Failed,
        }
    }

    /// Serves from the cache when it can; otherwise fetches on a thread.
    fn fetch(&self, url: &str) -> ImageState {
        let path = self.cache_path(url);
        if path.is_file() {
            return ImageState::Ready(path);
        }
        let states = self.states.clone();
        let updated = self.updated.clone();
        let url = url.to_string();
        std::thread::spawn(move || {
            let outcome = download(&url, &path);
            let state = match outcome {
                true => ImageState::Ready(path),
                false => ImageState::Failed,
            };
            states.lock().expect("image states").insert(url, state);
            updated.store(true, Ordering::Relaxed);
        });
        ImageState::Loading
    }

    /// The pixel size of an image file, read from its header once.
    pub fn dimensions(&self, path: &PathBuf) -> Option<(u32, u32)> {
        if let Some(known) = self.sizes.lock().expect("image sizes").get(path) {
            return *known;
        }
        // The cache files have no extension, so the format is read from the bytes.
        let size = image::ImageReader::open(path)
            .ok()
            .and_then(|reader| reader.with_guessed_format().ok())
            .and_then(|reader| reader.into_dimensions().ok());
        self.sizes.lock().expect("image sizes").insert(path.clone(), size);
        size
    }

    /// A copy of an image no wider than `width` pixels, made on a thread, so a screenshot
    /// shown small is not decoded and uploaded at its full size. An animated or already
    /// small enough image is used as it is.
    pub fn scaled(&self, path: &PathBuf, width: u32) -> ImageState {
        let Some((natural, _)) = self.dimensions(path) else {
            return ImageState::Ready(path.clone());
        };
        let animated = path.extension().and_then(|e| e.to_str()) == Some("gif");
        if animated || natural <= width + width / 4 {
            return ImageState::Ready(path.clone());
        }
        let key = format!("{}@{width}", path.display());
        if let Some(state) = self.states.lock().expect("image states").get(&key) {
            return state.clone();
        }
        self.states
            .lock()
            .expect("image states")
            .insert(key.clone(), ImageState::Loading);
        let target = self.cache_path(&key);
        let source = path.clone();
        let states = self.states.clone();
        let updated = self.updated.clone();
        std::thread::spawn(move || {
            let state = match shrink(&source, &target, width) {
                true => ImageState::Ready(target),
                false => ImageState::Ready(source),
            };
            states.lock().expect("image states").insert(key, state);
            updated.store(true, Ordering::Relaxed);
        });
        ImageState::Loading
    }

    /// Sources whose state is no longer `Loading`, for the ones that were asked for.
    pub fn settled(&self) -> Vec<(String, ImageState)> {
        self.states
            .lock()
            .expect("image states")
            .iter()
            .filter(|(_, state)| **state != ImageState::Loading)
            .map(|(source, state)| (source.clone(), state.clone()))
            .collect()
    }

    pub fn any_loading(&self) -> bool {
        self.states
            .lock()
            .expect("image states")
            .values()
            .any(|state| *state == ImageState::Loading)
    }
}

/// Writes a copy of `source` that is `width` pixels wide.
fn shrink(source: &PathBuf, target: &PathBuf, width: u32) -> bool {
    if target.is_file() {
        return true;
    }
    let Ok(decoded) = image::open(source) else {
        return false;
    };
    let height = (decoded.height() as f64 * width as f64 / decoded.width() as f64).max(1.0) as u32;
    let resized = decoded.resize_exact(width, height, image::imageops::FilterType::Triangle);
    resized.save_with_format(target, image::ImageFormat::Png).is_ok()
}

fn download(url: &str, path: &PathBuf) -> bool {
    let Ok(response) = ureq::get(url).timeout(std::time::Duration::from_secs(10)).call() else {
        return false;
    };
    let mut bytes = Vec::new();
    if std::io::Read::read_to_end(&mut response.into_reader(), &mut bytes).is_err() {
        return false;
    }
    if bytes.is_empty() {
        return false;
    }
    std::fs::write(path, bytes).is_ok()
}

fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0;
    for character in text.bytes() {
        let value = match character {
            b'A'..=b'Z' => character - b'A',
            b'a'..=b'z' => character - b'a' + 26,
            b'0'..=b'9' => character - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => return None,
        };
        buffer = (buffer << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(out)
}
