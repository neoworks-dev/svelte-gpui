//! Rows of a scroller drawn as views of their own that move, instead of being laid out and painted
//! again whenever the scroller moves.
//!
//! A scroller with the `rows` attribute (which `List` sets) draws each of its children that holds
//! enough nodes as a relocatable view (`AnyView::cached_relocatable`). Scrolling only changes where
//! those rows sit, so what they painted is moved, and a row that did not change costs the layout
//! one leaf of the height it had. A row that changes is drawn again, and only that row.
//!
//! Views do not nest, so nothing around a scroller with such rows is a view either: the parts of the
//! document above it are built every frame, as the parts that are not views always are.

use std::collections::HashSet;
use std::rc::Rc;

use gpui::{AnyElement, AnyView, Context, IntoElement, div, prelude::*};

use super::{NodeView, Painter, Primitive, VIEW_MIN_WEIGHT, primitive_of};
use crate::style::{self, Dimension, Style};
use crate::tree::{Kind, NodeId};
use crate::workbench::Workbench;

/// Which elements are rows drawn as views, and which hold them, for one version of the tree.
#[derive(Default)]
pub struct RowPlan {
    /// The scrollers with rows that are views, and everything above them.
    pub hosts: HashSet<NodeId>,
    pub rows: HashSet<NodeId>,
}

impl Painter<'_> {
    /// The plan for the tree as it is now, worked out once per version of it.
    pub(super) fn row_plan(&self) -> Rc<RowPlan> {
        {
            let cache = self.interaction.row_plan.borrow();
            if cache.0 == self.tree.version {
                return cache.1.clone();
            }
        }
        let plan = Rc::new(self.plan_rows());
        *self.interaction.row_plan.borrow_mut() = (self.tree.version, plan.clone());
        plan
    }

    fn plan_rows(&self) -> RowPlan {
        let mut plan = RowPlan::default();
        for (id, node) in &self.tree.nodes {
            let Kind::Element { tag } = &node.kind else {
                continue;
            };
            if tag != "g-scroll" || !node.attributes.contains_key("rows") || node.attributes.contains_key("horizontal")
            {
                continue;
            }
            let scroller_style = self.computed(*id).style.clone();
            let rows: Vec<NodeId> =
                node.children.iter().copied().filter(|child| self.can_be_a_row(*child, &scroller_style)).collect();
            if rows.is_empty() {
                continue;
            }
            plan.rows.extend(rows);
            self.add_with_ancestors(*id, &mut plan.hosts);
        }
        plan
    }

    fn add_with_ancestors(&self, id: NodeId, hosts: &mut HashSet<NodeId>) {
        let mut current = Some(id);
        while let Some(next) = current {
            if !hosts.insert(next) {
                return;
            }
            current = self.tree.nodes.get(&next).and_then(|node| node.parent);
        }
    }

    /// Whether the child of a scroller with rows is worth a view and can be drawn from what it
    /// painted before: it is sized by its content alone, and holds nothing that keeps positions of its own.
    fn can_be_a_row(&self, id: NodeId, scroller: &Style) -> bool {
        let Some(node) = self.tree.nodes.get(&id) else {
            return false;
        };
        let Some(primitive) = primitive_of(node) else {
            return false;
        };
        if matches!(primitive, Primitive::Overlay | Primitive::Input | Primitive::Element) {
            // The row itself cannot be one of these; what it holds is checked below.
            return false;
        }
        if self.weight(id) < VIEW_MIN_WEIGHT || !sized_by_content(&self.computed(id).style, scroller) {
            return false;
        }
        !self.holds_what_cannot_move(id)
    }

    /// Whether anything under the element is a field or a view of the app's own, which keep bounds
    /// that only their own drawing updates. An overlay is fine: GPUI sees that a row has deferred its
    /// drawing and draws that row from scratch for as long as it does.
    fn holds_what_cannot_move(&self, id: NodeId) -> bool {
        let mut pending = vec![id];
        while let Some(next) = pending.pop() {
            let Some(node) = self.tree.nodes.get(&next) else {
                continue;
            };
            if let Some(Primitive::Input | Primitive::Element) = primitive_of(node) {
                return true;
            }
            pending.extend(node.children.iter().copied());
        }
        false
    }

    /// The element as a row view, when it is one.
    pub(super) fn row_view_element(
        &self,
        id: NodeId,
        style: &Style,
        inherited: &super::Inherited,
        cx: &mut Context<Workbench>,
    ) -> Option<AnyElement> {
        if !self.row_plan().rows.contains(&id) {
            return None;
        }
        let workbench = self.interaction.workbench.borrow().clone()?;
        let existing = self.interaction.views.borrow().get(&id).cloned();
        let entity = match existing {
            Some(entity) => entity,
            None => {
                let inherited = inherited.clone();
                let entity = cx.new(|_| NodeView { workbench, root: id, inherited, row: true });
                self.interaction.views.borrow_mut().insert(id, entity.clone());
                entity
            }
        };
        entity.update(cx, |view, cx| {
            if view.inherited != *inherited {
                view.inherited = inherited.clone();
                cx.notify();
            }
        });
        self.interaction.view_roots.borrow_mut().reach(id);
        let mut outline = style::apply(div().flex(), style);
        let refinement = outline.style().clone();
        Some(AnyView::from(entity).cached_relocatable(refinement).into_any_element())
    }
}

/// Whether a row in a scroller of style `scroller` takes the width the scroller gives it and the
/// height of its content, so that it can be laid out as a leaf of the height it had and drawn again
/// at the width it is given: stretched across a column, not growing, with no margins to add to its
/// box, and in the flow.
fn sized_by_content(style: &Style, scroller: &Style) -> bool {
    let no_margin = |side: &Option<Dimension>| matches!(side, None | Some(Dimension::Px(0.0)));
    let unset = |dimension: Option<Dimension>| matches!(dimension, None | Some(Dimension::Auto));
    let stretches = |align: Option<style::Align>| matches!(align, None | Some(style::Align::Stretch));
    scroller.row != Some(true)
        && stretches(scroller.align_items)
        && stretches(style.align_self)
        && unset(style.width)
        && unset(style.min_width)
        && unset(style.max_width)
        && style.aspect_ratio.is_none()
        && style.absolute != Some(true)
        && style.translate.is_none()
        && style.hidden != Some(true)
        && style.grow.unwrap_or(0.0) == 0.0
        && style.margin.iter().all(no_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column() -> Style {
        let mut style = Style::default();
        style.row = Some(false);
        style
    }

    #[test]
    fn a_row_that_is_only_as_tall_as_its_content_can_be_laid_out_as_a_leaf() {
        let mut style = Style::default();
        style.shrink = Some(0.0);
        style.padding = [Some(8.0), None, Some(8.0), None];
        assert!(sized_by_content(&style, &column()));
    }

    #[test]
    fn a_row_that_grows_has_a_height_the_layout_around_it_decides() {
        let mut style = Style::default();
        style.grow = Some(1.0);
        assert!(!sized_by_content(&style, &column()));
    }

    #[test]
    fn a_margin_is_outside_the_box_a_row_records_so_it_is_not_cached() {
        let mut style = Style::default();
        style.margin = [Some(Dimension::Px(4.0)), None, None, None];
        assert!(!sized_by_content(&style, &column()));
        style.margin = [Some(Dimension::Px(0.0)), None, None, None];
        assert!(sized_by_content(&style, &column()));
    }

    #[test]
    fn a_row_outside_the_flow_is_not_cached() {
        let mut style = Style::default();
        style.absolute = Some(true);
        assert!(!sized_by_content(&style, &column()));
        let mut style = Style::default();
        style.translate = Some((0.0, 4.0));
        assert!(!sized_by_content(&style, &column()));
    }

    #[test]
    fn a_row_whose_width_is_not_the_stretch_of_its_column_is_not_cached() {
        let mut style = Style::default();
        style.width = Some(Dimension::Percent(50.0));
        assert!(!sized_by_content(&style, &column()));
        let mut centred = column();
        centred.align_items = Some(style::Align::Center);
        assert!(!sized_by_content(&Style::default(), &centred));
        let mut sideways = Style::default();
        sideways.row = Some(true);
        assert!(!sized_by_content(&Style::default(), &sideways));
    }
}
