//! Soft wrapping, which the `wrap` prop turns on. A line too wide for the view is broken where
//! GPUI's text system says it no longer fits, and stays one display row, a `ROW_HEIGHT` taller
//! for each piece after the first. The view needs every line's height to know how far it
//! scrolls, not only the heights of the lines in view, so the pieces of each line are counted
//! once per text and width and kept.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use svelte_gpui::gpui::{Pixels, Window, point, px};

use super::Editor;
use super::element::{VisibleLine, shape_wrapped_plain};
use super::rows::{ROW_HEIGHT, RowLayout};
use editor_core::display::{BlockMap, DisplayLine, DisplayRow};
use editor_core::{Command, EditorState, Motion, Position};

/// The narrowest a line is wrapped to, so a view squeezed to nothing does not break every character.
const NARROWEST_WRAP: Pixels = px(80.0);
/// Space kept free between the end of the longest piece and whatever is right of the text.
const RIGHT_MARGIN: Pixels = px(12.0);

/// How many pieces each line's text wraps into at the width the view last had.
#[derive(Default)]
pub struct WrapCache {
    width: Pixels,
    /// Pieces by a hash of the line's drawn text.
    pieces: HashMap<u64, usize>,
}

impl WrapCache {
    /// The rows of a document wrapped to `width` (see `wrap_width`): each line as tall as its
    /// pieces, and each row of a block one `ROW_HEIGHT`. Inlays are left out of the count.
    pub fn row_layout(
        &mut self,
        state: &EditorState,
        blocks: &BlockMap,
        width: Pixels,
        window: &mut Window,
    ) -> RowLayout {
        if width != self.width {
            self.width = width;
            self.pieces.clear();
        }
        let buffer = state.buffer();
        let total_rows = blocks.total_rows(buffer.line_count());
        let mut counted = HashMap::with_capacity(buffer.line_count());
        let mut heights = Vec::with_capacity(total_rows);
        for display_row in 0..total_rows {
            let DisplayRow::Text(row) = blocks.row_at(display_row) else {
                heights.push(ROW_HEIGHT);
                continue;
            };
            let text = DisplayLine::new(&buffer.line(row)).text;
            let key = hash_of(&text);
            let pieces = match self.pieces.get(&key) {
                Some(pieces) => *pieces,
                None => shape_wrapped_plain(window, &text, width).wrap_boundaries().len() + 1,
            };
            counted.insert(key, pieces);
            heights.push(ROW_HEIGHT * pieces as f32);
        }
        // Only the texts the document still has are kept, so the cache does not grow with every edit.
        self.pieces = counted;
        RowLayout::with_heights(heights)
    }
}

/// The width lines are wrapped to when the text has `room` from its left edge to the edge of
/// the view, or of the minimap.
pub fn wrap_width(room: Pixels) -> Pixels {
    (room - RIGHT_MARGIN).max(NARROWEST_WRAP)
}

impl Editor {
    /// The command a key's command becomes while lines wrap: the standard keymap's up and down
    /// move through the pieces of a line as they are drawn before going on to the next line,
    /// and Home and End go to the ends of the piece the head is on. Also gives the goals up and
    /// down aim for after the move. Other commands stay as they are.
    pub(super) fn through_wrapped_lines(
        &self,
        command: &Command,
        window: &mut Window,
    ) -> (Command, Option<Vec<Pixels>>) {
        let unchanged = (command.clone(), None);
        let Command::MoveSelections { motion, extend } = command else {
            return unchanged;
        };
        let Some(width) = self.layout.as_ref().and_then(|layout| layout.wrap_width) else {
            return unchanged;
        };
        let moved = self.read(|state| match motion {
            Motion::Vertical(rows @ (-1 | 1)) => {
                let (heads, goals) = vertical_heads(state, *rows > 0, &self.wrap_goals, width, window);
                Some((heads, Some(goals)))
            }
            Motion::LineStart => Some((piece_edge_heads(state, false, width, window), None)),
            Motion::LineEnd => Some((piece_edge_heads(state, true, width, window), None)),
            _ => None,
        });
        let Some(Some((heads, goals))) = moved else {
            return unchanged;
        };
        (Command::MoveHeads { heads, extend: *extend }, goals)
    }
}

/// A buffer line shaped and wrapped as the view draws it, leaving out its inlays.
fn wrapped_line(state: &EditorState, row: usize, width: Pixels, window: &mut Window) -> VisibleLine {
    let display = DisplayLine::new(&state.buffer().line(row));
    let shaped = shape_wrapped_plain(window, &display.text, width);
    VisibleLine { row, display, shaped }
}

/// Where up or down takes each selection's head, and how far right each aims: as far as on the
/// move before when `goals` has one for every selection, or else where the head is now.
fn vertical_heads(
    state: &EditorState,
    down: bool,
    goals: &[Pixels],
    width: Pixels,
    window: &mut Window,
) -> (Vec<Position>, Vec<Pixels>) {
    let selections = state.selections().all();
    let mut heads = Vec::with_capacity(selections.len());
    let mut aims = Vec::with_capacity(selections.len());
    for (index, selection) in selections.iter().enumerate() {
        let line = wrapped_line(state, selection.head.row, width, window);
        let offset = line.display.offset_of(selection.head.column);
        let mut goal = line.point_of(offset).x;
        if goals.len() == selections.len() {
            goal = goals[index];
        }
        heads.push(piece_above_or_below(state, &line, line.piece_of(offset), down, goal, width, window));
        aims.push(goal);
    }
    (heads, aims)
}

/// The place `goal` from the left on the piece above or below `piece`, which may be on the line
/// before or after. At the first or last line there is none, and the head stays on its piece.
fn piece_above_or_below(
    state: &EditorState,
    line: &VisibleLine,
    piece: usize,
    down: bool,
    goal: Pixels,
    width: Pixels,
    window: &mut Window,
) -> Position {
    if down && piece + 1 < line.piece_count() {
        return position_in(line, piece + 1, goal);
    }
    if !down && piece > 0 {
        return position_in(line, piece - 1, goal);
    }
    if down && line.row + 1 < state.buffer().line_count() {
        let next = wrapped_line(state, line.row + 1, width, window);
        return position_in(&next, 0, goal);
    }
    if !down && line.row > 0 {
        let previous = wrapped_line(state, line.row - 1, width, window);
        return position_in(&previous, previous.piece_count() - 1, goal);
    }
    position_in(line, piece, goal)
}

/// Where Home or End takes each selection's head: the start or the end of the piece it is on.
fn piece_edge_heads(state: &EditorState, end: bool, width: Pixels, window: &mut Window) -> Vec<Position> {
    let mut heads = Vec::new();
    for selection in state.selections().all() {
        let line = wrapped_line(state, selection.head.row, width, window);
        let piece = line.piece_of(line.display.offset_of(selection.head.column));
        let x = if end { Pixels::MAX } else { px(0.0) };
        heads.push(position_in(&line, piece, x));
    }
    heads
}

/// The buffer position nearest `x` from the left on a piece of a line.
fn position_in(line: &VisibleLine, piece: usize, x: Pixels) -> Position {
    let offset = line.offset_at(point(x, ROW_HEIGHT * piece as f32));
    Position { row: line.row, column: line.display.column_at(offset) }
}

fn hash_of(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
