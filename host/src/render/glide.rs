//! A mouse wheel's notches glide to where they point instead of jumping there.
//!
//! A `Glide` holds the distance a scroller still has to go. Each display frame asks it how far to
//! move now, so the motion keeps the display's refresh rate; a notch that arrives while one is
//! under way adds to what is left and starts the ease again.

use std::time::{Duration, Instant};

/// How long a notch takes to arrive.
pub const GLIDE: Duration = Duration::from_millis(150);

/// Where a notch has got to after `progress` of its time: fast at first, slowing into its target.
fn ease_out(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}

/// A distance being covered. It is signed in whatever direction the caller counts, and only the
/// caller knows the unit.
#[derive(Debug, Default)]
pub struct Glide {
    /// The distance this glide was started to cover, what was left and the notch that arrived.
    total: f32,
    /// How much of it has been handed out already.
    covered: f32,
    started: Option<Instant>,
}

impl Glide {
    pub fn is_active(&self) -> bool {
        self.started.is_some()
    }

    /// How far it still has to go.
    pub fn pending(&self) -> f32 {
        if self.is_active() { self.total - self.covered } else { 0.0 }
    }

    /// Adds `distance` to what is left, and eases over it from `now`.
    pub fn add(&mut self, distance: f32, now: Instant) {
        let remaining = self.pending() + distance;
        if remaining.abs() < 0.01 {
            self.cancel();
            return;
        }
        self.total = remaining;
        self.covered = 0.0;
        self.started = Some(now);
    }

    /// Forgets what is left: the scroller stays where it is.
    pub fn cancel(&mut self) {
        self.started = None;
        self.total = 0.0;
        self.covered = 0.0;
    }

    /// How far to move at `now`, counted from the last time it was asked.
    pub fn step(&mut self, now: Instant) -> f32 {
        let Some(started) = self.started else {
            return 0.0;
        };
        let progress = (now.saturating_duration_since(started).as_secs_f32() / GLIDE.as_secs_f32()).clamp(0.0, 1.0);
        let reached = self.total * ease_out(progress);
        let distance = reached - self.covered;
        self.covered = reached;
        if progress >= 1.0 {
            self.cancel();
        }
        distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn later(start: Instant, milliseconds: u64) -> Instant {
        start + Duration::from_millis(milliseconds)
    }

    #[test]
    fn a_notch_arrives_in_full_and_then_the_glide_is_over() {
        let start = Instant::now();
        let mut glide = Glide::default();
        glide.add(-78.0, start);
        let mut moved = 0.0;
        for milliseconds in [16, 32, 48, 64, 80, 96, 112, 128, 144, 160] {
            moved += glide.step(later(start, milliseconds));
        }
        assert!((moved + 78.0).abs() < 0.001, "moved {moved}");
        assert!(!glide.is_active());
        assert_eq!(glide.step(later(start, 200)), 0.0);
    }

    #[test]
    fn it_moves_the_most_at_first() {
        let start = Instant::now();
        let mut glide = Glide::default();
        glide.add(100.0, start);
        let first = glide.step(later(start, 16));
        let second = glide.step(later(start, 32));
        let third = glide.step(later(start, 48));
        assert!(first > second && second > third && third > 0.0);
    }

    #[test]
    fn a_notch_in_mid_glide_extends_the_target() {
        let start = Instant::now();
        let mut glide = Glide::default();
        glide.add(-78.0, start);
        let mut moved = glide.step(later(start, 50));
        glide.add(-78.0, later(start, 50));
        assert!(glide.pending() < -78.0 - 1.0, "what was left and a whole notch: {}", glide.pending());
        for milliseconds in (66..=250).step_by(16) {
            moved += glide.step(later(start, milliseconds));
        }
        assert!((moved + 156.0).abs() < 0.001, "two notches, moved {moved}");
    }

    #[test]
    fn a_notch_back_cancels_what_was_left() {
        let start = Instant::now();
        let mut glide = Glide::default();
        glide.add(50.0, start);
        glide.add(-50.0, start);
        assert!(!glide.is_active());
        assert_eq!(glide.pending(), 0.0);
    }

    #[test]
    fn cancelling_stops_where_it_is() {
        let start = Instant::now();
        let mut glide = Glide::default();
        glide.add(50.0, start);
        glide.step(later(start, 40));
        glide.cancel();
        assert_eq!(glide.step(later(start, 80)), 0.0);
        assert_eq!(glide.pending(), 0.0);
    }
}
