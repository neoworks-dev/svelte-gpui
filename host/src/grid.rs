//! The editor's text grid. Neovim's rows arrive from the app as runs of cells that
//! share a highlight, already coloured; this keeps them and paints them with GPUI
//! when a `<canvas>` that has a grid is drawn. Each glyph is forced onto the cell
//! width, so a font that is not quite monospaced still keeps its columns.

use gpui::{
    Bounds, FontWeight, Pixels, SharedString, StrikethroughStyle, TextRun, UnderlineStyle, Window,
    fill, font, point, px, rgb, size,
};
use serde_json::Value;

const BOLD: u64 = 1;
const ITALIC: u64 = 2;
const UNDERLINE: u64 = 4;
const UNDERCURL: u64 = 8;
const STRIKETHROUGH: u64 = 16;

#[derive(Clone, Debug)]
pub struct Run {
    pub col: u32,
    pub width: u32,
    pub text: String,
    pub foreground: u32,
    pub background: u32,
    pub special: u32,
    pub flags: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Cursor {
    pub row: i64,
    pub col: i64,
    /// 0 block, 1 vertical bar, 2 horizontal bar.
    pub shape: u8,
    pub percentage: f32,
    pub body: u32,
    pub text: u32,
    pub cell_text: String,
    pub flags: u64,
}

#[derive(Clone, Debug)]
pub struct GridModel {
    pub cols: u32,
    pub rows: u32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub baseline: f32,
    pub font_size: f32,
    pub family: String,
    pub background: u32,
    pub lines: Vec<Vec<Run>>,
    pub cursor: Option<Cursor>,
}

impl Default for GridModel {
    fn default() -> Self {
        Self {
            cols: 0,
            rows: 0,
            cell_width: 8.0,
            cell_height: 18.0,
            baseline: 14.0,
            font_size: 13.0,
            family: String::new(),
            background: 0x000000,
            lines: Vec::new(),
            cursor: None,
        }
    }
}

fn number(value: &Value, index: usize) -> f64 {
    value.get(index).and_then(Value::as_f64).unwrap_or(0.0)
}

fn text(value: &Value, index: usize) -> String {
    value
        .get(index)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

impl GridModel {
    /// Applies one instruction: the parts after the grid's id in an operation.
    pub fn apply(&mut self, parts: &[Value]) {
        let kind = parts.first().and_then(Value::as_str).unwrap_or("");
        let rest = Value::Array(parts[1..].to_vec());
        match kind {
            "configure" => {
                self.cols = number(&rest, 0) as u32;
                self.rows = number(&rest, 1) as u32;
                self.cell_width = number(&rest, 2) as f32;
                self.cell_height = number(&rest, 3) as f32;
                self.baseline = number(&rest, 4) as f32;
                self.font_size = number(&rest, 5) as f32;
                self.family = text(&rest, 6);
                self.lines.resize(self.rows as usize, Vec::new());
            }
            "clear" => {
                self.background = number(&rest, 0) as u32;
                self.lines = vec![Vec::new(); self.rows as usize];
            }
            "row" => {
                let row = number(&rest, 0) as usize;
                if row >= self.lines.len() {
                    self.lines.resize(row + 1, Vec::new());
                }
                let mut runs = Vec::new();
                if let Some(Value::Array(items)) = rest.get(1) {
                    for item in items {
                        runs.push(Run {
                            col: number(item, 0) as u32,
                            width: number(item, 1) as u32,
                            text: text(item, 2),
                            foreground: number(item, 3) as u32,
                            background: number(item, 4) as u32,
                            special: number(item, 5) as u32,
                            flags: number(item, 6) as u64,
                        });
                    }
                }
                self.lines[row] = runs;
            }
            "cursor" => {
                let row = number(&rest, 0) as i64;
                if row < 0 {
                    self.cursor = None;
                    return;
                }
                self.cursor = Some(Cursor {
                    row,
                    col: number(&rest, 1) as i64,
                    shape: number(&rest, 2) as u8,
                    percentage: number(&rest, 3) as f32,
                    body: number(&rest, 4) as u32,
                    text: number(&rest, 5) as u32,
                    cell_text: text(&rest, 6),
                    flags: number(&rest, 7) as u64,
                });
            }
            _ => {}
        }
    }

    /// Paints the whole grid into `bounds`.
    pub fn paint(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut gpui::App) {
        window.paint_quad(fill(bounds, rgb(self.background)));
        for (row, runs) in self.lines.iter().enumerate() {
            let top = bounds.origin.y + px(row as f32 * self.cell_height);
            for run in runs {
                if run.background == self.background {
                    continue;
                }
                let left = bounds.origin.x + px(run.col as f32 * self.cell_width);
                let cell = Bounds {
                    origin: point(left, top),
                    size: size(px(run.width as f32 * self.cell_width), px(self.cell_height)),
                };
                window.paint_quad(fill(cell, rgb(run.background)));
            }
            for run in runs {
                if run.text.trim().is_empty() {
                    continue;
                }
                let left = bounds.origin.x + px(run.col as f32 * self.cell_width);
                self.paint_text(&run.text, run.foreground, run.special, run.flags, left, top, window, cx);
            }
        }
        if let Some(cursor) = &self.cursor {
            self.paint_cursor(cursor, bounds, window, cx);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_text(
        &self,
        text: &str,
        foreground: u32,
        special: u32,
        flags: u64,
        left: Pixels,
        top: Pixels,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        // The face is the bundled monospace font whatever the page names: a name this
        // host has not loaded would fall back to a proportional font and lose the grid.
        let family = crate::render::MONO.to_string();
        let mut face = font(SharedString::from(family));
        if flags & BOLD != 0 {
            face.weight = FontWeight::BOLD;
        }
        if flags & ITALIC != 0 {
            face.style = gpui::FontStyle::Italic;
        }
        let underline = (flags & (UNDERLINE | UNDERCURL) != 0).then(|| UnderlineStyle {
            thickness: px(1.0),
            color: Some(rgb(special).into()),
            wavy: flags & UNDERCURL != 0,
        });
        let strikethrough = (flags & STRIKETHROUGH != 0).then(|| StrikethroughStyle {
            thickness: px(1.0),
            color: Some(rgb(foreground).into()),
        });
        let run = TextRun {
            len: text.len(),
            font: face,
            color: rgb(foreground).into(),
            background_color: None,
            underline,
            strikethrough,
        };
        let line = window.text_system().shape_line(
            SharedString::from(text.to_string()),
            px(self.font_size),
            &[run],
            Some(px(self.cell_width)),
        );
        let _ = line.paint(point(left, top), px(self.cell_height), window, cx);
    }

    fn paint_cursor(&self, cursor: &Cursor, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut gpui::App) {
        let left = bounds.origin.x + px(cursor.col as f32 * self.cell_width);
        let top = bounds.origin.y + px(cursor.row as f32 * self.cell_height);
        let share = cursor.percentage.clamp(0.05, 1.0);
        let cell = match cursor.shape {
            1 => Bounds {
                origin: point(left, top),
                size: size(px((self.cell_width * share).max(1.0)), px(self.cell_height)),
            },
            2 => {
                let height = (self.cell_height * share).max(1.0);
                Bounds {
                    origin: point(left, top + px(self.cell_height - height)),
                    size: size(px(self.cell_width), px(height)),
                }
            }
            _ => Bounds {
                origin: point(left, top),
                size: size(px(self.cell_width), px(self.cell_height)),
            },
        };
        window.paint_quad(fill(cell, rgb(cursor.body)));
        if cursor.shape == 0 && !cursor.cell_text.trim().is_empty() {
            self.paint_text(&cursor.cell_text, cursor.text, cursor.text, cursor.flags, left, top, window, cx);
        }
    }
}
