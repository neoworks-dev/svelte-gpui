//! An image on the system clipboard, kept as a file so the app reads it like any other file
//! (`clipboard.readImage()`). The file is named after the image's bytes, so pasting the same
//! image again writes nothing.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use gpui::ImageFormat;

/// Where the host wrote a clipboard image, and its MIME type.
pub struct ClipboardImage {
    pub path: String,
    pub mime: &'static str,
}

/// Writes the image `bytes` in `format` into `dir`, unless an earlier paste already did.
pub fn save_image(dir: &Path, format: ImageFormat, bytes: &[u8]) -> std::io::Result<ClipboardImage> {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let path = dir.join(format!("svelte-gpui-clipboard-{:016x}.{}", hasher.finish(), extension(format)));
    if !path.exists() {
        std::fs::write(&path, bytes)?;
    }
    Ok(ClipboardImage { path: path.to_string_lossy().into_owned(), mime: format.mime_type() })
}

fn extension(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Webp => "webp",
        ImageFormat::Gif => "gif",
        ImageFormat::Svg => "svg",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_image_is_written_once_under_one_name() {
        let dir = std::env::temp_dir().join(format!("svelte-gpui-clipboard-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let first = save_image(&dir, ImageFormat::Png, b"not really a png").unwrap();
        let again = save_image(&dir, ImageFormat::Png, b"not really a png").unwrap();
        let other = save_image(&dir, ImageFormat::Jpeg, b"another").unwrap();
        assert_eq!(first.path, again.path);
        assert!(first.path.ends_with(".png"));
        assert_eq!(first.mime, "image/png");
        assert_eq!(std::fs::read(&first.path).unwrap(), b"not really a png");
        assert!(other.path.ends_with(".jpg"));
        assert_ne!(first.path, other.path);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
