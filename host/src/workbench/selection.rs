//! Selecting text outside fields. A press on a `Text` with `selectable` starts a selection and
//! dragging extends it, into other selectable paragraphs too; a double click takes a word and a
//! triple click the paragraph, and dragging on grows by those. Shift extends the selection there was.
//! The window tells the workbench where the pointer goes while the button is down
//! (`render/press.rs`), and a press anywhere ends the selection before the element that was pressed
//! makes its own.

use std::cmp::Ordering;

use gpui::{Context, MouseButton, MouseMoveEvent, Pixels, Point};

use super::Workbench;
use crate::render::{Spot, TextSelection, Unit, changed, compare, forget, parts, resolve, unit_at};
use crate::tree::{NodeId, ROOT};

/// What the press that is going on has done so far.
#[derive(Default)]
pub(super) struct TextPress {
    /// A selectable paragraph heard it, so no other one should, and it is not a press on nothing.
    claimed: bool,
    /// Shift was held with a selection there, which this press extends instead of replacing.
    extending: bool,
}

impl Workbench {
    /// A left press began, before any element heard it: what was selected is not any more, unless
    /// shift is held, when a selectable text that hears the press extends it. A selectable text that
    /// is pressed makes a new selection when it hears the press.
    pub(crate) fn text_press_began(&mut self, shift: bool, cx: &mut Context<Self>) {
        let extending = shift && self.interaction.text_selection.borrow().is_some();
        self.text_press = TextPress { claimed: false, extending };
        if !extending {
            self.apply_text_selection(None, cx);
        }
    }

    /// The press has gone through the elements under it. One that was meant to extend the selection and
    /// landed on nothing selectable ends it, as any press elsewhere does.
    pub(crate) fn text_press_ended(&mut self, cx: &mut Context<Self>) {
        let press = std::mem::take(&mut self.text_press);
        if press.extending && !press.claimed {
            self.apply_text_selection(None, cx);
        }
    }

    /// A left press on a selectable text: place the selection there, take the word or the paragraph
    /// for a double or a triple click, or with shift extend the selection to here.
    pub(crate) fn text_mouse_down(
        &mut self,
        id: NodeId,
        position: Point<Pixels>,
        clicks: usize,
        shift: bool,
        cx: &mut Context<Self>,
    ) {
        // A paragraph inside this one heard the press first and has the selection.
        if self.text_press.claimed {
            return;
        }
        let extending = self.text_press.extending && shift;
        let selection = {
            let texts = self.interaction.texts.borrow();
            let Some(paragraph) = texts.get(&id) else {
                return;
            };
            let by = if extending { 1 } else { clicks };
            let spot = Spot { id, index: paragraph.index_at(position, by <= 1) };
            let unit = match by {
                0 | 1 => Unit::Char,
                2 => Unit::Word,
                _ => Unit::Paragraph,
            };
            let first = unit_at(unit, spot, &paragraph.text);
            let held = self.interaction.text_selection.borrow().clone();
            match held {
                Some(held) if extending => {
                    TextSelection::new(held.anchor, spot, Unit::Char, (held.anchor, held.anchor), true)
                }
                _ if unit == Unit::Char => TextSelection::new(spot, spot, unit, first, true),
                _ => TextSelection::new(first.0, first.1, unit, first, true),
            }
        };
        self.text_press.claimed = true;
        self.apply_text_selection(Some(selection), cx);
    }

    /// The pointer is over this selectable paragraph, as GPUI's own hit test has it, which respects what
    /// clips and covers it. The first one to say so for a move is the innermost.
    pub(crate) fn text_pointer_over(&mut self, id: NodeId, position: Point<Pixels>) {
        let dragging = self.interaction.text_selection.borrow().as_ref().is_some_and(|selection| selection.dragging);
        if dragging && self.text_over.is_none_or(|(_, seen)| seen != position) {
            self.text_over = Some((id, position));
        }
    }

    /// The pointer moved anywhere in the window: while the button is held from a press on selectable
    /// text, the end of the selection follows it, wherever it is.
    pub(crate) fn text_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let over = self.text_over.take().filter(|(_, seen)| *seen == event.position).map(|(id, _)| id);
        let Some(held) = self.interaction.text_selection.borrow().clone() else {
            return;
        };
        if !held.dragging {
            return;
        }
        if event.pressed_button != Some(MouseButton::Left) {
            // The button was let go where the window did not hear it.
            self.text_mouse_up();
            return;
        }
        let nearest = held.unit == Unit::Char;
        let Some(spot) = self.spot_at(event.position, over, nearest) else {
            return;
        };
        let Some(text) = self.interaction.texts.borrow().get(&spot.id).map(|paragraph| paragraph.text.clone()) else {
            return;
        };
        let pointer = unit_at(held.unit, spot, &text);
        let order = if pointer.0.id == held.first.0.id { Vec::new() } else { self.document_order() };
        let (anchor, cursor) = resolve(held.unit, held.first, held.anchor, pointer, &order);
        if (anchor, cursor) == (held.anchor, held.cursor) {
            return;
        }
        let mut moved = held;
        (moved.anchor, moved.cursor) = (anchor, cursor);
        self.apply_text_selection(Some(moved), cx);
    }

    /// The button was released: the selection stays, and the pointer no longer moves it.
    pub(crate) fn text_mouse_up(&mut self) {
        self.text_over = None;
        if let Some(selection) = self.interaction.text_selection.borrow_mut().as_mut() {
            selection.dragging = false;
        }
    }

    /// What is selected outside fields, joined with a line break between paragraphs, if anything is.
    pub(super) fn selected_paragraph_text(&self) -> Option<String> {
        let selection = self.interaction.text_selection.borrow();
        let selection = selection.as_ref()?;
        let tree = self.svelte.tree.borrow();
        let texts = self.interaction.texts.borrow();
        let pieces: Vec<String> = selection
            .parts
            .iter()
            .filter(|(id, _)| tree.nodes.contains_key(id))
            .filter_map(|(id, range)| texts.get(id).map(|paragraph| paragraph.copy(range.clone())))
            .filter(|piece| !piece.is_empty())
            .collect();
        (!pieces.is_empty()).then(|| pieces.join("\n"))
    }

    /// Paragraphs were removed from the tree: forget their layouts, and drop the selection if one of its
    /// ends was in one. A selection across paragraphs is worked out again when the tree changed at all,
    /// as the paragraphs between its ends may have.
    pub(super) fn forget_dropped_text(&mut self, cx: &mut Context<Self>) {
        let (gone, changed_tree) = {
            let mut tree = self.svelte.tree.borrow_mut();
            let changed = !tree.dirty.is_empty() || !tree.dirty_deep.is_empty();
            (std::mem::take(&mut tree.dropped_selectable), changed)
        };
        let held = self.interaction.text_selection.borrow().clone();
        let across = held.as_ref().is_some_and(|held| held.anchor.id != held.cursor.id);
        if gone.is_empty() && !(across && changed_tree) {
            return;
        }
        let mut kept = held;
        forget(&mut self.interaction.texts.borrow_mut(), &mut kept, &gone);
        self.apply_text_selection(kept, cx);
    }

    /// The selectable paragraphs that are in the tree, in document order.
    fn document_order(&self) -> Vec<NodeId> {
        let tree = self.svelte.tree.borrow();
        let texts = self.interaction.texts.borrow();
        let mut order = Vec::new();
        let mut pending = vec![ROOT];
        while let Some(next) = pending.pop() {
            if texts.contains_key(&next) {
                order.push(next);
            }
            if let Some(node) = tree.nodes.get(&next) {
                pending.extend(node.children.iter().rev());
            }
        }
        order
    }

    /// Where in the selectable text a pointer at `position` is. Over a paragraph (`over`) it is where
    /// in that one; between and around them it is the nearest in reading order: above a paragraph
    /// is its start, and below the last is its end.
    fn spot_at(&self, position: Point<Pixels>, over: Option<NodeId>, nearest: bool) -> Option<Spot> {
        if let Some(id) = over {
            let texts = self.interaction.texts.borrow();
            let paragraph = texts.get(&id)?;
            return Some(Spot { id, index: paragraph.index_at(position, nearest) });
        }
        let order = self.document_order();
        let texts = self.interaction.texts.borrow();
        let bounds = self.interaction.bounds.borrow();
        let measured: Vec<NodeId> = order.into_iter().filter(|id| bounds.contains_key(id)).collect();
        for id in &measured {
            let area = bounds[id];
            if area.bottom() > position.y {
                let index = if position.y < area.top() { 0 } else { texts[id].index_at(position, nearest) };
                return Some(Spot { id: *id, index });
            }
        }
        let last = measured.last()?;
        Some(Spot { id: *last, index: texts[last].text.len() })
    }

    /// Replaces the selection, working out what it covers, and has the paragraphs whose part of it
    /// changed drawn again.
    fn apply_text_selection(&mut self, selection: Option<TextSelection>, cx: &mut Context<Self>) {
        let mut selection = selection;
        if let Some(selection) = &mut selection {
            let (a, b) = (selection.anchor, selection.cursor);
            let order = if a.id == b.id { Vec::new() } else { self.document_order() };
            let (start, end) = if compare(a, b, &order) == Ordering::Greater { (b, a) } else { (a, b) };
            let texts = self.interaction.texts.borrow();
            let length = |id| texts.get(&id).map_or(0, |paragraph| paragraph.text.len());
            selection.set_parts(parts(start, end, &order, length));
        }
        let before = self.interaction.text_selection.replace(selection.clone());
        for id in changed(before.as_ref(), selection.as_ref()) {
            self.touch(id, cx);
        }
    }
}
