//! The app's clock. It ticks on the display's frames while anything moves, so motion
//! keeps the display's refresh rate, and on timers otherwise; an idle window does not tick.

use std::time::Instant;

use gpui::{Context, Window};

use super::Workbench;
use crate::render::{self};
use crate::runtime::Wake;
use crate::tree::NodeId;

impl Workbench {
    /// Advances the app's clock, carries requests out, reports what the window learned,
    /// redraws if anything changed, and arranges the next tick. `frame` says whether this
    /// runs on a display frame, which is when animation frames run.
    pub(crate) fn tick(&mut self, frame: bool, window: &mut Window, cx: &mut Context<Self>) {
        let milliseconds = self.svelte.started.elapsed().as_secs_f64() * 1000.0;
        let wake = self.svelte.pump(milliseconds, frame);
        self.run_commands(window, cx);
        self.run_window_commands(window);
        self.report_errors(cx);
        self.run_main_jobs(cx);
        self.report_focus(window, cx);
        self.blink(cx);
        self.report_scroll(cx);
        self.report_sizes(cx);
        self.report_images(cx);
        self.report_window(window);
        let version = self.svelte.tree.borrow().version;
        if version != self.seen_version {
            self.seen_version = version;
            self.flush_dirty(cx);
            cx.notify();
        }
        if let Some(id) = self.poked {
            self.touch(id, cx);
        }
        let easing: Vec<NodeId> = self.interaction.animating_ids.borrow_mut().drain().collect();
        for id in easing {
            self.touch(id, cx);
        }
        let wanted = self.frame_target;
        if wanted > 0 {
            if self.frame_times.len() >= wanted {
                let mut gaps: Vec<f64> = self
                    .frame_times
                    .windows(2)
                    .map(|pair| pair[1].duration_since(pair[0]).as_secs_f64() * 1000.0)
                    .collect();
                gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mean = gaps.iter().sum::<f64>() / gaps.len().max(1) as f64;
                let p95 = gaps[(gaps.len() as f64 * 0.95) as usize % gaps.len().max(1)];
                println!(
                    "frames: {} at {mean:.1} ms mean, {p95:.1} ms 95th, {:.1} ms worst",
                    gaps.len(),
                    gaps.last().copied().unwrap_or(0.0)
                );
                self.frame_times.clear();
                self.frame_target = 0;
            }
        }
        if self.interaction.images.updated.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.dirty_all_views(cx);
            cx.notify();
        }
        let moved: Vec<NodeId> = self.interaction.bounds_changed.borrow_mut().drain().collect();
        for id in moved {
            self.touch(id, cx);
        }
        self.schedule(wake, window, cx);
    }

    /// Arranges the next tick: on the display's next frame while anything animates, so
    /// motion keeps the display's refresh rate, or when the first timer or caret blink is due.
    fn schedule(&mut self, wake: Wake, window: &mut Window, cx: &mut Context<Self>) {
        let animating =
            wake == Wake::Frame || self.poked.is_some() || !self.interaction.animating_ids.borrow().is_empty();
        if animating {
            self.tick_next_frame(window, cx);
            // A frame has to be drawn for the display to offer the next one.
            cx.notify();
        }
        let mut due = match wake {
            Wake::After(delay) => Some(Instant::now() + delay),
            _ => None,
        };
        let blinking = self.focused.filter(|id| self.interaction.fields.borrow().contains_key(id));
        if blinking.is_some() {
            let blink = render::next_blink(self.interaction.caret_epoch.get());
            due = Some(due.map_or(blink, |due| due.min(blink)));
        }
        if let Some(due) = due {
            self.tick_at(due, window, cx);
        }
    }

    /// Ticks when the display next draws, once however often it is asked.
    pub(super) fn tick_next_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.frame_tick_pending {
            return;
        }
        self.frame_tick_pending = true;
        let this = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            this.update(cx, |workbench, cx| {
                workbench.frame_tick_pending = false;
                workbench.tick(true, window, cx);
            })
            .ok();
        });
    }

    /// Ticks at a moment, unless a tick already comes sooner.
    fn tick_at(&mut self, due: Instant, window: &mut Window, cx: &mut Context<Self>) {
        if self.timer_due.is_some_and(|sooner| sooner <= due) {
            return;
        }
        self.timer_due = Some(due);
        cx.spawn_in(window, async move |this, cx| {
            let delay = due.saturating_duration_since(Instant::now());
            cx.background_executor().timer(delay).await;
            this.update_in(cx, |workbench, window, cx| {
                if workbench.timer_due == Some(due) {
                    workbench.timer_due = None;
                }
                workbench.tick(false, window, cx);
            })
            .ok();
        })
        .detach();
    }

    /// A field's text, caret or selection changed: the caret shows solid again, and the
    /// field is drawn again. Fields sit inside cached views, which keep drawing what they
    /// last drew until something says otherwise.
    pub fn caret_moved(&mut self, id: NodeId, cx: &mut Context<Self>) {
        self.interaction.caret_epoch.set(Instant::now());
        self.interaction.caret_on.set(true);
        self.touch(id, cx);
    }

    /// Blinks the caret of the focused field, by drawing it again when it should turn on or off.
    fn blink(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.focused else {
            return;
        };
        if !self.interaction.fields.borrow().contains_key(&id) {
            return;
        }
        let on = render::caret_visible(self.interaction.caret_epoch.get());
        if on != self.interaction.caret_on.get() {
            self.interaction.caret_on.set(on);
            self.touch(id, cx);
        }
    }
}
