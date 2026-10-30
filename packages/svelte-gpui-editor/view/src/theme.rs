//! The colours the editor paints with. The app's theme sends them as the editor's `theme` prop
//! and sends them again when the theme changes, so the editor follows it live. Until the first
//! one arrives the editor paints with the defaults here.
//!
//! The prop looks like this, with every colour `#rrggbb` or `#rrggbbaa`:
//!
//! ```json
//! { "background": "#1c1c1e", "foreground": "#fafafa", "currentLine": "#1f1f23",
//!   "lineNumber": "#52525b", "currentLineNumber": "#a1a1aa", "selection": "#a78bfa40",
//!   "cursor": "#a78bfa", "error": "#f87171", "warning": "#fbbf24", "info": "#60a5fa",
//!   "hint": "#a1a1aa", "added": "#4ade80", "modified": "#60a5fa", "deleted": "#f87171",
//!   "inlay": "#8b8b94", "lineEnd": "#71717a",
//!   "syntax": { "keyword": "#a78bfa", "title": { "color": "#60a5fa", "bold": true } } }
//! ```
//!
//! `indentGuide` and `indentGuideActive` colour the guides at each indentation level and the one
//! around the cursor, `occurrence` is the background of the other places the word under the cursor
//! appears, and `jumpLabel`, `jumpLabelText` and `jumpMatch` colour the labels of the jump
//! command and the text they stand for.
//!
//! The nine after those are the colours of decorations (see `editor_core::decorations::Tone`): what
//! diagnostics, git changes, inlay hints and text after a line end are drawn in.
//!
//! Each key of `syntax` is one of `editor_core::syntax::HIGHLIGHT_NAMES`. A name the theme leaves
//! out takes the style of its dotted prefix (`function.method` of `function`), and text with
//! no style at all is painted in the foreground.
//!
//! The text is drawn in `fontFamily` at `fontSize` pixels, the editor's monospace font at 14
//! when they are left out. A style may name a `family` and a `size` of its own, as a prose theme
//! does for headings: `{ "title": { "color": "#fafafa", "bold": true, "size": 24 } }`. GPUI
//! shapes a line at one size, so a line is drawn at the largest size any text on it asks for,
//! and its rows are as tall as that size needs. The line numbers stay in the monospace font.

use editor_core::Tone;
use editor_core::syntax::{HIGHLIGHT_NAMES, HighlightId};
use serde_json::Value;
use svelte_gpui::gpui::{Hsla, Pixels, Rgba, SharedString, px, rgba};

use super::element::{FONT_FAMILY, FONT_SIZE};

/// How one kind of highlighted text is painted.
#[derive(Clone)]
pub struct HighlightStyle {
    pub color: Hsla,
    pub bold: bool,
    pub italic: bool,
    pub family: SharedString,
    /// The size the line it is on is drawn at, at least; none leaves the line at the text's size.
    pub size: Option<Pixels>,
}

impl HighlightStyle {
    /// Text in a colour, in the editor's monospace font.
    pub fn plain(color: Hsla) -> Self {
        HighlightStyle { color, bold: false, italic: false, family: FONT_FAMILY.into(), size: None }
    }
}

/// Every colour of the editor.
#[derive(Clone)]
pub struct EditorTheme {
    pub background: Hsla,
    pub foreground: Hsla,
    pub current_line: Hsla,
    pub line_number: Hsla,
    pub current_line_number: Hsla,
    pub selection: Hsla,
    pub cursor: Hsla,
    pub error: Hsla,
    pub warning: Hsla,
    pub info: Hsla,
    pub hint: Hsla,
    pub added: Hsla,
    pub modified: Hsla,
    pub deleted: Hsla,
    /// Text an inlay adds in a line.
    pub inlay: Hsla,
    /// Text added after the end of a line.
    pub line_end: Hsla,
    /// The indentation guides, and the one of the scope around the cursor.
    pub indent_guide: Hsla,
    pub indent_guide_active: Hsla,
    /// Behind the other occurrences of the word under the cursor.
    pub occurrence: Hsla,
    /// A jump label's background and text, and the background of the text it labels.
    pub jump_label: Hsla,
    pub jump_label_text: Hsla,
    pub jump_match: Hsla,
    /// The font the text is drawn in, and its size.
    pub font_family: SharedString,
    pub font_size: Pixels,
    /// The style of each highlight, by `HighlightId`.
    syntax: Vec<Option<HighlightStyle>>,
}

impl Default for EditorTheme {
    /// The dark colours the editor had before it followed the theme, with no syntax colours.
    fn default() -> Self {
        EditorTheme {
            background: hex(0x18181bff),
            foreground: hex(0xe4e4e7ff),
            current_line: hex(0x222226ff),
            line_number: hex(0x52525bff),
            current_line_number: hex(0xa1a1aaff),
            selection: hex(0x3b3b52ff),
            cursor: hex(0xf97316ff),
            error: hex(0xf87171ff),
            warning: hex(0xfbbf24ff),
            info: hex(0x60a5faff),
            hint: hex(0xa1a1aaff),
            added: hex(0x4ade80ff),
            modified: hex(0x60a5faff),
            deleted: hex(0xf87171ff),
            inlay: hex(0x8b8b94ff),
            line_end: hex(0x71717aff),
            indent_guide: hex(0xe4e4e71a),
            indent_guide_active: hex(0xa78bfa99),
            occurrence: hex(0xe4e4e71f),
            jump_label: hex(0xec4899ff),
            jump_label_text: hex(0x0b0b0dff),
            jump_match: hex(0x60a5fa59),
            font_family: FONT_FAMILY.into(),
            font_size: FONT_SIZE,
            syntax: vec![None; HIGHLIGHT_NAMES.len()],
        }
    }
}

impl EditorTheme {
    /// Reads the `theme` prop. A colour that is missing or not a colour keeps its default, so a
    /// half-written theme still draws.
    pub fn from_json(value: &Value) -> Self {
        let defaults = EditorTheme::default();
        let color = |key: &str, fallback: Hsla| match value.get(key).and_then(Value::as_str).and_then(parse_color) {
            Some(parsed) => parsed,
            None => fallback,
        };
        let mut font_family = defaults.font_family;
        if let Some(family) = value.get("fontFamily").and_then(Value::as_str) {
            font_family = SharedString::from(family.to_string());
        }
        let mut font_size = defaults.font_size;
        if let Some(size) = value.get("fontSize").and_then(Value::as_f64) {
            font_size = px(size as f32);
        }
        EditorTheme {
            background: color("background", defaults.background),
            foreground: color("foreground", defaults.foreground),
            current_line: color("currentLine", defaults.current_line),
            line_number: color("lineNumber", defaults.line_number),
            current_line_number: color("currentLineNumber", defaults.current_line_number),
            selection: color("selection", defaults.selection),
            cursor: color("cursor", defaults.cursor),
            error: color("error", defaults.error),
            warning: color("warning", defaults.warning),
            info: color("info", defaults.info),
            hint: color("hint", defaults.hint),
            added: color("added", defaults.added),
            modified: color("modified", defaults.modified),
            deleted: color("deleted", defaults.deleted),
            inlay: color("inlay", defaults.inlay),
            line_end: color("lineEnd", defaults.line_end),
            indent_guide: color("indentGuide", defaults.indent_guide),
            indent_guide_active: color("indentGuideActive", defaults.indent_guide_active),
            occurrence: color("occurrence", defaults.occurrence),
            jump_label: color("jumpLabel", defaults.jump_label),
            jump_label_text: color("jumpLabelText", defaults.jump_label_text),
            jump_match: color("jumpMatch", defaults.jump_match),
            syntax: syntax_styles(value.get("syntax"), &font_family),
            font_family,
            font_size,
        }
    }

    /// How text no highlight styles is painted.
    pub fn text_style(&self) -> HighlightStyle {
        HighlightStyle { family: self.font_family.clone(), ..HighlightStyle::plain(self.foreground) }
    }

    /// Whether the theme draws any text in another font or size than the editor's own, which
    /// makes rows as tall as their text needs rather than all one height.
    pub fn has_own_fonts(&self) -> bool {
        if self.font_family.as_ref() != FONT_FAMILY || self.font_size != FONT_SIZE {
            return true;
        }
        let mut styles = self.syntax.iter().flatten();
        styles.any(|style| style.size.is_some() || style.family.as_ref() != FONT_FAMILY)
    }

    /// The colour a decoration of a tone is drawn in.
    pub fn tone(&self, tone: Tone) -> Hsla {
        match tone {
            Tone::Error => self.error,
            Tone::Warning => self.warning,
            Tone::Info => self.info,
            Tone::Hint => self.hint,
            Tone::Added => self.added,
            Tone::Modified => self.modified,
            Tone::Deleted => self.deleted,
            Tone::Inlay => self.inlay,
            Tone::LineEnd => self.line_end,
            Tone::Occurrence => self.occurrence,
            Tone::AddedWord => self.added,
            Tone::DeletedWord => self.deleted,
            Tone::Filler => self.hint,
            Tone::SearchMatch => self.warning,
        }
    }

    /// The colour the theme gives a highlight name such as `function`, if it gives one.
    pub fn syntax_color(&self, name: &str) -> Option<Hsla> {
        let id = editor_core::syntax::highlight_for_capture(name)?;
        self.style_of(id).map(|style| style.color)
    }

    /// How a highlight is painted, or none for text in the foreground colour.
    pub fn style_of(&self, id: HighlightId) -> Option<&HighlightStyle> {
        self.syntax.get(id.0 as usize)?.as_ref()
    }
}

/// A colour part of the way from one colour to another, `amount` being 0 for the first and 1 for the second.
pub fn mix(from: Hsla, to: Hsla, amount: f32) -> Hsla {
    let (from, to): (Rgba, Rgba) = (from.into(), to.into());
    let blend = |first: f32, second: f32| first + (second - first) * amount;
    Rgba { r: blend(from.r, to.r), g: blend(from.g, to.g), b: blend(from.b, to.b), a: blend(from.a, to.a) }.into()
}

/// The style of every highlight name from the theme's `syntax` object, in `text_family` unless
/// a style names its own.
fn syntax_styles(syntax: Option<&Value>, text_family: &SharedString) -> Vec<Option<HighlightStyle>> {
    let Some(syntax) = syntax else {
        return vec![None; HIGHLIGHT_NAMES.len()];
    };
    HIGHLIGHT_NAMES.iter().map(|name| style_for_name(syntax, name, text_family)).collect()
}

/// The style the theme gives a highlight name, or else its dotted prefix.
fn style_for_name(syntax: &Value, name: &str, text_family: &SharedString) -> Option<HighlightStyle> {
    let mut candidate = name;
    loop {
        if let Some(entry) = syntax.get(candidate) {
            return parse_style(entry, text_family);
        }
        let (shorter, _) = candidate.rsplit_once('.')?;
        candidate = shorter;
    }
}

/// A style written as a colour, or as an object with a colour, whether it is bold or italic,
/// and the font family and size it asks for.
fn parse_style(entry: &Value, text_family: &SharedString) -> Option<HighlightStyle> {
    let plain = HighlightStyle { family: text_family.clone(), ..HighlightStyle::plain(Hsla::default()) };
    if let Some(text) = entry.as_str() {
        return Some(HighlightStyle { color: parse_color(text)?, ..plain });
    }
    let color = parse_color(entry.get("color")?.as_str()?)?;
    let bold = entry.get("bold").and_then(Value::as_bool).unwrap_or(false);
    let italic = entry.get("italic").and_then(Value::as_bool).unwrap_or(false);
    let mut family = plain.family;
    if let Some(named) = entry.get("family").and_then(Value::as_str) {
        family = SharedString::from(named.to_string());
    }
    let size = entry.get("size").and_then(Value::as_f64).map(|size| px(size as f32));
    Some(HighlightStyle { color, bold, italic, family, size })
}

/// A colour from `#rrggbb` or `#rrggbbaa`.
fn parse_color(text: &str) -> Option<Hsla> {
    let digits = text.strip_prefix('#')?;
    let value = u32::from_str_radix(digits, 16).ok()?;
    match digits.len() {
        6 => Some(hex((value << 8) | 0xff)),
        8 => Some(hex(value)),
        _ => None,
    }
}

/// A colour from its `0xRRGGBBAA` value.
fn hex(value: u32) -> Hsla {
    rgba(value).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_core::syntax::highlight_for_capture;
    use serde_json::json;

    fn id(name: &str) -> HighlightId {
        highlight_for_capture(name).unwrap()
    }

    #[test]
    fn colours_come_from_the_prop_and_fall_back_to_the_defaults() {
        let theme = EditorTheme::from_json(&json!({ "background": "#ffffff", "foreground": "nonsense" }));
        assert_eq!(theme.background, hex(0xffffffff));
        assert_eq!(theme.foreground, EditorTheme::default().foreground);
    }

    #[test]
    fn a_highlight_without_a_colour_takes_its_prefixs_and_else_none() {
        let theme = EditorTheme::from_json(&json!({
            "syntax": { "function": "#112233", "title": { "color": "#445566", "bold": true } }
        }));
        assert_eq!(theme.style_of(id("function.method")).unwrap().color, hex(0x112233ff));
        let title = theme.style_of(id("title")).unwrap();
        assert!(title.bold && !title.italic);
        assert!(theme.style_of(id("keyword")).is_none());
    }

    #[test]
    fn styles_take_the_texts_font_unless_they_name_their_own() {
        let theme = EditorTheme::from_json(&json!({
            "fontFamily": "Inter",
            "fontSize": 15,
            "syntax": { "keyword": "#112233", "title": { "color": "#445566", "size": 24, "family": "Lora" } }
        }));
        assert_eq!(theme.font_size, px(15.0));
        assert_eq!(theme.style_of(id("keyword")).unwrap().family.as_ref(), "Inter");
        let title = theme.style_of(id("title")).unwrap();
        assert_eq!((title.family.as_ref(), title.size), ("Lora", Some(px(24.0))));
        assert!(theme.has_own_fonts());
    }

    #[test]
    fn a_code_theme_has_no_fonts_of_its_own() {
        let theme = EditorTheme::from_json(&json!({ "syntax": { "keyword": { "color": "#112233", "bold": true } } }));
        assert!(!theme.has_own_fonts());
        assert!(!EditorTheme::default().has_own_fonts());
    }

    #[test]
    fn decoration_colours_come_from_the_prop_by_tone() {
        let theme = EditorTheme::from_json(&json!({ "error": "#010203", "lineEnd": "#040506" }));
        assert_eq!(theme.tone(Tone::Error), hex(0x010203ff));
        assert_eq!(theme.tone(Tone::LineEnd), hex(0x040506ff));
        assert_eq!(theme.tone(Tone::Added), EditorTheme::default().added);
    }

    #[test]
    fn mixing_moves_between_two_colours() {
        let black = hex(0x000000ff);
        let white = hex(0xffffffff);
        assert_eq!(mix(black, white, 0.0), black);
        assert_eq!(mix(black, white, 1.0), white);
        assert!((mix(black, white, 0.5).l - 0.5).abs() < 0.01);
    }

    #[test]
    fn alpha_is_read_from_eight_digit_colours() {
        assert_eq!(parse_color("#a78bfa40").unwrap(), hex(0xa78bfa40));
        assert!(parse_color("#abc").is_none());
    }
}
