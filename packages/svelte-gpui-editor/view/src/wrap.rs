//! How tall each line is drawn, for soft wrapping and for themes with fonts of their own.
//!
//! The `wrap` prop breaks a line too wide for the view where GPUI's text system says it no
//! longer fits. The line stays one display row, as tall as its pieces. A theme may draw a line
//! larger (see `theme.rs`), which makes each piece taller. The view needs every line's height
//! to know how far it scrolls, not only the heights of the lines in view, so each line's height
//! is worked out once per text, highlights and width and kept.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use svelte_gpui::gpui::{Pixels, Window, point, px};

use super::Editor;
use super::element::{VisibleLine, shape_line};
use super::rows::{ROW_HEIGHT, RowLayout};
use super::theme::EditorTheme;
use editor_core::display::{BlockMap, DisplayLine, DisplayRow};
use editor_core::syntax::HighlightSpan;
use editor_core::{Command, EditorState, Motion, Position};

/// The narrowest a line is wrapped to, so a view squeezed to nothing does not break every character.
const NARROWEST_WRAP: Pixels = px(80.0);
/// Space kept free between the end of the longest piece and whatever is right of the text.
const RIGHT_MARGIN: Pixels = px(12.0);

/// How tall each line was drawn at the wrap width the view last had.
#[derive(Default)]
pub struct LineHeights {
    wrap_width: Option<Pixels>,
    /// Heights by a hash of the line's drawn text and its highlights.
    heights: HashMap<u64, Pixels>,
}

impl LineHeights {
    /// The rows of a document: each line as tall as `shaping` draws it, and each row of a block
    /// one `ROW_HEIGHT`. Inlays are left out.
    pub fn row_layout(
        &mut self,
        state: &EditorState,
        blocks: &BlockMap,
        shaping: &LineShaping,
        window: &mut Window,
    ) -> RowLayout {
        if shaping.wrap_width != self.wrap_width {
            self.wrap_width = shaping.wrap_width;
            self.heights.clear();
        }
        let buffer = state.buffer();
        let total_rows = blocks.total_rows(buffer.line_count());
        let mut highlights = Vec::new();
        if shaping.theme.has_own_fonts() {
            highlights = state.highlights(0..buffer.line_count());
        }
        let mut measured = HashMap::with_capacity(buffer.line_count());
        let mut heights = Vec::with_capacity(total_rows);
        for display_row in 0..total_rows {
            let DisplayRow::Text(row) = blocks.row_at(display_row) else {
                heights.push(ROW_HEIGHT);
                continue;
            };
            let display = DisplayLine::new(&buffer.line(row));
            let spans = highlights.get(row).map(Vec::as_slice).unwrap_or_default();
            let key = hash_of(&display.text, spans);
            let height = match self.heights.get(&key) {
                Some(height) => *height,
                None => {
                    let line = shape_line(window, row, display, spans, &[], shaping.theme, shaping.wrap_width);
                    line.piece_top(line.piece_count())
                }
            };
            measured.insert(key, height);
            heights.push(height);
        }
        // Only the lines the document still has are kept, so the cache does not grow with every edit.
        self.heights = measured;
        RowLayout::with_heights(heights)
    }

    /// Forgets every height, as when the theme's fonts change.
    pub fn clear(&mut self) {
        self.heights.clear();
    }
}

/// What lines are shaped with outside of drawing them: the theme's fonts and the width they
/// wrap to, if they wrap.
pub struct LineShaping<'a> {
    pub theme: &'a EditorTheme,
    pub wrap_width: Option<Pixels>,
}

impl LineShaping<'_> {
    /// A buffer line shaped as the view draws it, leaving out its inlays.
    fn line(&self, state: &EditorState, row: usize, window: &mut Window) -> VisibleLine {
        let display = DisplayLine::new(&state.buffer().line(row));
        let mut highlights = Vec::new();
        if self.theme.has_own_fonts() {
            highlights = state.highlights(row..row + 1);
        }
        let spans = highlights.first().map(Vec::as_slice).unwrap_or_default();
        shape_line(window, row, display, spans, &[], self.theme, self.wrap_width)
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
        let Some(wrap_width) = self.layout.as_ref().and_then(|layout| layout.wrap_width) else {
            return unchanged;
        };
        let shaping = LineShaping { theme: &self.theme, wrap_width: Some(wrap_width) };
        let moved = self.read(|state| match motion {
            Motion::Vertical(rows @ (-1 | 1)) => {
                let (heads, goals) = vertical_heads(state, *rows > 0, &self.wrap_goals, &shaping, window);
                Some((heads, Some(goals)))
            }
            Motion::LineStart => Some((piece_edge_heads(state, false, &shaping, window), None)),
            Motion::LineEnd => Some((piece_edge_heads(state, true, &shaping, window), None)),
            _ => None,
        });
        let Some(Some((heads, goals))) = moved else {
            return unchanged;
        };
        (Command::MoveHeads { heads, extend: *extend }, goals)
    }
}

/// Where up or down takes each selection's head, and how far right each aims: as far as on the
/// move before when `goals` has one for every selection, or else where the head is now.
fn vertical_heads(
    state: &EditorState,
    down: bool,
    goals: &[Pixels],
    shaping: &LineShaping,
    window: &mut Window,
) -> (Vec<Position>, Vec<Pixels>) {
    let selections = state.selections().all();
    let mut heads = Vec::with_capacity(selections.len());
    let mut aims = Vec::with_capacity(selections.len());
    for (index, selection) in selections.iter().enumerate() {
        let line = shaping.line(state, selection.head.row, window);
        let offset = line.display.offset_of(selection.head.column);
        let mut goal = line.point_of(offset).x;
        if goals.len() == selections.len() {
            goal = goals[index];
        }
        heads.push(piece_above_or_below(state, &line, line.piece_of(offset), down, goal, shaping, window));
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
    shaping: &LineShaping,
    window: &mut Window,
) -> Position {
    if down && piece + 1 < line.piece_count() {
        return position_in(line, piece + 1, goal);
    }
    if !down && piece > 0 {
        return position_in(line, piece - 1, goal);
    }
    if down && line.row + 1 < state.buffer().line_count() {
        let next = shaping.line(state, line.row + 1, window);
        return position_in(&next, 0, goal);
    }
    if !down && line.row > 0 {
        let previous = shaping.line(state, line.row - 1, window);
        return position_in(&previous, previous.piece_count() - 1, goal);
    }
    position_in(line, piece, goal)
}

/// Where Home or End takes each selection's head: the start or the end of the piece it is on.
fn piece_edge_heads(state: &EditorState, end: bool, shaping: &LineShaping, window: &mut Window) -> Vec<Position> {
    let mut heads = Vec::new();
    for selection in state.selections().all() {
        let line = shaping.line(state, selection.head.row, window);
        let piece = line.piece_of(line.display.offset_of(selection.head.column));
        let x = if end { Pixels::MAX } else { px(0.0) };
        heads.push(position_in(&line, piece, x));
    }
    heads
}

/// The buffer position nearest `x` from the left on a piece of a line.
fn position_in(line: &VisibleLine, piece: usize, x: Pixels) -> Position {
    let offset = line.offset_at(point(x, line.piece_top(piece)));
    Position { row: line.row, column: line.display.column_at(offset) }
}

fn hash_of(text: &str, spans: &[HighlightSpan]) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    for span in spans {
        (span.start_column, span.end_column, span.id.0).hash(&mut hasher);
    }
    hasher.finish()
}
