//! What ships inside the binary: icons the plugins name, and the fonts.

use std::borrow::Cow;

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

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
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
