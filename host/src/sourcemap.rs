//! Source maps: where a position in the bundle came from in the app's own files, so an
//! error's stack can name `Files.svelte:42` instead of a line of generated code.
//!
//! Only what that needs is read: the `sources`, their text when the map holds it, and the
//! `mappings`, which are decoded once into a sorted table per generated line.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// One mapping on a generated line: from a column of the bundle to a place in a source.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Segment {
    generated_column: u32,
    source: u32,
    line: u32,
    column: u32,
}

/// Where a position of the bundle came from. Lines and columns count from 1.
#[derive(Clone, Debug, PartialEq)]
pub struct Origin {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
}

pub struct SourceMap {
    sources: Vec<PathBuf>,
    contents: Vec<Option<String>>,
    lines: Vec<Vec<Segment>>,
}

impl SourceMap {
    /// Reads a map from the JSON text esbuild wrote beside the bundle. `directory` is where the
    /// map's `sources` are relative to. `None` if the text is not a source map.
    pub fn parse(text: &str, directory: &Path) -> Option<Self> {
        let json: Value = serde_json::from_str(text).ok()?;
        let sources =
            json["sources"].as_array()?.iter().map(|source| directory.join(source.as_str().unwrap_or(""))).collect();
        let contents = match json["sourcesContent"].as_array() {
            Some(list) => list.iter().map(|content| content.as_str().map(str::to_string)).collect(),
            None => Vec::new(),
        };
        let lines = decode(json["mappings"].as_str()?)?;
        Some(SourceMap { sources, contents, lines })
    }

    /// The source position of a bundle position, both counting from 1.
    pub fn origin(&self, line: usize, column: usize) -> Option<Origin> {
        let segments = self.lines.get(line.checked_sub(1)?)?;
        let wanted = column.saturating_sub(1) as u32;
        let after = segments.partition_point(|segment| segment.generated_column <= wanted);
        let segment = segments.get(after.checked_sub(1)?)?;
        let path = normalize(self.sources.get(segment.source as usize)?);
        Some(Origin { path, line: segment.line as usize + 1, column: segment.column as usize + 1 })
    }

    /// The text of line `line` (from 1) of a source, when the map carries the sources' text.
    pub fn source_line(&self, path: &Path, line: usize) -> Option<String> {
        let index = self.sources.iter().position(|source| normalize(source) == path)?;
        let content = self.contents.get(index)?.as_ref()?;
        content.lines().nth(line.checked_sub(1)?).map(str::to_string)
    }
}

/// Resolves `.` and `..` in a path without touching the file system.
fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {}
            other => result.push(other),
        }
    }
    result
}

/// Decodes the `mappings` string: lines separated by `;`, segments by `,`, each a run of
/// base-64 variable-length quantities holding differences from the previous segment.
fn decode(mappings: &str) -> Option<Vec<Vec<Segment>>> {
    let mut lines = Vec::new();
    let (mut source, mut line, mut column) = (0i64, 0i64, 0i64);
    for text in mappings.split(';') {
        let mut generated = 0i64;
        let mut segments = Vec::new();
        for part in text.split(',').filter(|part| !part.is_empty()) {
            let numbers = quantities(part)?;
            generated += *numbers.first()?;
            // A segment of one number has no source: it marks code the build made up.
            if numbers.len() >= 4 {
                source += numbers[1];
                line += numbers[2];
                column += numbers[3];
                segments.push(Segment {
                    generated_column: generated.max(0) as u32,
                    source: source.max(0) as u32,
                    line: line.max(0) as u32,
                    column: column.max(0) as u32,
                });
            }
        }
        lines.push(segments);
    }
    Some(lines)
}

/// The numbers one segment holds.
fn quantities(segment: &str) -> Option<Vec<i64>> {
    let mut numbers = Vec::new();
    let (mut value, mut shift) = (0i64, 0);
    for character in segment.bytes() {
        let digit = match character {
            b'A'..=b'Z' => character - b'A',
            b'a'..=b'z' => character - b'a' + 26,
            b'0'..=b'9' => character - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as i64;
        value |= (digit & 31) << shift;
        if digit & 32 != 0 {
            shift += 5;
            continue;
        }
        let negative = value & 1 == 1;
        let magnitude = value >> 1;
        numbers.push(if negative { -magnitude } else { magnitude });
        value = 0;
        shift = 0;
    }
    Some(numbers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_quantities_of_a_segment() {
        assert_eq!(quantities("AAAA"), Some(vec![0, 0, 0, 0]));
        assert_eq!(quantities("AACA"), Some(vec![0, 0, 1, 0]));
        assert_eq!(quantities("gB"), Some(vec![16]));
        assert_eq!(quantities("D"), Some(vec![-1]));
    }

    #[test]
    fn finds_where_a_position_came_from() {
        let text = r#"{"version":3,"sources":["../src/a.svelte"],"sourcesContent":["one\ntwo\n"],"mappings":"AAAA;AACA,EAAE"}"#;
        let map = SourceMap::parse(text, Path::new("/app/dist")).expect("a map");
        let origin = map.origin(2, 3).expect("a position");
        assert_eq!(origin, Origin { path: PathBuf::from("/app/src/a.svelte"), line: 2, column: 3 });
        assert_eq!(map.source_line(&origin.path, 2).as_deref(), Some("two"));
        assert_eq!(map.origin(9, 1), None);
    }
}
