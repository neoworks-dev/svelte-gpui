//! Which elements are the tops of cached views right now.
//!
//! A part of the document is drawn as a view of its own, and keeps what it drew until it is told
//! to draw again, so a change to an element has to tell the view it sits in. Whether a part is
//! worth a view depends on how many nodes it holds, which changes as the app adds and removes
//! content: a column that was too big for one view, and so was split into several, becomes one
//! view when it shrinks, and the views it was split into stop being views. A change inside one
//! of them has to find the view that is drawn now, so the list is made again each time the whole
//! document is built, and holds only the tops that build reached.

use std::collections::HashSet;

use crate::tree::NodeId;

#[derive(Default)]
pub struct ViewRoots {
    /// The tops the last whole build reached.
    current: HashSet<NodeId>,
    /// The tops the build in progress has reached so far.
    building: HashSet<NodeId>,
}

impl ViewRoots {
    /// Whether the element was the top of a view when the document was last built.
    pub fn contains(&self, id: &NodeId) -> bool {
        self.current.contains(id)
    }

    /// A build of the whole document starts. Nothing it does not reach stays a view.
    pub fn begin(&mut self) {
        self.building.clear();
    }

    /// The build drew the element as the top of a view.
    pub fn reach(&mut self, id: NodeId) {
        self.building.insert(id);
    }

    /// The build is done: what it reached are the views. Returns the tops that stopped being
    /// one, whose views are no longer drawn.
    pub fn finish(&mut self) -> Vec<NodeId> {
        let stopped = self.current.iter().filter(|id| !self.building.contains(id)).copied().collect();
        std::mem::swap(&mut self.current, &mut self.building);
        self.building.clear();
        stopped
    }

    /// Forgets every top, for a reload: the new app numbers its elements from the start again.
    pub fn clear(&mut self) {
        self.current.clear();
        self.building.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::ViewRoots;

    fn build(roots: &mut ViewRoots, reached: &[u32]) -> Vec<u32> {
        roots.begin();
        for id in reached {
            roots.reach(*id);
        }
        let mut stopped = roots.finish();
        stopped.sort();
        stopped
    }

    #[test]
    fn the_tops_a_build_reached_are_the_views() {
        let mut roots = ViewRoots::default();
        assert!(!roots.contains(&5));
        build(&mut roots, &[5, 9]);
        assert!(roots.contains(&5));
        assert!(roots.contains(&9));
        assert!(!roots.contains(&7));
    }

    #[test]
    fn a_top_the_next_build_does_not_reach_stops_being_one() {
        // A column split into the views 5 and 9 becomes one view, 2, once it shrinks.
        let mut roots = ViewRoots::default();
        build(&mut roots, &[5, 9]);
        assert_eq!(build(&mut roots, &[2]), vec![5, 9]);
        assert!(roots.contains(&2));
        assert!(!roots.contains(&5));
        assert!(!roots.contains(&9));
    }

    #[test]
    fn what_a_build_in_progress_has_reached_does_not_change_the_answer_yet() {
        let mut roots = ViewRoots::default();
        build(&mut roots, &[5]);
        roots.begin();
        roots.reach(9);
        assert!(roots.contains(&5));
        assert!(!roots.contains(&9));
        assert_eq!(roots.finish(), vec![5]);
        assert!(roots.contains(&9));
    }

    #[test]
    fn a_top_that_stays_one_is_not_reported() {
        let mut roots = ViewRoots::default();
        build(&mut roots, &[5, 9]);
        assert_eq!(build(&mut roots, &[5, 9, 11]), Vec::<u32>::new());
    }

    #[test]
    fn clearing_forgets_everything() {
        let mut roots = ViewRoots::default();
        build(&mut roots, &[5]);
        roots.clear();
        assert!(!roots.contains(&5));
        assert_eq!(build(&mut roots, &[]), Vec::<u32>::new());
    }
}
