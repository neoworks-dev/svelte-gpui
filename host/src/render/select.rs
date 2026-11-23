//! Selectable text: where each paragraph that can be selected was drawn, so a pointer can be
//! turned into a place in its text, and the pieces of that that need no window.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;

use gpui::prelude::*;
use gpui::{
    Context, Div, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, Rgba, Stateful, TextLayout, TextRun,
};

use super::Painter;
use super::inline::FlowLayout;
use crate::tree::NodeId;
use crate::workbench::Workbench;

impl Painter<'_> {
    /// Mouse handling for a `Text` with `selectable`: a press starts a selection. The window
    /// follows the pointer from there (`press.rs`), as it must when the pointer leaves the text. The
    /// press goes on to the elements around it, so a button around the text is still pressed.
    pub(super) fn wire_text(&self, element: Stateful<Div>, id: NodeId, cx: &mut Context<Workbench>) -> Stateful<Div> {
        element
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |workbench, event: &MouseDownEvent, _window, cx| {
                    workbench.text_mouse_down(id, event.position, event.click_count, event.modifiers.shift, cx);
                }),
            )
            // Only a hovered element hears a move, so while a drag goes on this says which paragraph it is over.
            .on_mouse_move(cx.listener(move |workbench, event: &MouseMoveEvent, _window, _cx| {
                workbench.text_pointer_over(id, event.position);
            }))
    }
}

/// Where a paragraph drew its text, which is how a pointer finds a place in it.
#[derive(Clone)]
pub enum Source {
    /// One `StyledText`, whose layout GPUI keeps.
    Plain(TextLayout),
    /// A paragraph with boxes in it, whose lines `inline.rs` laid out.
    Flow(FlowLayout),
}

/// A `Text` with `selectable`, as the last frame drew it: the text it shows, and where.
pub struct Selectable {
    pub text: String,
    pub source: Source,
    /// What each box in the text copies as, by the offset it sits at, in order. An empty string is
    /// a box that copies as nothing.
    pub copies: Vec<(usize, String)>,
}

impl Selectable {
    /// The byte offset in the text a point in the window is at. With `nearest` it is the edge between
    /// characters closest to the point, which is where a selection's end goes; without it, the
    /// start of the character under the point, which is what a double click takes the word of. A
    /// point outside the paragraph gives the nearest end, as dragging past the end of a line does.
    pub fn index_at(&self, position: Point<Pixels>, nearest: bool) -> usize {
        let index = match &self.source {
            Source::Plain(layout) => match layout.index_for_position(position) {
                Ok(index) if nearest => {
                    // GPUI gives the character under the point; past its middle, the edge after it is closer.
                    let after = index + self.text[snap(&self.text, index)..].chars().next().map_or(0, char::len_utf8);
                    match (layout.position_for_index(index), layout.position_for_index(after)) {
                        (Some(here), Some(next)) if here.y == next.y && position.x > (here.x + next.x) / 2.0 => after,
                        _ => index,
                    }
                }
                Ok(index) | Err(index) => index,
            },
            Source::Flow(flow) => flow.index_for_position(position, nearest),
        };
        snap(&self.text, index)
    }

    /// What `range` of the text copies as.
    pub fn copy(&self, range: Range<usize>) -> String {
        copy_text(&self.text, &self.copies, range)
    }
}

/// A place in a selectable paragraph: a byte offset in its text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub id: NodeId,
    pub index: usize,
}

/// What a selection grows by as the pointer drags: characters, or the words or paragraphs that
/// a double or triple click started from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unit {
    Char,
    Word,
    Paragraph,
}

/// What is selected: from the anchor, which stays, to the cursor, which the pointer moves, in one
/// paragraph or across several.
#[derive(Clone, Debug)]
pub struct TextSelection {
    pub anchor: Spot,
    pub cursor: Spot,
    pub unit: Unit,
    /// The word or paragraph a double or triple click took, which dragging grows from and never
    /// shrinks below. For a single click it is the anchor, twice.
    pub first: (Spot, Spot),
    /// The button is still down, so moving the pointer moves the cursor.
    pub dragging: bool,
    /// What it covers, paragraph by paragraph in document order: the paragraphs that were drawn
    /// the last time it was worked out, and the byte range of each that is selected.
    pub parts: Vec<(NodeId, Range<usize>)>,
    ranges: HashMap<NodeId, Range<usize>>,
}

impl TextSelection {
    pub fn new(anchor: Spot, cursor: Spot, unit: Unit, first: (Spot, Spot), dragging: bool) -> Self {
        TextSelection { anchor, cursor, unit, first, dragging, parts: Vec::new(), ranges: HashMap::new() }
    }

    /// Replaces what the selection covers.
    pub fn set_parts(&mut self, parts: Vec<(NodeId, Range<usize>)>) {
        self.ranges = parts.iter().cloned().collect();
        self.parts = parts;
    }

    /// The part of paragraph `id` that is selected.
    pub fn range_in(&self, id: NodeId) -> Option<&Range<usize>> {
        self.ranges.get(&id)
    }
}

/// The paragraphs whose selected part differs between two selections, for drawing them again.
pub fn changed(before: Option<&TextSelection>, after: Option<&TextSelection>) -> Vec<NodeId> {
    let empty = HashMap::new();
    let ranges = |selection: Option<&TextSelection>| selection.map_or(&empty, |selection| &selection.ranges).clone();
    let (before, after) = (ranges(before), ranges(after));
    let mut out: Vec<NodeId> =
        after.iter().filter(|(id, range)| before.get(id) != Some(range)).map(|(id, _)| *id).collect();
    out.extend(before.keys().filter(|id| !after.contains_key(id)));
    out
}

/// Which of two places comes first in the document, given the selectable paragraphs in document order.
pub fn compare(a: Spot, b: Spot, order: &[NodeId]) -> Ordering {
    if a.id == b.id {
        return a.index.cmp(&b.index);
    }
    let rank = |id| order.iter().position(|candidate| *candidate == id);
    rank(a.id).cmp(&rank(b.id))
}

/// The word or paragraph, or the place, that the pointer is at, as a start and an end.
pub fn unit_at(unit: Unit, spot: Spot, text: &str) -> (Spot, Spot) {
    let (start, end) = match unit {
        Unit::Char => (spot.index, spot.index),
        Unit::Word => {
            let word = word_at(text, spot.index);
            (word.start, word.end)
        }
        Unit::Paragraph => (0, text.len()),
    };
    (Spot { id: spot.id, index: start }, Spot { id: spot.id, index: end })
}

/// The anchor and cursor a drag has when the pointer is at `pointer`, a start and an end from
/// `unit_at`. A drag by characters keeps its anchor. One by words or paragraphs keeps the first
/// word or paragraph whole, and grows from it towards the pointer.
pub fn resolve(unit: Unit, first: (Spot, Spot), anchor: Spot, pointer: (Spot, Spot), order: &[NodeId]) -> (Spot, Spot) {
    if unit == Unit::Char {
        return (anchor, pointer.1);
    }
    if compare(pointer.0, first.0, order) == Ordering::Less { (first.1, pointer.0) } else { (first.0, pointer.1) }
}

/// The selected stretch of each paragraph from `start` to `end`, which must not be the other way
/// round: the rest of the first, all of those between, and the start of the last. `order` lists the
/// selectable paragraphs in document order and `length` says how long each one's text is.
pub fn parts(
    start: Spot,
    end: Spot,
    order: &[NodeId],
    length: impl Fn(NodeId) -> usize,
) -> Vec<(NodeId, Range<usize>)> {
    let mut out = Vec::new();
    let mut keep = |id: NodeId, from: usize, to: usize| {
        let to = to.min(length(id));
        if from < to {
            out.push((id, from..to));
        }
    };
    if start.id == end.id {
        keep(start.id, start.index, end.index);
        return out;
    }
    let position = |id| order.iter().position(|candidate| *candidate == id);
    let (Some(first), Some(last)) = (position(start.id), position(end.id)) else {
        return out;
    };
    for (offset, id) in order.iter().enumerate().take(last + 1).skip(first) {
        let from = if offset == first { start.index } else { 0 };
        let to = if offset == last { end.index } else { usize::MAX };
        keep(*id, from, to);
    }
    out
}

/// Forgets paragraphs that were removed, and drops the selection when one of its ends was in one.
/// Whether the selection was dropped.
pub fn forget(texts: &mut HashMap<NodeId, Selectable>, selection: &mut Option<TextSelection>, gone: &[NodeId]) -> bool {
    for id in gone {
        texts.remove(id);
    }
    let endpoint_gone = selection
        .as_ref()
        .is_some_and(|selection| [selection.anchor.id, selection.cursor.id].iter().any(|id| !texts.contains_key(id)));
    if endpoint_gone {
        *selection = None;
    }
    endpoint_gone
}

/// Whether a box at offset `at` of a text `length` bytes long is inside a selection of `range`:
/// one at the very end of the text counts as part of what ends there.
pub fn box_selected(range: &Range<usize>, at: usize, length: usize) -> bool {
    range.start <= at && (at < range.end || (at == range.end && range.end == length && range.start < range.end))
}

/// What `range` of a paragraph's text copies as. A box in it copies as what it says (`copies`, by the
/// offset it sits at, sorted), and one that says nothing copies as nothing, without leaving the two
/// spaces around it.
pub fn copy_text(text: &str, copies: &[(usize, String)], range: Range<usize>) -> String {
    let range = within(text, range);
    let mut out = String::new();
    let mut silent = false;
    let push = |out: &mut String, chunk: &str, silent: &mut bool| {
        let chunk = if *silent && (out.is_empty() || out.ends_with(' ')) {
            chunk.strip_prefix(' ').unwrap_or(chunk)
        } else {
            chunk
        };
        if !chunk.is_empty() {
            out.push_str(chunk);
            *silent = false;
        }
    };
    let mut from = range.start;
    for (at, copy) in copies {
        if !box_selected(&range, *at, text.len()) {
            continue;
        }
        let at = within(text, *at..*at).start.max(from);
        push(&mut out, &text[from..at], &mut silent);
        from = at;
        if copy.is_empty() {
            silent = true;
        } else {
            push(&mut out, copy, &mut silent);
        }
    }
    push(&mut out, &text[from..range.end], &mut silent);
    out
}

/// `range` as a stretch of `text`: inside it, and on character boundaries.
pub fn within(text: &str, range: Range<usize>) -> Range<usize> {
    snap(text, range.start)..snap(text, range.end)
}

/// The nearest character boundary at or before `index`, inside the text.
pub fn snap(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// The word around `index`, as a double click selects it: the run of letters and digits, of
/// spaces, or of other characters that the point is in.
pub fn word_at(text: &str, index: usize) -> Range<usize> {
    if text.is_empty() {
        return 0..0;
    }
    let mut at = snap(text, index);
    if at >= text.len() {
        // Past the end, the last character.
        at = text.char_indices().next_back().map_or(0, |(start, _)| start);
    }
    let class = |character: char| {
        if character.is_alphanumeric() || character == '_' {
            0
        } else if character.is_whitespace() {
            1
        } else {
            2
        }
    };
    let here = text[at..].chars().next().map_or(0, class);
    let start = text[..at].char_indices().rev().take_while(|(_, character)| class(*character) == here).last();
    let end = text[at..].char_indices().find(|(_, character)| class(*character) != here);
    start.map_or(at, |(start, _)| start)..end.map_or(text.len(), |(end, _)| at + end)
}

/// The runs of a paragraph with a background laid under `range`, which cuts the runs it starts
/// or ends in.
pub fn highlight_runs(runs: &[TextRun], range: Range<usize>, colour: Rgba) -> Vec<TextRun> {
    let mut out = Vec::with_capacity(runs.len() + 2);
    let mut offset = 0;
    for run in runs {
        let (start, end) = (offset, offset + run.len);
        offset = end;
        let (from, to) = (start.max(range.start), end.min(range.end));
        if to <= from {
            out.push(run.clone());
            continue;
        }
        if from > start {
            out.push(TextRun { len: from - start, ..run.clone() });
        }
        out.push(TextRun { len: to - from, background_color: Some(Hsla::from(colour)), ..run.clone() });
        if to < end {
            out.push(TextRun { len: end - to, ..run.clone() });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(len: usize) -> TextRun {
        TextRun {
            len,
            font: gpui::font("sans"),
            color: Hsla::default(),
            background_color: None,
            underline: None,
            strikethrough: None,
        }
    }

    #[test]
    fn a_double_click_takes_the_word_under_it() {
        let text = "Hello, brave new world";
        assert_eq!(&text[word_at(text, 0)], "Hello");
        assert_eq!(&text[word_at(text, 3)], "Hello");
        assert_eq!(&text[word_at(text, 5)], ",");
        assert_eq!(&text[word_at(text, 6)], " ");
        assert_eq!(&text[word_at(text, 8)], "brave");
        // Past the end it is the last word, and an empty text has none.
        assert_eq!(&text[word_at(text, 99)], "world");
        assert_eq!(word_at("", 3), 0..0);
    }

    #[test]
    fn a_word_is_cut_on_character_boundaries() {
        let text = "naïve café";
        // Byte 3 is inside the two bytes of "ï".
        assert_eq!(&text[word_at(text, 3)], "naïve");
        assert_eq!(&text[word_at(text, 100)], "café");
        assert_eq!(snap("é", 1), 0);
        assert_eq!(within("aé", 2..9), 1..3);
    }

    fn spot(id: NodeId, index: usize) -> Spot {
        Spot { id, index }
    }

    /// Every paragraph is ten bytes long.
    fn ten(_: NodeId) -> usize {
        10
    }

    #[test]
    fn a_selection_in_one_paragraph_is_the_stretch_between_its_ends() {
        assert_eq!(parts(spot(2, 3), spot(2, 8), &[1, 2, 3], ten), [(2, 3..8)]);
        assert!(parts(spot(2, 4), spot(2, 4), &[1, 2, 3], ten).is_empty());
    }

    #[test]
    fn a_selection_across_paragraphs_has_the_rest_of_the_first_all_between_and_the_start_of_the_last() {
        assert_eq!(parts(spot(1, 4), spot(3, 6), &[1, 2, 3, 4], ten), [(1, 4..10), (2, 0..10), (3, 0..6)]);
        // Paragraphs that are not selectable are not in the order, so they are skipped.
        assert_eq!(parts(spot(1, 4), spot(3, 6), &[1, 3, 4], ten), [(1, 4..10), (3, 0..6)]);
        // Nothing of a paragraph the selection only touches the edge of.
        assert!(parts(spot(1, 10), spot(2, 0), &[1, 2], ten).is_empty());
    }

    #[test]
    fn places_compare_by_paragraph_order_and_then_offset() {
        let order = [5, 3, 9];
        assert_eq!(compare(spot(5, 8), spot(3, 1), &order), Ordering::Less);
        assert_eq!(compare(spot(9, 0), spot(3, 7), &order), Ordering::Greater);
        assert_eq!(compare(spot(3, 2), spot(3, 7), &order), Ordering::Less);
        assert_eq!(compare(spot(3, 2), spot(3, 2), &order), Ordering::Equal);
    }

    #[test]
    fn dragging_by_words_keeps_the_first_word_and_grows_towards_the_pointer() {
        let order = [1, 2, 3];
        let first = (spot(2, 3), spot(2, 8));
        // Down into the next paragraph: from the start of the first word to the end of the word there.
        assert_eq!(resolve(Unit::Word, first, spot(2, 3), (spot(3, 0), spot(3, 5)), &order), (spot(2, 3), spot(3, 5)));
        // Up into the one before: from the end of the first word back to the start of that word.
        assert_eq!(resolve(Unit::Word, first, spot(2, 3), (spot(1, 2), spot(1, 6)), &order), (spot(2, 8), spot(1, 2)));
        // Back on the first word, it is that word.
        assert_eq!(resolve(Unit::Word, first, spot(2, 3), (spot(2, 3), spot(2, 8)), &order), (spot(2, 3), spot(2, 8)));
        // By characters the anchor stays where it was.
        let point = (spot(1, 4), spot(1, 4));
        assert_eq!(resolve(Unit::Char, (spot(2, 3), spot(2, 3)), spot(2, 3), point, &order), (spot(2, 3), spot(1, 4)));
    }

    #[test]
    fn a_triple_click_takes_the_paragraph_and_a_double_click_the_word() {
        assert_eq!(unit_at(Unit::Paragraph, spot(4, 6), "some text"), (spot(4, 0), spot(4, 9)));
        assert_eq!(unit_at(Unit::Word, spot(4, 6), "some text"), (spot(4, 5), spot(4, 9)));
        assert_eq!(unit_at(Unit::Char, spot(4, 6), "some text"), (spot(4, 6), spot(4, 6)));
    }

    #[test]
    fn drawing_again_is_for_the_paragraphs_whose_selected_part_changed() {
        let selection = |parts: Vec<(NodeId, Range<usize>)>| {
            let mut selection = TextSelection::new(spot(1, 0), spot(3, 1), Unit::Char, (spot(1, 0), spot(1, 0)), false);
            selection.set_parts(parts);
            selection
        };
        let before = selection(vec![(1, 2..10), (2, 0..10)]);
        let after = selection(vec![(1, 2..10), (2, 0..4), (3, 0..1)]);
        let mut moved = changed(Some(&before), Some(&after));
        moved.sort();
        assert_eq!(moved, [2, 3]);
        // A new selection changes every paragraph in it, and ending one the same.
        let mut all = changed(None, Some(&before));
        all.sort();
        assert_eq!(all, [1, 2]);
        let mut ended = changed(Some(&before), None);
        ended.sort();
        assert_eq!(ended, [1, 2]);
        // One that is gone must be drawn again too, to lose its highlight.
        assert_eq!(changed(Some(&before), Some(&selection(vec![(1, 2..10)]))), [2]);
    }

    fn paragraph() -> Selectable {
        Selectable { text: "text".to_string(), source: Source::Plain(TextLayout::default()), copies: Vec::new() }
    }

    #[test]
    fn a_removed_paragraph_is_forgotten_and_takes_a_selection_that_ended_in_it() {
        let mut texts: HashMap<NodeId, Selectable> = (1..=3).map(|id| (id, paragraph())).collect();
        let selected = || Some(TextSelection::new(spot(1, 0), spot(3, 2), Unit::Char, (spot(1, 0), spot(1, 0)), false));
        // One in the middle goes: the selection stays, over what is left.
        let mut selection = selected();
        assert!(!forget(&mut texts, &mut selection, &[2]));
        assert!(selection.is_some());
        assert!(!texts.contains_key(&2));
        // One of its ends goes: the selection does too, and the entry with it.
        assert!(forget(&mut texts, &mut selection, &[3]));
        assert!(selection.is_none());
        assert_eq!(texts.len(), 1);
        // Nothing selected, nothing to drop.
        assert!(!forget(&mut texts, &mut None, &[1]));
        assert!(texts.is_empty());
    }

    #[test]
    fn a_box_copies_as_what_it_says_and_one_that_says_nothing_leaves_no_double_space() {
        // "a star " then a box, then " in the line": two spaces around where the box is.
        let text = "a star  in the line";
        assert_eq!(copy_text(text, &[(7, String::new())], 0..text.len()), "a star in the line");
        assert_eq!(copy_text(text, &[(7, "*".to_string())], 0..text.len()), "a star * in the line");
        assert_eq!(copy_text(text, &[], 0..text.len()), text);
        // A box outside what is copied is not part of it.
        assert_eq!(copy_text(text, &[(7, "*".to_string())], 0..5), "a sta");
        assert_eq!(copy_text(text, &[(7, "*".to_string())], 8..text.len()), "in the line");
        // At the start and at the end of a paragraph.
        assert_eq!(copy_text(" in", &[(0, String::new())], 0..3), "in");
        assert_eq!(copy_text("end ", &[(4, "!".to_string())], 0..4), "end !");
        assert_eq!(copy_text("end ", &[(4, "!".to_string())], 0..2), "en");
    }

    #[test]
    fn the_highlight_cuts_the_runs_it_starts_and_ends_in() {
        let runs = [run(4), run(6), run(5)];
        let lengths =
            |runs: &[TextRun]| runs.iter().map(|run| (run.len, run.background_color.is_some())).collect::<Vec<_>>();
        let colour = Rgba { r: 0.2, g: 0.4, b: 0.6, a: 0.5 };
        // From inside the first run to inside the last.
        assert_eq!(
            lengths(&highlight_runs(&runs, 2..12, colour)),
            [(2, false), (2, true), (6, true), (2, true), (3, false)]
        );
        // Inside one run, and nothing at all.
        assert_eq!(
            lengths(&highlight_runs(&runs, 5..7, colour)),
            [(4, false), (1, false), (2, true), (3, false), (5, false)]
        );
        assert_eq!(lengths(&highlight_runs(&runs, 3..3, colour)), [(4, false), (6, false), (5, false)]);
        // The text is as long as before.
        assert_eq!(highlight_runs(&runs, 2..12, colour).iter().map(|run| run.len).sum::<usize>(), 15);
    }
}
