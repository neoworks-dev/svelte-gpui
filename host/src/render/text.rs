//! Text: paragraphs gathered from nested `g-text` runs, with what they inherit.

use gpui::prelude::*;
use gpui::{AnyElement, Div, FontWeight, Hsla, IntoElement, Rgba, SharedString, StyledText, div, px};

use super::Painter;
use crate::style::{SANS, Style};
use crate::tree::{Kind, Node, NodeId};

/// What text inherits from the elements above it.
#[derive(Clone, PartialEq)]
pub struct Inherited {
    pub(super) font_size: f32,
    pub(super) colour: Rgba,
    pub(super) weight: f32,
    pub(super) family: SharedString,
    pub(super) italic: bool,
    pub(super) underline: bool,
    pub(super) strikethrough: bool,
    pub(super) uppercase: bool,
}

impl Inherited {
    pub fn root() -> Self {
        Inherited {
            font_size: DEFAULT_FONT_SIZE,
            colour: DEFAULT_TEXT,
            weight: 400.0,
            family: SharedString::from(SANS),
            italic: false,
            underline: false,
            strikethrough: false,
            uppercase: false,
        }
    }

    pub(super) fn under(&self, style: &Style) -> Self {
        Inherited {
            font_size: style.font_size.unwrap_or(self.font_size),
            colour: style.color.unwrap_or(self.colour),
            weight: style.font_weight.unwrap_or(self.weight),
            family: style.font_family.clone().unwrap_or_else(|| self.family.clone()),
            italic: style.italic.unwrap_or(self.italic),
            underline: style.underline.unwrap_or(self.underline),
            strikethrough: style.strikethrough.unwrap_or(self.strikethrough),
            uppercase: style.uppercase.unwrap_or(self.uppercase),
        }
    }
}

pub(super) const DEFAULT_FONT_SIZE: f32 = 14.0;

pub(super) const DEFAULT_TEXT: Rgba = Rgba { r: 0.957, g: 0.957, b: 0.961, a: 1.0 };

impl Painter<'_> {
    /// Text and nested `g-text` elements drawn as one wrapping paragraph, each nested
    /// element's weight, colour and font applied to its stretch of the text.
    pub(super) fn text_content(
        &self,
        element: Div,
        id: NodeId,
        node: &Node,
        style: &Style,
        inherited: &Inherited,
    ) -> Div {
        let preserve = node.attributes.contains_key("pre");
        let mut text = String::new();
        let mut runs: Vec<gpui::TextRun> = Vec::new();
        self.flow(id, inherited, preserve, &mut text, &mut runs);
        if !preserve {
            // Whitespace at the ends of the paragraph is template indentation.
            let trimmed_start = text.len() - text.trim_start().len();
            let trimmed_end = text.trim_end().len();
            trim_runs(&mut text, &mut runs, trimmed_start, trimmed_end);
        }
        if text.is_empty() {
            return element;
        }
        let single_line = style.nowrap == Some(true) || style.truncate == Some(true);
        let mut element = element;
        if !single_line && !preserve {
            // Text may shrink and wrap, but not below its longest word, which GPUI cannot work
            // out for itself and would otherwise break between letters.
            let longest = text.split_whitespace().map(|word| word.chars().count()).max().unwrap_or(0);
            if text.chars().any(char::is_whitespace) {
                element = element.min_w(px(longest as f32 * inherited.font_size * 0.6));
            } else {
                element = element.whitespace_nowrap();
            }
        }
        if preserve {
            element = element.whitespace_nowrap();
        }
        element.child(StyledText::new(SharedString::from(text)).with_runs(runs))
    }

    fn flow(
        &self,
        id: NodeId,
        inherited: &Inherited,
        preserve: bool,
        text: &mut String,
        runs: &mut Vec<gpui::TextRun>,
    ) {
        let Some(node) = self.tree.nodes.get(&id) else {
            return;
        };
        for child in &node.children {
            let Some(child_node) = self.tree.nodes.get(child) else {
                continue;
            };
            match &child_node.kind {
                Kind::Text { data } => {
                    let piece = if preserve { data.clone() } else { collapse(data) };
                    // Collapsing continues across pieces, as it would in one string.
                    let piece = if !preserve && text.ends_with(' ') { piece.trim_start().to_string() } else { piece };
                    push_run(text, runs, &transform(&piece, inherited), inherited, None);
                }
                Kind::Element { tag } if tag == "g-text" => {
                    let computed = self.computed(*child);
                    if computed.style.hidden == Some(true) {
                        continue;
                    }
                    let here = inherited.under(&computed.style);
                    let before = runs.len();
                    self.flow(*child, &here, preserve, text, runs);
                    if let Some(background) = computed.style.background {
                        for run in &mut runs[before..] {
                            run.background_color = Some(Hsla::from(background));
                        }
                    }
                }
                Kind::Element { tag } => {
                    eprintln!("svelte-gpui: <{tag}> inside a g-text is not drawn; only text and g-text are");
                }
            }
        }
    }
}

pub(super) fn push_run(
    text: &mut String,
    runs: &mut Vec<gpui::TextRun>,
    piece: &str,
    inherited: &Inherited,
    background: Option<Rgba>,
) {
    if piece.is_empty() {
        return;
    }
    let mut font = gpui::font(inherited.family.clone());
    font.weight = FontWeight(inherited.weight);
    if inherited.italic {
        font.style = gpui::FontStyle::Italic;
    }
    let line = |on: bool| {
        on.then(|| gpui::UnderlineStyle { thickness: px(1.0), color: Some(inherited.colour.into()), wavy: false })
    };
    text.push_str(piece);
    runs.push(gpui::TextRun {
        len: piece.len(),
        font,
        color: inherited.colour.into(),
        background_color: background.map(Hsla::from),
        underline: line(inherited.underline),
        strikethrough: inherited
            .strikethrough
            .then(|| gpui::StrikethroughStyle { thickness: px(1.0), color: Some(inherited.colour.into()) }),
    });
}

/// Cuts a paragraph down to `start..end`, shortening the runs to match.
pub(super) fn trim_runs(text: &mut String, runs: &mut Vec<gpui::TextRun>, start: usize, end: usize) {
    if start == 0 && end == text.len() {
        return;
    }
    let mut kept = Vec::new();
    let mut offset = 0;
    for run in runs.drain(..) {
        let run_start = offset.max(start);
        let run_end = (offset + run.len).min(end);
        offset += run.len;
        if run_end > run_start {
            kept.push(gpui::TextRun { len: run_end - run_start, ..run });
        }
    }
    *runs = kept;
    *text = text[start..end.max(start)].to_string();
}

pub(super) fn transform(text: &str, inherited: &Inherited) -> String {
    if inherited.uppercase { text.to_uppercase() } else { text.to_string() }
}

/// A run of text that can shrink and wrap, but not below its longest word.
pub(super) fn text_box(shown: String, font_size: f32) -> AnyElement {
    let longest = shown.split_whitespace().map(|word| word.chars().count()).max().unwrap_or(0);
    let mut text = div();
    if shown.chars().any(char::is_whitespace) {
        text = text.min_w(px(longest as f32 * font_size * 0.6));
    } else {
        text = text.whitespace_nowrap();
    }
    text.child(SharedString::from(shown)).into_any_element()
}

/// Collapses runs of whitespace to one space.
pub(super) fn collapse(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous_space = false;
    for character in text.chars() {
        if character.is_whitespace() {
            if !previous_space {
                out.push(' ');
            }
            previous_space = true;
            continue;
        }
        out.push(character);
        previous_space = false;
    }
    out
}
