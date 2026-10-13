//! Draws the host's tree with GPUI. Each element becomes a `div` with the style
//! the stylesheet gives it; text nodes become the text of their parent. The tree
//! is walked afresh on every frame, which is cheap next to the layout and paint
//! that follow, and means nothing here has to track what changed.

use gpui::{
    AnyElement, Context, Div, FontWeight, IntoElement, Length, Rgba, SharedString, auto, div,
    prelude::*, px, relative,
};

use crate::Workbench;
use crate::style::{Align, Computed, Dimension, Display, HoverStyle, Justify, Resolver, Style};
use crate::tree::{Kind, NodeId, ROOT, Tree};

pub const SANS: &str = "Geist";
pub const MONO: &str = "JetBrainsMono Nerd Font Mono";

pub struct Painter<'a> {
    pub tree: &'a Tree,
    pub resolver: &'a Resolver,
}

/// How an element lays out the text and elements inside it.
#[derive(Clone, Copy, PartialEq)]
enum Flow {
    /// Children are flex items: whitespace between them is dropped.
    Flex,
    /// Children sit in a line as they would in running text: spaces stay.
    Inline,
    /// Block children stack; text and inline children share anonymous lines.
    Block,
}

enum Piece {
    Text(String),
    Node(NodeId),
}

/// What a child inherits from the elements above it.
#[derive(Clone, Copy)]
struct Inherited {
    uppercase: bool,
}

impl Painter<'_> {
    /// The whole document: the root's children, filling the window.
    pub fn document(&self, cx: &mut Context<Workbench>) -> AnyElement {
        let root = self.resolver.root_style();
        let mut element = div().size_full().flex().flex_col();
        element = element.font_family(SANS).text_size(px(14.0)).line_height(relative(1.5));
        if let Some(colour) = root.color {
            element = element.text_color(colour);
        }
        if let Some(size) = root.font_size {
            element = element.text_size(px(size));
        }
        let inherited = Inherited { uppercase: false };
        for child in self.children(ROOT, Flow::Flex, inherited, cx) {
            element = element.child(child);
        }
        element.into_any_element()
    }

    fn pieces(&self, parent: NodeId) -> Vec<Piece> {
        let mut pieces: Vec<Piece> = Vec::new();
        let Some(node) = self.tree.nodes.get(&parent) else {
            return pieces;
        };
        for child in &node.children {
            let Some(child_node) = self.tree.nodes.get(child) else {
                continue;
            };
            match &child_node.kind {
                Kind::Text { data } => match pieces.last_mut() {
                    Some(Piece::Text(previous)) => previous.push_str(data),
                    _ => pieces.push(Piece::Text(data.clone())),
                },
                Kind::Element { .. } => pieces.push(Piece::Node(*child)),
            }
        }
        pieces
    }

    fn children(
        &self,
        parent: NodeId,
        flow: Flow,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
    ) -> Vec<AnyElement> {
        let pieces = self.expand_contents(self.pieces(parent));
        let mut elements = Vec::new();
        match flow {
            Flow::Flex => {
                for piece in &pieces {
                    if let Some(element) = self.piece(piece, true, inherited, cx) {
                        elements.push(element);
                    }
                }
            }
            Flow::Inline => {
                let last = pieces.len().saturating_sub(1);
                for (index, piece) in pieces.iter().enumerate() {
                    let text = match piece {
                        Piece::Text(data) => Some(collapse(data, index == 0, index == last)),
                        Piece::Node(_) => None,
                    };
                    if let Some(text) = text {
                        if !text.is_empty() {
                            elements.push(self.text_element(&text, inherited));
                        }
                        continue;
                    }
                    if let Some(element) = self.piece(piece, false, inherited, cx) {
                        elements.push(element);
                    }
                }
            }
            Flow::Block => self.block_children(&pieces, inherited, cx, &mut elements),
        }
        elements
    }

    /// Splices the children of `display: contents` elements into their parent's list.
    fn expand_contents(&self, pieces: Vec<Piece>) -> Vec<Piece> {
        let mut expanded = Vec::new();
        for piece in pieces {
            let Piece::Node(id) = piece else {
                expanded.push(piece);
                continue;
            };
            let computed = self.computed(id);
            if computed.style.display != Display::Contents {
                expanded.push(Piece::Node(id));
                continue;
            }
            expanded.extend(self.expand_contents(self.pieces(id)));
        }
        expanded
    }

    /// Block flow: elements stack, and runs of text and inline elements share a line.
    fn block_children(
        &self,
        pieces: &[Piece],
        inherited: Inherited,
        cx: &mut Context<Workbench>,
        out: &mut Vec<AnyElement>,
    ) {
        let mut run: Vec<&Piece> = Vec::new();
        for piece in pieces {
            if self.is_inline_level(piece) {
                run.push(piece);
                continue;
            }
            self.flush_run(&mut run, inherited, cx, out, false);
            if let Some(element) = self.piece(piece, false, inherited, cx) {
                out.push(element);
            }
        }
        self.flush_run(&mut run, inherited, cx, out, true);
    }

    fn is_inline_level(&self, piece: &Piece) -> bool {
        match piece {
            Piece::Text(_) => true,
            Piece::Node(id) => self.computed(*id).style.display == Display::Inline,
        }
    }

    fn flush_run(
        &self,
        run: &mut Vec<&Piece>,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
        out: &mut Vec<AnyElement>,
        at_end: bool,
    ) {
        if run.is_empty() {
            return;
        }
        let only_whitespace = run
            .iter()
            .all(|piece| matches!(piece, Piece::Text(data) if data.trim().is_empty()));
        if only_whitespace {
            run.clear();
            return;
        }
        let first_in_block = out.is_empty();
        let mut items: Vec<AnyElement> = Vec::new();
        let last = run.len() - 1;
        for (index, piece) in run.iter().enumerate() {
            match piece {
                Piece::Text(data) => {
                    let text = collapse(data, first_in_block && index == 0, at_end && index == last);
                    if !text.is_empty() {
                        items.push(self.text_element(&text, inherited));
                    }
                }
                Piece::Node(_) => {
                    if let Some(element) = self.piece(piece, false, inherited, cx) {
                        items.push(element);
                    }
                }
            }
        }
        run.clear();
        match items.len() {
            0 => {}
            1 => out.push(items.pop().expect("one item")),
            _ => {
                let mut line = div().flex().flex_row().flex_wrap().items_baseline();
                for item in items {
                    line = line.child(item);
                }
                out.push(line.into_any_element());
            }
        }
    }

    fn piece(
        &self,
        piece: &Piece,
        trim: bool,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
    ) -> Option<AnyElement> {
        match piece {
            Piece::Text(data) => {
                if !trim {
                    return Some(self.text_element(data, inherited));
                }
                let text = collapse(data, true, true);
                if text.is_empty() {
                    return None;
                }
                Some(self.text_element(&text, inherited))
            }
            Piece::Node(id) => self.element(*id, inherited, cx),
        }
    }

    fn text_element(&self, text: &str, inherited: Inherited) -> AnyElement {
        let shown = if inherited.uppercase {
            text.to_uppercase()
        } else {
            text.to_string()
        };
        div().child(SharedString::from(shown)).into_any_element()
    }

    fn computed(&self, id: NodeId) -> Computed {
        let Some(node) = self.tree.nodes.get(&id) else {
            return self.resolver.compute("div", "", &[]);
        };
        let tag = match &node.kind {
            Kind::Element { tag } => tag.as_str(),
            Kind::Text { .. } => "span",
        };
        let class = match node.attributes.get("class") {
            Some(class) => class.as_str(),
            None => "",
        };
        self.resolver.compute(tag, class, &node.style)
    }

    /// One element, with its children, or nothing when it is not displayed.
    fn element(
        &self,
        id: NodeId,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
    ) -> Option<AnyElement> {
        let node = self.tree.nodes.get(&id)?;
        let computed = self.computed(id);
        let style = &computed.style;
        if style.display == Display::None {
            return None;
        }
        let inherited = Inherited {
            uppercase: inherited.uppercase || style.uppercase,
        };

        let mut element = apply_style(div(), style);
        let flow = match style.display {
            Display::Flex => Flow::Flex,
            Display::Inline => Flow::Inline,
            _ => Flow::Block,
        };
        for child in self.children(id, flow, inherited, cx) {
            element = element.child(child);
        }
        element = self.border_strips(element, style);

        let class = match node.attributes.get("class") {
            Some(class) => class.as_str(),
            None => "",
        };
        Some(self.interactive(element, id, style, &computed, class, cx))
    }

    /// GPUI draws one border colour per element; sides that differ get a strip of their own.
    fn border_strips(&self, mut element: Div, style: &Style) -> Div {
        let Some(main) = main_border_colour(style) else {
            return element;
        };
        for side in 0..4 {
            let width = style.border_width[side];
            let Some(colour) = style.border_color[side] else {
                continue;
            };
            if width <= 0.0 || colour == main {
                continue;
            }
            let mut strip = div().absolute().bg(colour);
            strip = match side {
                0 => strip.left_0().right_0().top(px(-width)).h(px(width)),
                1 => strip.top_0().bottom_0().right(px(-width)).w(px(width)),
                2 => strip.left_0().right_0().bottom(px(-width)).h(px(width)),
                _ => strip.top_0().bottom_0().left(px(-width)).w(px(width)),
            };
            element = element.child(strip);
        }
        element
    }

    /// Gives an element an id, so it can take clicks, hover and scrolling.
    fn interactive(
        &self,
        element: Div,
        id: NodeId,
        style: &Style,
        computed: &Computed,
        class: &str,
        cx: &mut Context<Workbench>,
    ) -> AnyElement {
        let mut element = element.id(id as usize);
        if class.split_whitespace().any(|name| name == "group") {
            element = element.group("group");
        }
        if style.scroll_y {
            element = element.overflow_y_scroll();
        }
        if style.pointer {
            element = element.cursor_pointer();
        }
        if !computed.hover.is_empty() {
            let hover = computed.hover.clone();
            element = element.hover(move |refinement| apply_hover(refinement, &hover));
        }
        if !computed.group_hover.is_empty() {
            let hover = computed.group_hover.clone();
            element = element.group_hover("group", move |refinement| apply_hover(refinement, &hover));
        }
        element
            .on_click(cx.listener(move |workbench, _event, _window, cx| {
                workbench.click(id, cx);
                cx.stop_propagation();
            }))
            .into_any_element()
    }
}

fn apply_hover(mut refinement: gpui::StyleRefinement, hover: &HoverStyle) -> gpui::StyleRefinement {
    if let Some(background) = hover.background {
        refinement = refinement.bg(background);
    }
    if let Some(colour) = hover.color {
        refinement = refinement.text_color(colour);
    }
    if let Some(opacity) = hover.opacity {
        refinement = refinement.opacity(opacity);
    }
    if let Some(colour) = hover.border_color {
        refinement = refinement.border_color(colour);
    }
    refinement
}

/// Collapses runs of whitespace to one space, as CSS does for ordinary text.
fn collapse(text: &str, trim_start: bool, trim_end: bool) -> String {
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
    let mut trimmed = out.as_str();
    if trim_start {
        trimmed = trimmed.trim_start();
    }
    if trim_end {
        trimmed = trimmed.trim_end();
    }
    trimmed.to_string()
}

fn main_border_colour(style: &Style) -> Option<Rgba> {
    for side in 0..4 {
        if style.border_width[side] > 0.0 {
            if let Some(colour) = style.border_color[side] {
                return Some(colour);
            }
        }
    }
    None
}

fn length(dimension: Dimension) -> Length {
    match dimension {
        Dimension::Px(pixels) => px(pixels).into(),
        Dimension::Percent(percent) => relative(percent / 100.0).into(),
        Dimension::Auto => auto(),
    }
}

/// Puts a computed style onto a div, property by property.
fn apply_style(mut element: Div, style: &Style) -> Div {
    element = match style.display {
        Display::Block => element.block(),
        _ => element.flex(),
    };
    element = apply_flex(element, style);
    element = apply_box(element, style);
    element = apply_border(element, style);
    apply_text(element, style)
}

fn apply_flex(mut element: Div, style: &Style) -> Div {
    element = if style.column { element.flex_col() } else { element.flex_row() };
    if style.wrap {
        element = element.flex_wrap();
    }
    if let Some(grow) = style.grow {
        element.style().flex_grow = Some(grow);
    }
    if let Some(shrink) = style.shrink {
        element.style().flex_shrink = Some(shrink);
    }
    if let Some(basis) = style.basis {
        element.style().flex_basis = Some(length(basis));
    }
    element = match style.align_items {
        Some(Align::Start) => element.items_start(),
        Some(Align::End) => element.items_end(),
        Some(Align::Center) => element.items_center(),
        Some(Align::Baseline) => element.items_baseline(),
        _ => element,
    };
    element = match style.justify {
        Some(Justify::Start) => element.justify_start(),
        Some(Justify::End) => element.justify_end(),
        Some(Justify::Center) => element.justify_center(),
        Some(Justify::Between) => element.justify_between(),
        Some(Justify::Around) => element.justify_around(),
        None => element,
    };
    if let Some(gap) = style.row_gap {
        element = element.gap_y(px(gap));
    }
    if let Some(gap) = style.column_gap {
        element = element.gap_x(px(gap));
    }
    element
}

fn apply_box(mut element: Div, style: &Style) -> Div {
    let [top, right, bottom, left] = style.padding;
    if let Some(value) = top {
        element = element.pt(px(value));
    }
    if let Some(value) = right {
        element = element.pr(px(value));
    }
    if let Some(value) = bottom {
        element = element.pb(px(value));
    }
    if let Some(value) = left {
        element = element.pl(px(value));
    }
    let [top, right, bottom, left] = style.margin;
    if let Some(value) = top {
        element = element.mt(length(value));
    }
    if let Some(value) = right {
        element = element.mr(length(value));
    }
    if let Some(value) = bottom {
        element = element.mb(length(value));
    }
    if let Some(value) = left {
        element = element.ml(length(value));
    }
    if let Some(value) = style.width {
        element = element.w(length(value));
    }
    if let Some(value) = style.height {
        element = element.h(length(value));
    }
    if let Some(value) = style.min_width {
        element = element.min_w(length(value));
    }
    if let Some(value) = style.min_height {
        element = element.min_h(length(value));
    }
    if let Some(value) = style.max_width {
        element = element.max_w(length(value));
    }
    if let Some(value) = style.max_height {
        element = element.max_h(length(value));
    }
    if style.absolute {
        element = element.absolute();
        let [top, right, bottom, left] = style.inset;
        if let Some(value) = top {
            element = element.top(length(value));
        }
        if let Some(value) = right {
            element = element.right(length(value));
        }
        if let Some(value) = bottom {
            element = element.bottom(length(value));
        }
        if let Some(value) = left {
            element = element.left(length(value));
        }
    }
    element = match (style.clip_x, style.clip_y) {
        (true, true) => element.overflow_hidden(),
        (true, false) => element.overflow_x_hidden(),
        (false, true) => element.overflow_y_hidden(),
        (false, false) => element,
    };
    if let Some(colour) = style.background {
        element = element.bg(colour);
    }
    if let Some(opacity) = style.opacity {
        element = element.opacity(opacity);
    }
    element
}

fn apply_border(mut element: Div, style: &Style) -> Div {
    let [top, right, bottom, left] = style.border_width;
    if top > 0.0 {
        element.style().border_widths.top = Some(px(top).into());
    }
    if right > 0.0 {
        element.style().border_widths.right = Some(px(right).into());
    }
    if bottom > 0.0 {
        element.style().border_widths.bottom = Some(px(bottom).into());
    }
    if left > 0.0 {
        element.style().border_widths.left = Some(px(left).into());
    }
    if let Some(colour) = main_border_colour(style) {
        element = element.border_color(colour);
    }
    let [top_left, top_right, bottom_right, bottom_left] = style.radius;
    if top_left > 0.0 {
        element.style().corner_radii.top_left = Some(px(top_left).into());
    }
    if top_right > 0.0 {
        element.style().corner_radii.top_right = Some(px(top_right).into());
    }
    if bottom_right > 0.0 {
        element.style().corner_radii.bottom_right = Some(px(bottom_right).into());
    }
    if bottom_left > 0.0 {
        element.style().corner_radii.bottom_left = Some(px(bottom_left).into());
    }
    element
}

fn apply_text(mut element: Div, style: &Style) -> Div {
    if let Some(colour) = style.color {
        element = element.text_color(colour);
    }
    if let Some(size) = style.font_size {
        element = element.text_size(px(size));
    }
    if let Some(weight) = style.font_weight {
        element = element.font_weight(FontWeight(weight));
    }
    match style.mono {
        Some(true) => element = element.font_family(MONO),
        Some(false) => element = element.font_family(SANS),
        None => {}
    }
    if let Some(height) = style.line_height {
        element = if height < 0.0 {
            element.line_height(px(-height))
        } else {
            element.line_height(relative(height))
        };
    }
    if style.italic {
        element = element.italic();
    }
    if style.underline {
        element = element.underline();
    }
    if style.center_text {
        element = element.text_center();
    }
    if style.end_text {
        element = element.text_right();
    }
    if style.nowrap {
        element = element.whitespace_nowrap();
    }
    if style.ellipsis {
        element = element.text_ellipsis();
    }
    element
}
