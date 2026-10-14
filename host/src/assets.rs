//! What ships inside the binary: icons the plugins name, and the fonts.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use gpui::{AssetSource, Result, SharedString};

macro_rules! embedded {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_bytes!(concat!("../assets/", $path)) as &[u8])),*]
    };
}

const FILES: &[(&str, &[u8])] = embedded![
    "icons/circle.svg",
    "icons/check-circle.svg",
    "icons/git-merge.svg",
    "icons/circle-dashed.svg",
    "icons/chat-circle.svg",
    "icons/arrow-clockwise.svg",
    "icons/plus.svg",
    "icons/x.svg",
    "icons/caret-down.svg",
    "icons/push-pin.svg",
    "icons/magnifying-glass.svg",
];

/// SVG documents the page drew, keyed by a path made up from their content.
static DOCUMENTS: OnceLock<Mutex<HashMap<String, Arc<Vec<u8>>>>> = OnceLock::new();

fn documents() -> &'static Mutex<HashMap<String, Arc<Vec<u8>>>> {
    DOCUMENTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Registers an SVG document and returns the asset path GPUI should draw it from.
pub fn register_svg(markup: String) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    markup.hash(&mut hasher);
    let path = format!("svg/{:016x}.svg", hasher.finish());
    documents()
        .lock()
        .expect("svg documents")
        .entry(path.clone())
        .or_insert_with(|| Arc::new(markup.into_bytes()));
    path
}

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.starts_with("svg/") {
            let found = documents().lock().expect("svg documents").get(path).cloned();
            return Ok(found.map(|bytes| Cow::Owned(bytes.as_ref().clone())));
        }
        for (name, bytes) in FILES {
            if *name == path {
                return Ok(Some(Cow::Borrowed(bytes)));
            }
        }
        Ok(None)
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(FILES.iter().map(|(name, _)| SharedString::from(*name)).collect())
    }
}

/// The fonts the theme uses, as TTF bytes.
pub fn fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-400.ttf") as &[u8]),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-500.ttf") as &[u8]),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-600.ttf") as &[u8]),
        Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf") as &[u8]),
    ]
}
