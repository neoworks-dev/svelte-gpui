//! Draws the host's tree with GPUI. Each element becomes a `div` with the style
//! the stylesheet gives it; text nodes become the text of their parent. The tree
//! is walked afresh on every frame, which is cheap next to the layout and paint
//! that follow, and means nothing here has to track what changed.
//!
//! Input goes the other way. Which events an element listens to decides which
//! GPUI handlers it gets, and each handler reports to the app by node id.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, Bounds, ClickEvent, Context, Div, FocusHandle, FontWeight, HighlightStyle, Hsla,
    IntoElement, Length, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ObjectFit,
    Rgba, ScrollDelta, ScrollHandle, ScrollWheelEvent, SharedString, StyledImage, StyledText,
    Corner, anchored, auto, canvas, deferred, div, fill, img, point, prelude::*, px, relative,
    size, svg,
};

use crate::Workbench;
use crate::forms::{Control, Field, control_of};
use crate::select::choices;
use crate::assets::register_svg;
use crate::images::{ImageState, Images};
use crate::style::{
    Align, Computed, Dimension, Display, Fit, HoverStyle, Justify, Resolver, Style,
};
use crate::tree::{Kind, Node, NodeId, ROOT, Tree};

pub const SANS: &str = "Geist";
pub const MONO: &str = "JetBrainsMono Nerd Font Mono";

/// What lives between frames: the handles GPUI needs to keep focus and scroll
/// positions, what the last layout measured, and the images being loaded.
pub struct Interaction {
    pub root_focus: FocusHandle,
    pub focus: RefCell<HashMap<NodeId, FocusHandle>>,
    pub scroll: RefCell<HashMap<NodeId, ScrollHandle>>,
    pub sizes: Rc<RefCell<HashMap<NodeId, (f32, f32)>>>,
    pub fields: Rc<RefCell<HashMap<NodeId, Field>>>,
    /// Where each measured element came out in the window, for anchoring menus to it.
    pub bounds: Rc<RefCell<HashMap<NodeId, Bounds<gpui::Pixels>>>>,
    /// The select whose list is showing, if any.
    pub open_select: RefCell<Option<NodeId>>,
    /// Elements the pointer is over, for hover styles that ease instead of snapping.
    pub hovered: RefCell<HashSet<NodeId>>,
    /// Transitions in flight, by element.
    pub animations: RefCell<HashMap<NodeId, Animation>>,
    /// Set while painting when something is still easing, so another frame is asked for.
    pub animating: Cell<bool>,
    pub images: Images,
}

impl Interaction {
    fn focus_handle(&self, id: NodeId, cx: &mut Context<Workbench>) -> FocusHandle {
        self.focus
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| cx.focus_handle())
            .clone()
    }

    fn scroll_handle(&self, id: NodeId) -> ScrollHandle {
        self.scroll
            .borrow_mut()
            .entry(id)
            .or_insert_with(ScrollHandle::new)
            .clone()
    }
}

/// The properties a CSS transition eases between.
#[derive(Clone, Copy, PartialEq)]
pub struct Eased {
    background: Option<Rgba>,
    color: Option<Rgba>,
    border: Option<Rgba>,
    opacity: f32,
}

/// A transition in flight: from where the element was to where its style now says.
pub struct Animation {
    from: Eased,
    to: Eased,
    /// When the easing begins, delay included; None once it has finished.
    start: Option<Instant>,
}

fn mix(from: Rgba, to: Rgba, amount: f32) -> Rgba {
    Rgba {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

/// A colour easing towards another; a missing one counts as transparent.
fn mix_option(from: Option<Rgba>, to: Option<Rgba>, amount: f32) -> Option<Rgba> {
    match (from, to) {
        (Some(from), Some(to)) => Some(mix(from, to, amount)),
        (None, Some(to)) => Some(mix(Rgba { a: 0.0, ..to }, to, amount)),
        (Some(from), None) => Some(mix(from, Rgba { a: 0.0, ..from }, amount)),
        (None, None) => None,
    }
}

impl Animation {
    /// Where the easing has got to at `now`, and how far along it is.
    fn value_at(&self, now: Instant, transition: &crate::style::Transition) -> (Eased, f32) {
        let Some(start) = self.start else {
            return (self.to, 1.0);
        };
        if now < start {
            return (self.from, 0.0);
        }
        let elapsed = now.duration_since(start).as_secs_f32() * 1000.0;
        let progress = (elapsed / transition.duration_ms.max(1.0)).clamp(0.0, 1.0);
        if progress >= 1.0 {
            return (self.to, 1.0);
        }
        let eased = transition.easing.at(progress);
        let mut value = self.to;
        if transition.colours {
            value.background = mix_option(self.from.background, self.to.background, eased);
            value.color = mix_option(self.from.color, self.to.color, eased);
            value.border = mix_option(self.from.border, self.to.border, eased);
        }
        if transition.opacity {
            value.opacity = self.from.opacity + (self.to.opacity - self.from.opacity) * eased;
        }
        (value, progress)
    }
}

pub struct Painter<'a> {
    pub tree: &'a Tree,
    pub resolver: &'a Resolver,
    pub interaction: &'a Interaction,
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
    font_size: f32,
    /// The text colour in force, which an `svg` is tinted with.
    colour: Rgba,
    weight: f32,
    italic: bool,
    mono: bool,
    underline: bool,
}

const DEFAULT_FONT_SIZE: f32 = 14.0;

impl Painter<'_> {
    /// The whole document: the root's children, filling the window.
    pub fn document(&self, cx: &mut Context<Workbench>) -> AnyElement {
        let root = self.resolver.root_style();
        let mut element = div().size_full().flex().flex_col();
        element = element
            .font_family(SANS)
            .text_size(px(DEFAULT_FONT_SIZE))
            .line_height(relative(1.5))
            .track_focus(&self.interaction.root_focus)
            .on_key_down(cx.listener(|workbench, event, window, cx| {
                // A key the host or the app used must not also be typed by the input method.
                if workbench.key(event, true, window, cx) {
                    cx.stop_propagation();
                }
            }))
            .on_key_up(cx.listener(|workbench, event, window, cx| {
                workbench.key_up(event, window, cx);
            }));
        if let Some(colour) = root.color {
            element = element.text_color(colour);
        }
        let mut inherited = Inherited {
            uppercase: false,
            font_size: DEFAULT_FONT_SIZE,
            colour: root.color.unwrap_or(gpui::rgb(0xfafafa)),
            weight: 400.0,
            italic: false,
            mono: false,
            underline: false,
        };
        if let Some(size) = root.font_size {
            element = element.text_size(px(size));
            inherited.font_size = size;
        }
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
                let all: Vec<&Piece> = pieces.iter().collect();
                if let Some(flowed) = self.text_flow(&all, true, true, inherited) {
                    elements.push(flowed);
                    return elements;
                }
                let last = pieces.len().saturating_sub(1);
                for (index, piece) in pieces.iter().enumerate() {
                    if let Piece::Text(data) = piece {
                        let text = collapse(data, index == 0, index == last);
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
        if let Some(flowed) = self.text_flow(run, first_in_block, at_end, inherited) {
            run.clear();
            out.push(flowed);
            return;
        }
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

    /// A run of text and inline text elements drawn as one wrapping paragraph, each element's
    /// weight, colour and font applied to its stretch of the text. None when something in
    /// the run is not text, such as an icon or a control, and it needs a box of its own.
    fn text_flow(
        &self,
        run: &[&Piece],
        trim_start: bool,
        trim_end: bool,
        inherited: Inherited,
    ) -> Option<AnyElement> {
        let mut text = String::new();
        let mut runs: Vec<gpui::TextRun> = Vec::new();
        let last = run.len().saturating_sub(1);
        for (index, piece) in run.iter().enumerate() {
            match piece {
                Piece::Text(data) => {
                    let collapsed = collapse(data, trim_start && index == 0, trim_end && index == last);
                    self.push_run(&mut text, &mut runs, &collapsed, inherited);
                }
                Piece::Node(id) => self.flow_element(*id, inherited, &mut text, &mut runs)?,
            }
        }
        if text.trim().is_empty() {
            return None;
        }
        let styled = StyledText::new(SharedString::from(text)).with_runs(runs);
        Some(div().min_w_0().child(styled).into_any_element())
    }

    fn push_run(&self, text: &mut String, runs: &mut Vec<gpui::TextRun>, piece: &str, inherited: Inherited) {
        self.push_styled(text, runs, piece, inherited, None);
    }

    fn push_styled(
        &self,
        text: &mut String,
        runs: &mut Vec<gpui::TextRun>,
        piece: &str,
        inherited: Inherited,
        background: Option<Rgba>,
    ) {
        if piece.is_empty() {
            return;
        }
        let mut font = gpui::font(if inherited.mono { MONO } else { SANS });
        font.weight = FontWeight(inherited.weight);
        if inherited.italic {
            font.style = gpui::FontStyle::Italic;
        }
        let underline = inherited.underline.then(|| gpui::UnderlineStyle {
            thickness: px(1.0),
            color: Some(inherited.colour.into()),
            wavy: false,
        });
        let text_piece = if inherited.uppercase { piece.to_uppercase() } else { piece.to_string() };
        text.push_str(&text_piece);
        runs.push(gpui::TextRun {
            len: text_piece.len(),
            font,
            color: inherited.colour.into(),
            background_color: background.map(Hsla::from),
            underline,
            strikethrough: None,
        });
    }

    /// Adds an inline element's text to a paragraph; false if it holds anything but text.
    fn flow_element(
        &self,
        id: NodeId,
        inherited: Inherited,
        text: &mut String,
        runs: &mut Vec<gpui::TextRun>,
    ) -> Option<()> {
        let node = self.tree.nodes.get(&id)?;
        let Kind::Element { tag } = &node.kind else {
            return None;
        };
        if tag == "br" {
            self.push_run(text, runs, "\n", inherited);
            return Some(());
        }
        if matches!(tag.as_str(), "img" | "svg" | "input" | "select" | "textarea" | "button" | "label") {
            return None;
        }
        let computed = self.computed(id);
        let style = &computed.style;
        if style.display == Display::None {
            return Some(());
        }
        // Padding, borders and rounding of an inline element are lost in running text;
        // what matters is that the paragraph wraps as one.
        if style.display != Display::Inline || !computed.hover.is_empty() || style.pointer {
            return None;
        }
        let here = Inherited {
            uppercase: inherited.uppercase || style.uppercase,
            font_size: inherited.font_size,
            colour: style.color.unwrap_or(inherited.colour),
            weight: style.font_weight.unwrap_or(inherited.weight),
            italic: style.italic || inherited.italic,
            mono: style.mono.unwrap_or(inherited.mono),
            underline: style.underline || inherited.underline,
        };
        for piece in self.expand_contents(self.pieces(id)) {
            match piece {
                Piece::Text(data) => {
                    let collapsed = collapse(&data, false, false);
                    self.push_styled(text, runs, &collapsed, here, style.background);
                }
                Piece::Node(child) => self.flow_element(child, here, text, runs)?,
            }
        }
        Some(())
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
        // Text may shrink below its full length and wrap, as it does in a flex row.
        let mut text = div().child(SharedString::from(shown));
        if std::env::var("SG_NO_TEXT_DIV_MINW").is_err() {
            text = text.min_w_0();
        }
        text.into_any_element()
    }

    fn computed(&self, id: NodeId) -> Computed {
        self.resolver.compute(self.tree, id)
    }

    /// One element, with its children, or nothing when it is not displayed.
    fn element(
        &self,
        id: NodeId,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
    ) -> Option<AnyElement> {
        let node = self.tree.nodes.get(&id)?;
        let mut computed = self.computed(id);
        if computed.style.display == Display::None {
            return None;
        }
        let eased = computed.style.transition.duration_ms > 0.0
            && (computed.style.transition.colours || computed.style.transition.opacity);
        if eased {
            self.ease_style(id, &mut computed);
        }
        let style = &computed.style;
        if let Ok(wanted) = std::env::var("SG_DEBUG_TAG") {
            if matches!(&node.kind, Kind::Element { tag } if *tag == wanted) {
                println!("{wanted} class={:?}\n  {:?}", attribute(node, "class"), style);
            }
        }
        let inherited = Inherited {
            uppercase: inherited.uppercase || style.uppercase,
            font_size: style.font_size.unwrap_or(inherited.font_size),
            colour: style.color.unwrap_or(inherited.colour),
            weight: style.font_weight.unwrap_or(inherited.weight),
            italic: style.italic || inherited.italic,
            mono: style.mono.unwrap_or(inherited.mono),
            underline: style.underline || inherited.underline,
        };

        let mut element = apply_style(div(), style);
        if self.holds_only_text(node) && std::env::var("SG_NO_TEXT_MINW").is_err() {
            element = element.min_w_0();
        }
        let tag = match &node.kind {
            Kind::Element { tag } => tag.as_str(),
            Kind::Text { .. } => "",
        };
        match tag {
            "svg" => element = self.svg_content(element, id, node, style, inherited),
            "img" => element = self.image_content(element, node, style),
            "select" => element = self.select_content(element, id, node, style, inherited, cx),
            "input" | "textarea" => match control_of(node) {
                Some(Control::Text) | Some(Control::Multiline) => {
                    element = self.field_content(element, id, node, style, inherited, cx);
                }
                Some(control) => element = self.checkable_content(element, node, style, control),
                None => {}
            },
            _ => {
                let flow = match style.display {
                    Display::Flex => Flow::Flex,
                    Display::Inline => Flow::Inline,
                    _ => Flow::Block,
                };
                for child in self.children(id, flow, inherited, cx) {
                    element = element.child(child);
                }
            }
        }
        element = self.border_strips(element, style);
        let is_select = matches!(&node.kind, Kind::Element { tag } if tag == "select");
        if node.observed || is_select {
            element = element.child(self.measure(id));
        }
        Some(self.interactive(element, id, node, style, &computed, eased, cx))
    }

    /// Whether an ancestor marked `group` has the pointer over it.
    fn in_hovered_group(&self, id: NodeId) -> bool {
        let hovered = self.interaction.hovered.borrow();
        let mut current = self.tree.nodes.get(&id).and_then(|node| node.parent);
        while let Some(candidate) = current {
            let Some(node) = self.tree.nodes.get(&candidate) else {
                return false;
            };
            let grouped = attribute(node, "class").split_whitespace().any(|name| name == "group");
            if grouped && hovered.contains(&candidate) {
                return true;
            }
            current = node.parent;
        }
        false
    }

    /// Applies hover to the style itself, and eases from the last drawn colours to the new
    /// ones. Elements that ease take hover through here rather than GPUI's own refinement.
    fn ease_style(&self, id: NodeId, computed: &mut Computed) {
        let hovered = self.interaction.hovered.borrow().contains(&id);
        let grouped = self.in_hovered_group(id);
        let style = &mut computed.style;
        for (active, hover) in [(hovered, &computed.hover), (grouped, &computed.group_hover)] {
            if !active {
                continue;
            }
            if hover.background.is_some() {
                style.background = hover.background;
            }
            if hover.color.is_some() {
                style.color = hover.color;
            }
            if hover.opacity.is_some() {
                style.opacity = hover.opacity;
            }
            if let Some(colour) = hover.border_color {
                style.border_color = [Some(colour); 4];
            }
        }
        let target = Eased {
            background: style.background,
            color: style.color,
            border: main_border_colour(style),
            opacity: style.opacity.unwrap_or(1.0),
        };
        let now = Instant::now();
        let transition = style.transition.clone();
        let mut animations = self.interaction.animations.borrow_mut();
        let animation = animations.entry(id).or_insert(Animation {
            from: target,
            to: target,
            start: None,
        });
        if animation.to != target {
            let (current, _) = animation.value_at(now, &transition);
            animation.from = current;
            animation.to = target;
            let delay = Duration::from_secs_f32(transition.delay_ms / 1000.0);
            animation.start = Some(now + delay);
        }
        let (shown, progress) = animation.value_at(now, &transition);
        if progress < 1.0 {
            self.interaction.animating.set(true);
        } else {
            animation.start = None;
        }
        style.background = shown.background;
        style.color = shown.color;
        if let Some(colour) = shown.border {
            for side in style.border_color.iter_mut() {
                if side.is_some() {
                    *side = Some(colour);
                }
            }
        }
        if shown.opacity < 1.0 || style.opacity.is_some() {
            style.opacity = Some(shown.opacity);
        }
        computed.hover = HoverStyle::default();
        computed.group_hover = HoverStyle::default();
    }

    /// Whether an element's children are all text, so it can shrink and let the text wrap.
    fn holds_only_text(&self, node: &Node) -> bool {
        !node.children.is_empty()
            && node
                .children
                .iter()
                .all(|child| matches!(self.tree.nodes.get(child).map(|n| &n.kind), Some(Kind::Text { .. })))
    }

    /// An invisible layer that records how big its parent came out.
    fn measure(&self, id: NodeId) -> AnyElement {
        let sizes = self.interaction.sizes.clone();
        let placed = self.interaction.bounds.clone();
        canvas(
            move |bounds, _, _| {
                let size = (f32::from(bounds.size.width), f32::from(bounds.size.height));
                sizes.borrow_mut().insert(id, size);
                placed.borrow_mut().insert(id, bounds);
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full()
        .into_any_element()
    }

    /// An `<svg>` becomes a document GPUI draws from memory, tinted with the text colour.
    fn svg_content(
        &self,
        mut element: Div,
        id: NodeId,
        node: &Node,
        style: &Style,
        inherited: Inherited,
    ) -> Div {
        let (width, height) = intrinsic_size(node, style, inherited.font_size);
        if style.width.is_none() {
            element = element.w(px(width));
        }
        if style.height.is_none() {
            element = element.h(px(height));
        }
        let markup = self.serialize_svg(id);
        if std::env::var("SVELTE_GPUI_TRACE_SVG").is_ok() {
            println!("svg {id}: {width}x{height} {markup}");
        }
        let path = register_svg(markup);
        element.child(
            svg()
                .path(SharedString::from(path))
                .size_full()
                .text_color(inherited.colour),
        )
    }

    fn serialize_svg(&self, id: NodeId) -> String {
        let mut out = String::new();
        self.write_svg(id, true, &mut out);
        out
    }

    fn write_svg(&self, id: NodeId, root: bool, out: &mut String) {
        let Some(node) = self.tree.nodes.get(&id) else {
            return;
        };
        let Kind::Element { tag } = &node.kind else {
            return;
        };
        out.push('<');
        out.push_str(tag);
        if root {
            out.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
        }
        let mut names: Vec<&String> = node.attributes.keys().collect();
        names.sort();
        for name in names {
            if matches!(name.as_str(), "class" | "style" | "xmlns") || (root && matches!(name.as_str(), "width" | "height")) {
                continue;
            }
            out.push(' ');
            out.push_str(name);
            out.push_str("=\"");
            out.push_str(&node.attributes[name].replace('"', "&quot;"));
            out.push('"');
        }
        if node.children.is_empty() {
            out.push_str("/>");
            return;
        }
        out.push('>');
        for child in &node.children {
            self.write_svg(*child, false, out);
        }
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
    }

    /// An `<img>` draws its file once the source has resolved to one.
    fn image_content(&self, mut element: Div, node: &Node, style: &Style) -> Div {
        let (width, height) = intrinsic_size(node, style, DEFAULT_FONT_SIZE);
        if style.width.is_none() && node.attributes.contains_key("width") {
            element = element.w(px(width));
        }
        if style.height.is_none() && node.attributes.contains_key("height") {
            element = element.h(px(height));
        }
        let source = attribute(node, "src");
        if source.is_empty() {
            return element;
        }
        let ImageState::Ready(path) = self.interaction.images.state(source) else {
            return element;
        };
        let fit = match style.fit {
            Fit::Cover => ObjectFit::Cover,
            Fit::Fill => ObjectFit::Fill,
            Fit::Contain => ObjectFit::Contain,
        };
        let mut picture = img(path).size_full().object_fit(fit);
        let [top_left, top_right, bottom_right, bottom_left] = style.radius;
        picture.style().corner_radii.top_left = Some(px(top_left).into());
        picture.style().corner_radii.top_right = Some(px(top_right).into());
        picture.style().corner_radii.bottom_right = Some(px(bottom_right).into());
        picture.style().corner_radii.bottom_left = Some(px(bottom_left).into());
        element.child(picture)
    }

    /// A theme colour by its custom property, for the parts of a control CSS does not reach.
    fn theme_colour(&self, name: &str, fallback: u32) -> Rgba {
        match self.resolver.sheet.variables.get(name) {
            Some(value) => crate::css::parse_colour(value).unwrap_or(gpui::rgb(fallback)),
            None => gpui::rgb(fallback),
        }
    }

    /// The text of a `textarea` that has no `value` of its own yet: what is written inside it.
    fn initial_text(&self, id: NodeId, node: &Node) -> String {
        if let Some(value) = node.attributes.get("value") {
            return value.clone();
        }
        match &node.kind {
            Kind::Element { tag } if tag == "textarea" => self.tree.text_content(id),
            _ => String::new(),
        }
    }

    /// The editable text of an `input` or `textarea`, with its selection and caret.
    fn field_content(
        &self,
        element: Div,
        id: NodeId,
        node: &Node,
        style: &Style,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
    ) -> Div {
        let multiline = control_of(node) == Some(Control::Multiline);
        let password = attribute(node, "type") == "password";
        let placeholder = attribute(node, "placeholder").to_string();
        let handle = self.interaction.focus_handle(id, cx);
        let initial = self.initial_text(id, node);
        let line_height = style.line_height.unwrap_or(1.5);
        let font_size = inherited.font_size;

        let (shown, cursor, selection, scroll, composing) = {
            let mut fields = self.interaction.fields.borrow_mut();
            let field = fields.entry(id).or_insert_with(Field::new);
            if field.applied.as_deref() != Some(initial.as_str()) {
                field.text = initial.clone();
                field.cursor = field.text.len();
                field.anchor = field.cursor;
                field.applied = Some(initial);
            }
            field.cursor = field.cursor.min(field.text.len());
            field.anchor = field.anchor.min(field.text.len());
            let shown = if password {
                "\u{2022}".repeat(field.text.chars().count())
            } else {
                field.text.clone()
            };
            let map = |byte: usize| -> usize {
                if password {
                    field.text[..byte].chars().count() * "\u{2022}".len()
                } else {
                    byte
                }
            };
            let selection = field.selection();
            let composing = field.marked.clone().map(|range| map(range.start)..map(range.end));
            (shown, map(field.cursor), map(selection.start)..map(selection.end), field.scroll_x, composing)
        };

        let empty = shown.is_empty();
        let text_to_draw = if empty { placeholder } else { shown };
        let mut styled = StyledText::new(SharedString::from(text_to_draw.clone()));
        if let (Some(range), false) = (composing.clone(), empty) {
            let underline = gpui::UnderlineStyle {
                thickness: px(1.0),
                color: Some(inherited.colour.into()),
                wavy: false,
            };
            let highlight = HighlightStyle {
                underline: Some(underline),
                ..Default::default()
            };
            styled = styled.with_highlights(vec![(range, highlight)]);
        } else if !empty && !selection.is_empty() {
            let highlight = HighlightStyle {
                background_color: Some(Hsla::from(self.theme_colour("--focus-ring", 0x3b6ea8)).opacity(0.45)),
                ..Default::default()
            };
            styled = styled.with_highlights(vec![(selection, highlight)]);
        }
        if !empty {
            let layout = styled.layout().clone();
            if let Some(field) = self.interaction.fields.borrow_mut().get_mut(&id) {
                field.layout = Some(layout);
            }
        }

        let dim = self.theme_colour("--text-dim", 0x71717a);
        let mut text = div();
        if !multiline {
            text = text.whitespace_nowrap().ml(px(-scroll));
        }
        if empty {
            text = text.text_color(dim);
        }
        let text = text.child(styled);

        let fields = self.interaction.fields.clone();
        let caret_colour = inherited.colour;
        let focus = handle.clone();
        let workbench = cx.entity();
        let caret = canvas(
            |bounds, _, _| bounds,
            move |bounds: Bounds<gpui::Pixels>, _, window, cx| {
                // The input method types into whatever is focused, through this handler.
                window.handle_input(
                    &focus,
                    gpui::ElementInputHandler::new(bounds, workbench.clone()),
                    cx,
                );
                if !focus.is_focused(window) {
                    return;
                }
                let mut fields = fields.borrow_mut();
                let Some(field) = fields.get_mut(&id) else {
                    return;
                };
                let measured = match (&field.layout, empty) {
                    (Some(layout), false) => layout
                        .position_for_index(cursor)
                        .map(|position| (position, layout.line_height())),
                    _ => None,
                };
                let (position, height) = match measured {
                    Some(found) => found,
                    None => (bounds.origin, px(font_size * line_height)),
                };
                if !multiline {
                    let relative_x = f32::from(position.x - bounds.origin.x);
                    let width = f32::from(bounds.size.width);
                    let target = if relative_x > width - 2.0 {
                        field.scroll_x + relative_x - (width - 2.0)
                    } else if relative_x < 0.0 {
                        (field.scroll_x + relative_x).max(0.0)
                    } else {
                        field.scroll_x
                    };
                    if (target - field.scroll_x).abs() > 0.5 {
                        field.scroll_x = target;
                        window.request_animation_frame();
                    }
                }
                window.paint_quad(fill(
                    Bounds {
                        origin: position,
                        size: size(px(1.5), height),
                    },
                    caret_colour,
                ));
            },
        )
        .absolute()
        .size_full();

        let mut container = div().relative().flex_1().min_w_0().child(text).child(caret);
        if !multiline {
            container = container.overflow_hidden();
        } else {
            let rows: f32 = attribute(node, "rows").parse().unwrap_or(2.0);
            container = container.min_h(px(rows * font_size * line_height));
        }
        element.child(container)
    }

    /// A closed select shows its chosen option and a caret; an open one adds the list.
    fn select_content(
        &self,
        mut element: Div,
        id: NodeId,
        node: &Node,
        style: &Style,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
    ) -> Div {
        let options = choices(self.tree, id);
        let current = options.iter().find(|option| option.selected);
        let label = match current {
            Some(option) => option.label.clone(),
            None => String::new(),
        };
        let disabled = node.attributes.contains_key("disabled");
        let _ = disabled;
        if style.display != Display::Flex {
            element = element.flex().flex_row();
        }
        element = element.items_center().justify_between().gap_x(px(6.0));
        element = element.child(div().flex_1().min_w_0().truncate().child(SharedString::from(label)));
        let caret = register_svg(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 256 256\"><path d=\"M213.66,101.66l-80,80a8,8,0,0,1-11.32,0l-80-80A8,8,0,0,1,53.66,90.34L128,164.69l74.34-74.35a8,8,0,0,1,11.32,11.32Z\"/></svg>".to_string(),
        );
        element = element.child(
            svg()
                .path(SharedString::from(caret))
                .size(px(inherited.font_size * 0.8))
                .flex_none()
                .text_color(self.theme_colour("--text-dim", 0x71717a)),
        );
        if *self.interaction.open_select.borrow() != Some(id) {
            return element;
        }
        let Some(bounds) = self.interaction.bounds.borrow().get(&id).copied() else {
            return element;
        };
        element.child(self.select_menu(id, &options, bounds, inherited, cx))
    }

    /// The list under an open select, drawn above everything else.
    fn select_menu(
        &self,
        select: NodeId,
        options: &[crate::select::Choice],
        bounds: Bounds<gpui::Pixels>,
        inherited: Inherited,
        cx: &mut Context<Workbench>,
    ) -> AnyElement {
        let background = self.theme_colour("--bg-elevated", 0x262628);
        let border = self.theme_colour("--border-strong", 0x3f3f46);
        let hover = self.theme_colour("--surface-hover", 0x1f1f23);
        let dim = self.theme_colour("--text-dim", 0x71717a);
        let text = self.theme_colour("--text", 0xfafafa);

        let mut menu = div()
            .id(("select-menu", select as usize))
            .occlude()
            .flex()
            .flex_col()
            .min_w(bounds.size.width)
            .max_h(px(260.0))
            .overflow_y_scroll()
            .p(px(4.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(border)
            .bg(background)
            .text_size(px(inherited.font_size))
            .text_color(text)
            .on_mouse_down_out(cx.listener(|workbench, _event: &MouseDownEvent, _window, cx| {
                workbench.close_select(cx);
            }));
        let mut last_group: Option<String> = None;
        for (index, option) in options.iter().enumerate() {
            if option.group != last_group {
                if let Some(group) = &option.group {
                    menu = menu.child(
                        div().px(px(8.0)).py(px(3.0)).text_color(dim).child(SharedString::from(group.clone())),
                    );
                }
                last_group = option.group.clone();
            }
            let mut row = div()
                .id(("select-option", option.id as usize))
                .flex()
                .items_center()
                .justify_between()
                .gap_x(px(12.0))
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(5.0))
                .child(SharedString::from(option.label.clone()));
            if option.disabled {
                row = row.text_color(dim);
            } else {
                row = row.cursor_pointer().hover(move |style| style.bg(hover)).on_click(cx.listener(
                    move |workbench, _event: &ClickEvent, _window, cx| {
                        workbench.choose_option(select, index, cx);
                        cx.stop_propagation();
                    },
                ));
            }
            if option.selected {
                row = row.bg(hover);
            }
            menu = menu.child(row);
        }
        deferred(
            anchored()
                .position(point(bounds.left(), bounds.bottom() + px(2.0)))
                .anchor(Corner::TopLeft)
                .snap_to_window_with_margin(px(8.0))
                .child(menu),
        )
        .with_priority(1)
        .into_any_element()
    }

    /// A checkbox or radio, drawn plainly unless the page styles it.
    fn checkable_content(&self, mut element: Div, node: &Node, style: &Style, control: Control) -> Div {
        let checked = node.attributes.get("checked").map(String::as_str) != Some("false")
            && node.attributes.contains_key("checked");
        let action = self.theme_colour("--primary", 0xfafafa);
        let on_action = self.theme_colour("--primary-fg", 0x0b0b0d);
        if style.width.is_none() {
            element = element.w(px(14.0)).h(px(14.0)).flex_none();
        }
        // Tailwind's reset makes controls transparent, which counts as unstyled here.
        let unpainted = match style.background {
            Some(colour) => colour.a == 0.0,
            None => true,
        };
        if unpainted {
            element = element.bg(if checked { action } else { self.theme_colour("--surface-input", 0x0e0e10) });
        }
        if main_border_colour(style).is_none() {
            element.style().border_widths.top = Some(px(1.0).into());
            element.style().border_widths.right = Some(px(1.0).into());
            element.style().border_widths.bottom = Some(px(1.0).into());
            element.style().border_widths.left = Some(px(1.0).into());
            element = element.border_color(self.theme_colour("--border-strong", 0x3f3f46));
        }
        let radius = if control == Control::Radio { 7.0 } else { 3.0 };
        if style.radius == [0.0; 4] {
            element = element.rounded(px(radius));
        }
        element = element.items_center().justify_center();
        // A page that paints its own checkbox draws its own mark too.
        if !checked || !unpainted {
            return element;
        }
        if control == Control::Radio {
            return element.child(div().size(px(6.0)).rounded_full().bg(on_action));
        }
        let mark = register_svg(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 256 256\"><path d=\"M232.49,80.49l-128,128a12,12,0,0,1-17,0l-56-56a12,12,0,1,1,17-17L96,183,215.51,63.51a12,12,0,0,1,17,17Z\"/></svg>".to_string(),
        );
        element.child(svg().path(SharedString::from(mark)).size(px(10.0)).text_color(on_action))
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

    /// Whether the app delegates an event type: it listens for it at the root.
    fn delegated(&self, name: &str) -> bool {
        match self.tree.nodes.get(&ROOT) {
            Some(root) => root.listeners.contains(name),
            None => false,
        }
    }

    /// How an element takes an event type: delegated events bubble from the deepest
    /// element under the pointer, direct ones go to the element that asked for them.
    fn route(&self, node: &Node, name: &str) -> Option<Route> {
        if self.delegated(name) {
            return Some(Route::Delegated);
        }
        if node.listeners.contains(name) {
            return Some(Route::Direct);
        }
        None
    }

    /// Gives an element an id, so it can take clicks, hover, focus and scrolling.
    fn interactive(
        &self,
        element: Div,
        id: NodeId,
        node: &Node,
        style: &Style,
        computed: &Computed,
        eased: bool,
        cx: &mut Context<Workbench>,
    ) -> AnyElement {
        let mut element = element.id(id as usize);
        let grouped = attribute(node, "class").split_whitespace().any(|name| name == "group");
        if grouped {
            element = element.group("group");
        }
        if eased || grouped {
            element = element.on_hover(cx.listener(move |workbench, hovered: &bool, _window, cx| {
                workbench.hover_changed(id, *hovered, cx);
            }));
        }
        if is_focusable(node) {
            let handle = self.interaction.focus_handle(id, cx);
            element = element.track_focus(&handle);
        }
        if style.scroll_y || style.scroll_x {
            let handle = self.interaction.scroll_handle(id);
            element = element.track_scroll(&handle);
            if style.scroll_y {
                element = element.overflow_y_scroll();
            }
            if style.scroll_x {
                element = element.overflow_x_scroll();
            }
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
        let element = self.wire_field(element, id, node, cx);
        let element = self.wire_clicks(element, id, node, cx);
        let element = self.wire_buttons(element, id, node, cx);
        let element = self.wire_motion(element, id, node, cx);
        self.wire_wheel(element, id, node, cx).into_any_element()
    }

    fn wire_clicks(
        &self,
        element: gpui::Stateful<Div>,
        id: NodeId,
        node: &Node,
        cx: &mut Context<Workbench>,
    ) -> gpui::Stateful<Div> {
        let click = self.route(node, "click");
        let double = self.route(node, "dblclick");
        let has_default = control_of(node).is_some()
            || matches!(&node.kind, Kind::Element { tag } if tag == "label" || tag == "select");
        if click.is_none() && double.is_none() && !has_default {
            return element;
        }
        element.on_click(cx.listener(move |workbench, event: &ClickEvent, window, cx| {
            let detail = pointer_detail(event.position(), event.modifiers(), 0, event.click_count());
            let mut prevented = false;
            if let Some(route) = click {
                prevented = workbench.emit(id, "click", route.detail(&detail), cx).prevented;
            }
            if event.click_count() == 2 {
                if let Some(route) = double {
                    workbench.emit(id, "dblclick", route.detail(&detail), cx);
                }
            }
            if has_default && !prevented {
                workbench.click_default(id, window, cx);
            }
            let delegated = click == Some(Route::Delegated) || double == Some(Route::Delegated);
            if delegated || has_default {
                cx.stop_propagation();
            }
        }))
    }

    /// Mouse handling for a text field: place the cursor, and drag to select.
    fn wire_field(
        &self,
        element: gpui::Stateful<Div>,
        id: NodeId,
        node: &Node,
        cx: &mut Context<Workbench>,
    ) -> gpui::Stateful<Div> {
        if !matches!(control_of(node), Some(Control::Text) | Some(Control::Multiline)) {
            return element;
        }
        element
            .cursor_text()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |workbench, event: &MouseDownEvent, _window, cx| {
                    workbench.field_mouse_down(id, event.position, event.modifiers.shift);
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(move |workbench, event: &MouseMoveEvent, _window, cx| {
                workbench.field_mouse_move(id, event.position);
                cx.notify();
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |workbench, _event: &MouseUpEvent, _window, _cx| {
                    workbench.field_mouse_up(id);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(move |workbench, _event: &MouseUpEvent, _window, _cx| {
                    workbench.field_mouse_up(id);
                }),
            )
    }

    /// `mousedown`, `mouseup`, `pointerdown`, `pointerup` and `contextmenu`.
    fn wire_buttons(
        &self,
        mut element: gpui::Stateful<Div>,
        id: NodeId,
        node: &Node,
        cx: &mut Context<Workbench>,
    ) -> gpui::Stateful<Div> {
        let down: Vec<(&'static str, Route)> = ["mousedown", "pointerdown"]
            .into_iter()
            .filter_map(|name| self.route(node, name).map(|route| (name, route)))
            .collect();
        let menu = self.route(node, "contextmenu");
        let up: Vec<(&'static str, Route)> = ["mouseup", "pointerup"]
            .into_iter()
            .filter_map(|name| self.route(node, name).map(|route| (name, route)))
            .collect();
        if !down.is_empty() || menu.is_some() {
            for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
                let down = down.clone();
                element = element.on_mouse_down(
                    button,
                    cx.listener(move |workbench, event: &MouseDownEvent, _window, cx| {
                        let detail = pointer_detail(
                            event.position,
                            event.modifiers,
                            button_number(event.button),
                            event.click_count,
                        );
                        let mut delegated = false;
                        for (name, route) in &down {
                            workbench.emit(id, name, route.detail(&detail), cx);
                            delegated |= *route == Route::Delegated;
                        }
                        if event.button == MouseButton::Right {
                            if let Some(route) = menu {
                                workbench.emit(id, "contextmenu", route.detail(&detail), cx);
                                delegated |= route == Route::Delegated;
                            }
                        }
                        if delegated {
                            cx.stop_propagation();
                        }
                    }),
                );
            }
        }
        if !up.is_empty() {
            for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
                let up = up.clone();
                element = element.on_mouse_up(
                    button,
                    cx.listener(move |workbench, event: &MouseUpEvent, _window, cx| {
                        let detail = pointer_detail(
                            event.position,
                            event.modifiers,
                            button_number(event.button),
                            event.click_count,
                        );
                        let mut delegated = false;
                        for (name, route) in &up {
                            workbench.emit(id, name, route.detail(&detail), cx);
                            delegated |= *route == Route::Delegated;
                        }
                        if delegated {
                            cx.stop_propagation();
                        }
                    }),
                );
            }
        }
        element
    }

    /// Hover in and out, in every spelling the DOM has for it, and pointer movement.
    fn wire_motion(
        &self,
        mut element: gpui::Stateful<Div>,
        id: NodeId,
        node: &Node,
        cx: &mut Context<Workbench>,
    ) -> gpui::Stateful<Div> {
        let entering = self.routes(node, &["mouseenter", "pointerenter", "mouseover", "pointerover"]);
        let leaving = self.routes(node, &["mouseleave", "pointerleave", "mouseout", "pointerout"]);
        if !entering.is_empty() || !leaving.is_empty() {
            element = element.on_hover(cx.listener(move |workbench, hovered: &bool, _window, cx| {
                let names = if *hovered { &entering } else { &leaving };
                for (name, route) in names {
                    workbench.emit(id, name, route.detail("{}"), cx);
                }
            }));
        }
        let moving = self.routes(node, &["mousemove", "pointermove"]);
        if moving.is_empty() {
            return element;
        }
        element.on_mouse_move(cx.listener(move |workbench, event: &MouseMoveEvent, _window, cx| {
            let detail = pointer_detail(event.position, event.modifiers, 0, 0);
            for (name, route) in &moving {
                workbench.emit(id, name, route.detail(&detail), cx);
            }
        }))
    }

    fn routes(&self, node: &Node, names: &[&'static str]) -> Vec<(&'static str, Route)> {
        names
            .iter()
            .filter_map(|name| self.route(node, name).map(|route| (*name, route)))
            .collect()
    }

    /// `wheel` goes only to elements that asked for it, so scrolling still reaches the
    /// containers underneath.
    fn wire_wheel(
        &self,
        element: gpui::Stateful<Div>,
        id: NodeId,
        node: &Node,
        cx: &mut Context<Workbench>,
    ) -> gpui::Stateful<Div> {
        if !node.listeners.contains("wheel") {
            return element;
        }
        element.on_scroll_wheel(cx.listener(move |workbench, event: &ScrollWheelEvent, _window, cx| {
            let (dx, dy) = match event.delta {
                ScrollDelta::Pixels(delta) => (f32::from(delta.x), f32::from(delta.y)),
                ScrollDelta::Lines(delta) => (delta.x * 16.0, delta.y * 16.0),
            };
            // GPUI's delta is how far the content moves; the DOM's is where the view goes.
            let detail = format!(
                "{{\"deltaX\":{},\"deltaY\":{},\"deltaMode\":0,\"bubbles\":false{}}}",
                -dx,
                -dy,
                modifier_fields(event.modifiers)
            );
            workbench.emit(id, "wheel", detail, cx);
        }))
    }
}

/// How an event reaches the app.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Route {
    /// From the deepest element under the pointer, bubbling up through the app.
    Delegated,
    /// To the element that listens for it, and nowhere else.
    Direct,
}

impl Route {
    /// The detail with the right bubbling for this route.
    fn detail(&self, base: &str) -> String {
        match self {
            Route::Delegated => base.to_string(),
            Route::Direct => {
                let inner = base.trim_start_matches('{').trim_end_matches('}');
                if inner.is_empty() {
                    return "{\"bubbles\":false}".to_string();
                }
                format!("{{\"bubbles\":false,{inner}}}")
            }
        }
    }
}

fn button_number(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        _ => 3,
    }
}

fn modifier_fields(modifiers: gpui::Modifiers) -> String {
    format!(
        ",\"ctrlKey\":{},\"shiftKey\":{},\"altKey\":{},\"metaKey\":{}",
        modifiers.control, modifiers.shift, modifiers.alt, modifiers.platform
    )
}

/// The fields a DOM mouse event carries.
fn pointer_detail(
    position: gpui::Point<gpui::Pixels>,
    modifiers: gpui::Modifiers,
    button: i32,
    clicks: usize,
) -> String {
    format!(
        "{{\"clientX\":{},\"clientY\":{},\"button\":{},\"detail\":{}{}}}",
        f32::from(position.x),
        f32::from(position.y),
        button,
        clicks,
        modifier_fields(modifiers)
    )
}

fn attribute<'a>(node: &'a Node, name: &str) -> &'a str {
    match node.attributes.get(name) {
        Some(value) => value.as_str(),
        None => "",
    }
}

/// Elements a browser lets take focus: form controls, links with an address, and
/// anything given a `tabindex` or made editable.
fn is_focusable(node: &Node) -> bool {
    if node.attributes.contains_key("tabindex") || node.attributes.contains_key("contenteditable") {
        return true;
    }
    match &node.kind {
        Kind::Element { tag } => match tag.as_str() {
            "button" | "input" | "select" | "textarea" => !node.attributes.contains_key("disabled"),
            "a" => node.attributes.contains_key("href"),
            _ => false,
        },
        Kind::Text { .. } => false,
    }
}

/// The size an `svg` or `img` takes from its attributes, when CSS says nothing.
fn intrinsic_size(node: &Node, style: &Style, font_size: f32) -> (f32, f32) {
    let read = |name: &str, fallback: f32| -> f32 {
        let Some(value) = node.attributes.get(name) else {
            return fallback;
        };
        let trimmed = value.trim();
        if let Some(ems) = trimmed.strip_suffix("em") {
            return ems.parse::<f32>().map(|ems| ems * font_size).unwrap_or(fallback);
        }
        trimmed.trim_end_matches("px").parse().unwrap_or(fallback)
    };
    let width = read("width", font_size);
    let height = read("height", width);
    let width = match style.width {
        Some(Dimension::Px(pixels)) => pixels,
        _ => width,
    };
    let height = match style.height {
        Some(Dimension::Px(pixels)) => pixels,
        _ => height,
    };
    (width, height)
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
    let (shift_x, shift_y) = style.translate;
    if !style.absolute && (shift_x != 0.0 || shift_y != 0.0) {
        element = element.relative().left(px(shift_x)).top(px(shift_y));
    }
    if style.absolute {
        element = element.absolute();
        let shifted = |value: Option<Dimension>, shift: f32| match value {
            Some(Dimension::Px(pixels)) => Some(Dimension::Px(pixels + shift)),
            None if shift != 0.0 => Some(Dimension::Px(shift)),
            other => other,
        };
        let [top, right, bottom, left] = style.inset;
        let top = shifted(top, shift_y);
        let left = shifted(left, shift_x);
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
