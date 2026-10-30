//! Where display rows sit vertically, in pixels from the top of the first row. Everything that
//! turns a display row into pixels or pixels into a display row asks `RowLayout`, so rows can be
//! given heights of their own by changing this file alone. Rows are `ROW_HEIGHT` tall unless the
//! layout is given their heights, as it is for soft wrapping and for themes with fonts of their
//! own (see `wrap.rs`).

use svelte_gpui::gpui::{Pixels, px};

/// The height of a row of text in the editor's font.
pub const ROW_HEIGHT: Pixels = px(21.0);
/// How much taller than its font's size a row of text is: 21 pixels for the editor's 14.
const LINE_SPACING: f32 = 1.5;

/// How tall a piece of a line drawn at `font_size` is.
pub fn piece_height(font_size: Pixels) -> Pixels {
    (font_size * LINE_SPACING).round()
}

/// The rows of a document's display: its lines and the rows of its blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct RowLayout {
    rows: usize,
    /// The top of each row and, last, the bottom of the last one; none while every row is
    /// `ROW_HEIGHT` tall. Rows past the last are `ROW_HEIGHT` tall either way.
    tops: Option<Vec<Pixels>>,
}

impl RowLayout {
    /// A layout of `rows` display rows, each `ROW_HEIGHT` tall.
    pub fn new(rows: usize) -> Self {
        RowLayout { rows, tops: None }
    }

    /// A layout of rows with the heights given, in order.
    pub fn with_heights(heights: impl IntoIterator<Item = Pixels>) -> Self {
        let mut tops = vec![px(0.0)];
        let mut bottom = px(0.0);
        for height in heights {
            bottom += height;
            tops.push(bottom);
        }
        RowLayout { rows: tops.len() - 1, tops: Some(tops) }
    }

    /// How many display rows there are.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// The y of a row's top.
    pub fn top_of(&self, row: usize) -> Pixels {
        let Some(tops) = &self.tops else {
            return ROW_HEIGHT * row as f32;
        };
        if row <= self.rows {
            return tops[row];
        }
        tops[self.rows] + ROW_HEIGHT * (row - self.rows) as f32
    }

    pub fn height_of(&self, row: usize) -> Pixels {
        self.top_of(row + 1) - self.top_of(row)
    }

    /// The y of a row's bottom, which is the next row's top.
    pub fn bottom_of(&self, row: usize) -> Pixels {
        self.top_of(row + 1)
    }

    /// The row a y falls in. A y above the first row is in the first row.
    pub fn row_at(&self, y: Pixels) -> usize {
        self.rows_with_top_at_most(y).saturating_sub(1)
    }

    /// The first row whose top is at or below a y.
    pub fn first_row_from(&self, y: Pixels) -> usize {
        self.rows_with_top_above(y)
    }

    /// How many whole rows fit in a height, starting at `row`; at least one.
    pub fn rows_fitting(&self, row: usize, height: Pixels) -> usize {
        let past_last = self.rows_with_top_at_most(self.top_of(row) + height);
        past_last.saturating_sub(1).saturating_sub(row).max(1)
    }

    /// How many rows, starting at `row`, cover a height when the first may be scrolled part way out.
    pub fn rows_covering(&self, row: usize, height: Pixels) -> usize {
        self.rows_with_top_above(self.bottom_of(row) + height) - row
    }

    /// The y the view scrolls to at most: the last row's top, so it may sit at the top of the view.
    pub fn furthest_scroll(&self) -> Pixels {
        self.top_of(self.rows.saturating_sub(1))
    }

    /// How many rows, counting on past the last, have their top at or above a y.
    fn rows_with_top_at_most(&self, y: Pixels) -> usize {
        if y < px(0.0) {
            return 0;
        }
        let Some(tops) = &self.tops else {
            return (y / ROW_HEIGHT).floor() as usize + 1;
        };
        let bottom = tops[self.rows];
        if y >= bottom {
            return self.rows + ((y - bottom) / ROW_HEIGHT).floor() as usize + 1;
        }
        tops[..self.rows].partition_point(|top| *top <= y)
    }

    /// How many rows, counting on past the last, have their top above a y.
    fn rows_with_top_above(&self, y: Pixels) -> usize {
        if y <= px(0.0) {
            return 0;
        }
        let Some(tops) = &self.tops else {
            return (y / ROW_HEIGHT).ceil() as usize;
        };
        let bottom = tops[self.rows];
        if y > bottom {
            return self.rows + ((y - bottom) / ROW_HEIGHT).ceil() as usize;
        }
        tops[..self.rows].partition_point(|top| *top < y)
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

    #[test]
    fn rows_of_one_height_given_one_by_one_map_like_the_default() {
        let given = RowLayout::with_heights(vec![ROW_HEIGHT; 100]);
        let default = RowLayout::new(100);
        for y in [-5.0, 0.0, 20.9, 21.0, 63.0, 83.9, 100.0, 2099.0, 2100.0, 2200.0] {
            assert_eq!(given.row_at(px(y)), default.row_at(px(y)), "row at {y}");
            assert_eq!(given.first_row_from(px(y)), default.first_row_from(px(y)), "first row from {y}");
        }
        for row in [0, 5, 98, 99, 100, 120] {
            assert_eq!(given.top_of(row), default.top_of(row));
            assert_eq!(given.rows_fitting(row, px(100.0)), default.rows_fitting(row, px(100.0)));
            assert_eq!(given.rows_covering(row, px(100.0)), default.rows_covering(row, px(100.0)));
        }
        assert_eq!(given.furthest_scroll(), default.furthest_scroll());
    }

    #[test]
    fn a_piece_is_half_again_as_tall_as_its_font() {
        assert_eq!(piece_height(px(14.0)), ROW_HEIGHT);
        assert_eq!(piece_height(px(24.0)), px(36.0));
        assert_eq!(piece_height(px(15.0)), px(23.0));
    }

    #[test]
    fn tall_rows_take_the_space_of_their_height() {
        // A line wrapped into three rows' worth between two plain ones.
        let layout = RowLayout::with_heights([ROW_HEIGHT, ROW_HEIGHT * 3.0, ROW_HEIGHT]);
        assert_eq!(layout.top_of(1), px(21.0));
        assert_eq!(layout.height_of(1), px(63.0));
        assert_eq!(layout.top_of(2), px(84.0));
        assert_eq!(layout.row_at(px(83.0)), 1);
        assert_eq!(layout.row_at(px(84.0)), 2);
        assert_eq!(layout.first_row_from(px(22.0)), 2);
        assert_eq!(layout.rows_fitting(0, px(84.0)), 2);
        assert_eq!(layout.rows_fitting(1, px(30.0)), 1);
        assert_eq!(layout.furthest_scroll(), px(84.0));
        // Past the last row, rows are the default height.
        assert_eq!(layout.top_of(4), px(126.0));
        assert_eq!(layout.row_at(px(130.0)), 4);
    }
}
