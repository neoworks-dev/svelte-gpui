//! Where display rows sit vertically, in pixels from the top of the first row. Everything that
//! turns a display row into pixels or pixels into a display row asks `RowLayout`, so rows can be
//! given heights of their own (soft wrapping, larger headings) by changing this file alone. Every
//! row is `ROW_HEIGHT` tall for now.

use svelte_gpui::gpui::{Pixels, px};

/// The height of a row of text in the editor's font.
pub const ROW_HEIGHT: Pixels = px(21.0);

/// The rows of a document's display: its lines and the rows of its blocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowLayout {
    rows: usize,
}

impl RowLayout {
    /// A layout of `rows` display rows.
    pub fn new(rows: usize) -> Self {
        RowLayout { rows }
    }

    /// How many display rows there are.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// The y of a row's top.
    pub fn top_of(&self, row: usize) -> Pixels {
        ROW_HEIGHT * row as f32
    }

    pub fn height_of(&self, _row: usize) -> Pixels {
        ROW_HEIGHT
    }

    /// The y of a row's bottom, which is the next row's top.
    pub fn bottom_of(&self, row: usize) -> Pixels {
        self.top_of(row) + self.height_of(row)
    }

    /// The row a y falls in. A y above the first row is in the first row.
    pub fn row_at(&self, y: Pixels) -> usize {
        (y / ROW_HEIGHT).floor().max(0.0) as usize
    }

    /// The first row whose top is at or below a y.
    pub fn first_row_from(&self, y: Pixels) -> usize {
        (y / ROW_HEIGHT).ceil().max(0.0) as usize
    }

    /// How many whole rows fit in a height, starting at `row`; at least one.
    pub fn rows_fitting(&self, _row: usize, height: Pixels) -> usize {
        (height / ROW_HEIGHT).floor().max(1.0) as usize
    }

    /// How many rows, starting at `row`, cover a height when the first may be scrolled part way out.
    pub fn rows_covering(&self, _row: usize, height: Pixels) -> usize {
        (height / ROW_HEIGHT).ceil() as usize + 1
    }

    /// The y the view scrolls to at most: the last row's top, so it may sit at the top of the view.
    pub fn furthest_scroll(&self) -> Pixels {
        self.top_of(self.rows.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_pixels_map_both_ways() {
        let layout = RowLayout::new(10);
        assert_eq!(layout.top_of(3), px(63.0));
        assert_eq!(layout.bottom_of(3), px(84.0));
        assert_eq!(layout.row_at(px(63.0)), 3);
        assert_eq!(layout.row_at(px(83.9)), 3);
        assert_eq!(layout.row_at(px(-5.0)), 0);
        assert_eq!(layout.first_row_from(px(64.0)), 4);
        assert_eq!(layout.first_row_from(px(63.0)), 3);
    }

    #[test]
    fn a_view_holds_whole_rows_and_draws_one_more() {
        let layout = RowLayout::new(100);
        assert_eq!(layout.rows_fitting(0, px(100.0)), 4);
        assert_eq!(layout.rows_fitting(0, px(5.0)), 1);
        assert_eq!(layout.rows_covering(0, px(100.0)), 6);
    }

    #[test]
    fn the_last_row_may_scroll_to_the_top() {
        assert_eq!(RowLayout::new(10).furthest_scroll(), px(189.0));
        assert_eq!(RowLayout::new(0).furthest_scroll(), px(0.0));
    }
}
