//! Paragraphs that hold elements: words and inline boxes laid out in lines. GPUI's text can't
//! hold a box, so each word is shaped on its own and placed, and the boxes between them.

use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::panic::Location;
use std::rc::Rc;

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Point, Rgba, ShapedLine, SharedString, Size, TextRun, Window, fill, point, px, size,
};

use super::select::box_selected;

/// An element inside a paragraph, at byte `at` of its text, with the size it was given.
pub(super) struct InlineBox {
    pub at: usize,
    pub element: AnyElement,
    pub size: Size<Pixels>,
    /// How far it sits above the baseline; negative lowers it.
    pub raise: f32,
    /// What copying a selection that holds the box gives; nothing when empty.
    pub copy: String,
}

/// What line breaking needs to know of a word or a box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Atom {
    /// The width with the spaces after it, which is how far the next atom starts.
    pub advance: f32,
    /// The width without those spaces, which is what must fit on the line.
    pub fit: f32,
    /// How far it reaches above and below the baseline.
    pub above: f32,
    pub below: f32,
    /// A line break follows it.
    pub breaks: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Line {
    pub top: f32,
    pub baseline: f32,
    pub width: f32,
}

/// Atoms broken into lines: each atom's line and x, and the paragraph's height.
#[derive(Debug, Default)]
pub(super) struct Lines {
    pub lines: Vec<Line>,
    pub placed: Vec<(usize, f32)>,
    pub height: f32,
}

/// Places atoms on lines no wider than `wrap`, left to right, breaking before an atom that
/// doesn't fit. Every line is at least as tall as the text strut `above` and `below`, as an
/// empty line of text would be.
pub(super) fn break_lines(atoms: &[Atom], wrap: Option<f32>, above: f32, below: f32) -> Lines {
    let mut placed = Vec::with_capacity(atoms.len());
    let mut extents: Vec<(f32, f32, f32)> = vec![(above, below, 0.0)];
    let mut x = 0.0;
    let mut starts_line = true;
    for atom in atoms {
        let overflows = wrap.is_some_and(|wrap| x + atom.fit > wrap);
        if overflows && !starts_line {
            extents.push((above, below, 0.0));
            x = 0.0;
        }
        let line = extents.len() - 1;
        let extent = &mut extents[line];
        extent.0 = extent.0.max(atom.above);
        extent.1 = extent.1.max(atom.below);
        extent.2 = extent.2.max(x + atom.fit);
        placed.push((line, x));
        x += atom.advance;
        starts_line = false;
        if atom.breaks {
            extents.push((above, below, 0.0));
            x = 0.0;
            starts_line = true;
        }
    }
    let mut lines = Vec::with_capacity(extents.len());
    let mut top = 0.0;
    for (line_above, line_below, width) in extents {
        lines.push(Line { top, baseline: top + line_above, width });
        top += line_above + line_below;
    }
    Lines { lines, placed, height: top }
}

/// A paragraph's text cut into words, each with the spaces after it, and the boxes between
/// them. A newline ends a word and breaks the line.
fn pieces(text: &str, boxes: &[usize]) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut next_box = 0;
    let mut in_spaces = false;
    for (index, character) in text.char_indices() {
        while next_box < boxes.len() && boxes[next_box] <= index {
            if start < index {
                out.push(Piece::Text { start, end: index, breaks: false });
            }
            out.push(Piece::Box(next_box));
            next_box += 1;
            start = index;
            in_spaces = false;
        }
        if character == '\n' {
            out.push(Piece::Text { start, end: index, breaks: true });
            start = index + 1;
            in_spaces = false;
            continue;
        }
        let space = character == ' ';
        if in_spaces && !space && start < index {
            out.push(Piece::Text { start, end: index, breaks: false });
            start = index;
        }
        in_spaces = space;
    }
    if start < text.len() {
        out.push(Piece::Text { start, end: text.len(), breaks: false });
    }
    while next_box < boxes.len() {
        out.push(Piece::Box(next_box));
        next_box += 1;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Piece {
    Text { start: usize, end: usize, breaks: bool },
    Box(usize),
}

/// The runs that cover `start..end` of the text, cut to it.
fn runs_within(runs: &[TextRun], start: usize, end: usize) -> Vec<TextRun> {
    let mut out = Vec::new();
    let mut offset = 0;
    for run in runs {
        let from = offset.max(start);
        let to = (offset + run.len).min(end);
        offset += run.len;
        if to > from {
            out.push(TextRun { len: to - from, ..run.clone() });
        }
    }
    out
}

enum Shaped {
    Text(Box<ShapedLine>),
    Box(usize),
}

/// Where an atom is in the paragraph's text: the bytes of a word, or the offset a box sits at.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Span {
    Text(Range<usize>),
    Box(usize),
}

struct Layout {
    shaped: Vec<Shaped>,
    atoms: Vec<Atom>,
    spans: Vec<Span>,
    broken: Lines,
    line_height: Pixels,
}

pub(super) struct InlineFlow {
    text: SharedString,
    runs: Vec<TextRun>,
    boxes: Vec<InlineBox>,
    layout: Rc<RefCell<Option<Layout>>>,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    /// The part of the text that is selected, drawn with this colour behind it.
    selection: Option<(Range<usize>, Rgba)>,
}

impl InlineFlow {
    pub(super) fn new(text: String, runs: Vec<TextRun>, boxes: Vec<InlineBox>) -> Self {
        InlineFlow {
            text: SharedString::from(text),
            runs,
            boxes,
            layout: Rc::default(),
            bounds: Rc::default(),
            selection: None,
        }
    }

    /// Draws `range` of the text selected, in `colour`.
    pub(super) fn selected(mut self, range: Range<usize>, colour: Rgba) -> Self {
        self.selection = Some((range, colour));
        self
    }

    /// Where this paragraph is laid out, for turning a pointer into a place in its text.
    pub(super) fn layout_handle(&self) -> FlowLayout {
        FlowLayout { layout: self.layout.clone(), bounds: self.bounds.clone(), len: self.text.len() }
    }
}

/// What a paragraph with boxes keeps of its layout, once it is drawn.
#[derive(Clone)]
pub struct FlowLayout {
    layout: Rc<RefCell<Option<Layout>>>,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    len: usize,
}

impl FlowLayout {
    /// The byte offset in the text a point in the window is at: the edge between characters
    /// closest to it, or with `nearest` off, the start of the character under it.
    pub fn index_for_position(&self, position: Point<Pixels>, nearest: bool) -> usize {
        let state = self.layout.borrow();
        let Some(layout) = state.as_ref() else {
            return 0;
        };
        let relative = position - self.bounds.get().origin;
        let at = (f32::from(relative.x), f32::from(relative.y));
        locate(&layout.broken, &layout.atoms, &layout.spans, at, self.len, |atom, x| match &layout.shaped[atom] {
            Shaped::Text(line) if nearest => line.closest_index_for_x(px(x)),
            Shaped::Text(line) => line.index_for_x(px(x)).unwrap_or(line.len),
            Shaped::Box(_) => 0,
        })
    }
}

/// The byte offset a point falls at, `(x, y)` from the paragraph's top left. It is in the word
/// the point is over, found by `closest` from its x within the word; past the end of a line it
/// is where the line's text ends, and below or above the paragraph, its end or start.
pub(super) fn locate(
    broken: &Lines,
    atoms: &[Atom],
    spans: &[Span],
    (x, y): (f32, f32),
    len: usize,
    closest: impl Fn(usize, f32) -> usize,
) -> usize {
    if y < 0.0 {
        return 0;
    }
    if y >= broken.height {
        return len;
    }
    let Some(line) = broken.lines.iter().rposition(|line| line.top <= y) else {
        return 0;
    };
    let mut end = None;
    for (atom, (span, (placed, at))) in spans.iter().zip(&broken.placed).enumerate() {
        if *placed != line {
            continue;
        }
        if x < at + atoms[atom].advance {
            return match span {
                Span::Text(range) => (range.start + closest(atom, x - at)).min(range.end),
                Span::Box(offset) => *offset,
            };
        }
        end = Some(match span {
            Span::Text(range) => range.end,
            Span::Box(offset) => *offset,
        });
    }
    end.unwrap_or(len)
}

/// Shapes every word once; breaking them into lines is redone for each width.
fn shape(
    text: &SharedString,
    runs: &[TextRun],
    boxes: &[(usize, Size<Pixels>, f32)],
    font_size: Pixels,
    line_height: Pixels,
    window: &mut Window,
) -> (Vec<Shaped>, Vec<Atom>, Vec<Span>) {
    let offsets: Vec<usize> = boxes.iter().map(|(at, _, _)| *at).collect();
    let mut shaped = Vec::new();
    let mut atoms = Vec::new();
    let mut spans = Vec::new();
    for piece in pieces(text, &offsets) {
        match piece {
            Piece::Text { start, end, breaks } => {
                spans.push(Span::Text(start..end));
                let word = &text[start..end];
                let line = window.text_system().shape_line(
                    SharedString::from(word.to_string()),
                    font_size,
                    &runs_within(runs, start, end),
                    None,
                );
                let above = (line_height - line.ascent - line.descent) / 2. + line.ascent;
                let fit = line.x_for_index(word.trim_end_matches(' ').len());
                atoms.push(Atom {
                    advance: f32::from(line.width),
                    fit: f32::from(fit),
                    above: f32::from(above),
                    below: f32::from(line_height - above),
                    breaks,
                });
                shaped.push(Shaped::Text(Box::new(line)));
            }
            Piece::Box(index) => {
                spans.push(Span::Box(boxes[index].0));
                let (_, box_size, raise) = boxes[index];
                let height = f32::from(box_size.height);
                atoms.push(Atom {
                    advance: f32::from(box_size.width),
                    fit: f32::from(box_size.width),
                    above: height + raise,
                    below: -raise,
                    breaks: false,
                });
                shaped.push(Shaped::Box(index));
            }
        }
    }
    (shaped, atoms, spans)
}

impl Element for InlineFlow {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, ()) {
        let text_style = window.text_style();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let line_height = text_style.line_height.to_pixels(font_size.into(), window.rem_size());
        let text = self.text.clone();
        let runs = self.runs.clone();
        let boxes: Vec<(usize, Size<Pixels>, f32)> =
            self.boxes.iter().map(|inline| (inline.at, inline.size, inline.raise)).collect();
        // A line of plain text, which every line is at least: GPUI centres the font's ascent and
        // descent in the line height.
        let font_id = window.text_system().resolve_font(&text_style.font());
        let ascent = window.text_system().ascent(font_id, font_size);
        let descent = window.text_system().descent(font_id, font_size);
        let above = f32::from((line_height - ascent - descent) / 2. + ascent);
        let below = f32::from(line_height) - above;
        let state = self.layout.clone();
        let layout_id = window.request_measured_layout(Default::default(), move |known, available, window, _cx| {
            let wrap = known.width.or(match available.width {
                AvailableSpace::Definite(width) => Some(width),
                _ => None,
            });
            let mut state = state.borrow_mut();
            let layout = state.get_or_insert_with(|| {
                let (shaped, atoms, spans) = shape(&text, &runs, &boxes, font_size, line_height, window);
                Layout { shaped, atoms, spans, broken: Lines::default(), line_height }
            });
            layout.broken = break_lines(&layout.atoms, wrap.map(f32::from), above, below);
            let width = layout.broken.lines.iter().map(|line| line.width).fold(0.0, f32::max).ceil();
            size(px(width), px(layout.broken.height))
        });
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.bounds.set(bounds);
        let state = self.layout.borrow();
        let Some(layout) = state.as_ref() else {
            return;
        };
        for (shaped, (line, x)) in layout.shaped.iter().zip(&layout.broken.placed) {
            let Shaped::Box(index) = shaped else {
                continue;
            };
            let inline = &mut self.boxes[*index];
            let baseline = layout.broken.lines[*line].baseline;
            let top = baseline - f32::from(inline.size.height) - inline.raise;
            let origin = bounds.origin + point(px(*x), px(top));
            inline.element.layout_as_root(inline.size.map(AvailableSpace::Definite), window, cx);
            inline.element.prepaint_at(origin, window, cx);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut (),
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let state = self.layout.borrow();
        let Some(layout) = state.as_ref() else {
            return;
        };
        let atoms = layout.shaped.iter().zip(&layout.broken.placed).zip(&layout.atoms).zip(&layout.spans);
        for (((shaped, (line, x)), atom), span) in atoms {
            // The selection fills the whole line, so a tall box does not leave notches in it.
            let line_top = layout.broken.lines[*line].top;
            let line_bottom = layout.broken.lines.get(line + 1).map_or(layout.broken.height, |next| next.top);
            let line_size = |width: Pixels| size(width, px(line_bottom - line_top));
            match shaped {
                Shaped::Text(text) => {
                    let top = layout.broken.lines[*line].baseline - atom.above;
                    let origin = bounds.origin + point(px(*x), px(top));
                    text.paint_background(origin, layout.line_height, window, cx).ok();
                    if let (Some((selected, colour)), Span::Text(range)) = (&self.selection, span) {
                        let (from, to) = (selected.start.max(range.start), selected.end.min(range.end));
                        if from < to {
                            let left = text.x_for_index(from - range.start);
                            let right = text.x_for_index(to - range.start);
                            let origin = bounds.origin + point(px(*x) + left, px(line_top));
                            window.paint_quad(fill(Bounds { origin, size: line_size(right - left) }, *colour));
                        }
                    }
                    text.paint(origin, layout.line_height, window, cx).ok();
                }
                Shaped::Box(index) => {
                    if let (Some((selected, colour)), Span::Box(at)) = (&self.selection, span) {
                        // A box the selection runs over is selected with the words around it.
                        if box_selected(selected, *at, self.text.len()) {
                            let origin = bounds.origin + point(px(*x), px(line_top));
                            window.paint_quad(fill(
                                Bounds { origin, size: line_size(self.boxes[*index].size.width) },
                                *colour,
                            ));
                        }
                    }
                    self.boxes[*index].element.paint(window, cx);
                }
            }
        }
    }
}

impl IntoElement for InlineFlow {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(width: f32) -> Atom {
        Atom { advance: width + 4.0, fit: width, above: 14.0, below: 6.0, breaks: false }
    }

    #[test]
    fn words_keep_their_spaces_and_boxes_split_them() {
        let text = "a bc  d\ne";
        let cut = pieces(text, &[4]);
        assert_eq!(
            cut,
            [
                Piece::Text { start: 0, end: 2, breaks: false },
                Piece::Text { start: 2, end: 4, breaks: false },
                Piece::Box(0),
                Piece::Text { start: 4, end: 6, breaks: false },
                Piece::Text { start: 6, end: 7, breaks: true },
                Piece::Text { start: 8, end: 9, breaks: false },
            ]
        );
        assert_eq!(pieces("ab", &[2]), [Piece::Text { start: 0, end: 2, breaks: false }, Piece::Box(0)]);
    }

    #[test]
    fn a_word_that_does_not_fit_starts_the_next_line() {
        let atoms = [word(40.0), word(40.0), word(40.0)];
        let broken = break_lines(&atoms, Some(90.0), 14.0, 6.0);
        assert_eq!(broken.placed, [(0, 0.0), (0, 44.0), (1, 0.0)]);
        assert_eq!(broken.lines[0].width, 84.0);
        assert_eq!(broken.height, 40.0);
    }

    #[test]
    fn a_word_wider_than_the_line_still_gets_one_of_its_own() {
        let atoms = [word(200.0), word(10.0)];
        let broken = break_lines(&atoms, Some(90.0), 14.0, 6.0);
        assert_eq!(broken.placed, [(0, 0.0), (1, 0.0)]);
    }

    #[test]
    fn a_tall_box_makes_its_line_taller_above_and_below_the_baseline() {
        // 30 high, lowered by 8: it reaches 22 above the baseline and 8 below.
        let formula = Atom { advance: 50.0, fit: 50.0, above: 22.0, below: 8.0, breaks: false };
        let broken = break_lines(&[word(20.0), formula, word(20.0)], Some(90.0), 14.0, 6.0);
        assert_eq!(broken.placed, [(0, 0.0), (0, 24.0), (1, 0.0)]);
        assert_eq!(broken.lines[0].baseline, 22.0);
        assert_eq!(broken.lines[1].top, 30.0);
        assert_eq!(broken.lines[1].baseline, 44.0);
        assert_eq!(broken.height, 50.0);
    }

    #[test]
    fn a_point_in_a_word_is_a_place_in_its_text_and_past_the_line_is_its_end() {
        // Three words of five bytes: two on the first line and one on the second.
        let atoms = [word(40.0), word(40.0), word(40.0)];
        let broken = break_lines(&atoms, Some(90.0), 14.0, 6.0);
        let spans = [Span::Text(0..5), Span::Text(5..10), Span::Text(10..15)];
        // A word's letters are 8 wide, which stands in for shaping.
        let closest = |_atom: usize, x: f32| ((x / 8.0).round().max(0.0)) as usize;
        let at = |x: f32, y: f32| locate(&broken, &atoms, &spans, (x, y), 15, closest);
        assert_eq!(at(10.0, 5.0), 1);
        assert_eq!(at(60.0, 5.0), 7);
        assert_eq!(at(3.0, 25.0), 10);
        // Past the end of a line it is the end of the line's text, and the spaces count as text.
        assert_eq!(at(200.0, 5.0), 10);
        // Above the paragraph is its start, and below it its end.
        assert_eq!(at(20.0, -4.0), 0);
        assert_eq!(at(20.0, 45.0), 15);
        // Before the first word, with a drag that went left of the paragraph.
        assert_eq!(at(-30.0, 25.0), 10);
    }

    #[test]
    fn a_point_over_a_box_is_where_the_box_sits() {
        let boxed = Atom { advance: 30.0, fit: 30.0, above: 14.0, below: 6.0, breaks: false };
        let atoms = [word(20.0), boxed, word(20.0)];
        let broken = break_lines(&atoms, None, 14.0, 6.0);
        let spans = [Span::Text(0..3), Span::Box(3), Span::Text(3..6)];
        let closest = |_atom: usize, x: f32| (x / 8.0) as usize;
        let at = |x: f32| locate(&broken, &atoms, &spans, (x, 5.0), 6, closest);
        assert_eq!(at(8.0), 1);
        assert_eq!(at(30.0), 3);
        assert_eq!(at(70.0), 5);
    }

    #[test]
    fn a_newline_breaks_the_line_and_an_empty_line_keeps_its_height() {
        let atoms = [Atom { breaks: true, ..word(10.0) }, Atom { breaks: true, ..word(0.0) }, word(10.0)];
        let broken = break_lines(&atoms, None, 14.0, 6.0);
        assert_eq!(broken.placed, [(0, 0.0), (1, 0.0), (2, 0.0)]);
        assert_eq!(broken.height, 60.0);
    }
}
