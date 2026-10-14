//! Draws the host's tree with GPUI. Each element becomes a `div` with the style
//! the stylesheet gives it; text nodes become the text of their parent. The tree
//! is walked afresh on every frame, which is cheap next to the layout and paint
//! that follow, and means nothing here has to track what changed.
//!
//! Input goes the other way. Which events an element listens to decides which
//! GPUI handlers it gets, and each handler reports to the app by node id.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Context, Div, FocusHandle, FontWeight, IntoElement, Length,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ObjectFit, Rgba, ScrollDelta,
    ScrollHandle, ScrollWheelEvent, SharedString, StyledImage, auto, canvas, div, img,
    prelude::*, px, relative, svg,
};

use crate::Workbench;
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
                workbench.key(event, true, window, cx);
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
        self.resolver.compute(tag, attribute(node, "class"), &node.style)
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
            font_size: style.font_size.unwrap_or(inherited.font_size),
            colour: style.color.unwrap_or(inherited.colour),
        };

        let mut element = apply_style(div(), style);
        let tag = match &node.kind {
            Kind::Element { tag } => tag.as_str(),
            Kind::Text { .. } => "",
        };
        match tag {
            "svg" => element = self.svg_content(element, id, node, style, inherited),
            "img" => element = self.image_content(element, node, style),
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
        if node.observed {
            element = element.child(self.measure(id));
        }
        Some(self.interactive(element, id, node, style, &computed, cx))
    }

    /// An invisible layer that records how big its parent came out.
    fn measure(&self, id: NodeId) -> AnyElement {
        let sizes = self.interaction.sizes.clone();
        canvas(
            move |bounds, _, _| {
                let size = (f32::from(bounds.size.width), f32::from(bounds.size.height));
                sizes.borrow_mut().insert(id, size);
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
        cx: &mut Context<Workbench>,
    ) -> AnyElement {
        let mut element = element.id(id as usize);
        if attribute(node, "class").split_whitespace().any(|name| name == "group") {
            element = element.group("group");
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
        if click.is_none() && double.is_none() {
            return element;
        }
        element.on_click(cx.listener(move |workbench, event: &ClickEvent, _window, cx| {
            let detail = pointer_detail(event.position(), event.modifiers(), 0, event.click_count());
            if let Some(route) = click {
                workbench.emit(id, "click", route.detail(&detail), cx);
            }
            if event.click_count() == 2 {
                if let Some(route) = double {
                    workbench.emit(id, "dblclick", route.detail(&detail), cx);
                }
            }
            if click == Some(Route::Delegated) || double == Some(Route::Delegated) {
                cx.stop_propagation();
            }
        }))
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
