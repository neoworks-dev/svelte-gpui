//! The window itself: what the app asks of it, what it reports back, and whether it may close.

use gpui::{Context, Window};

use super::Workbench;
use crate::window_control::{WindowCommand, WindowState};

impl Workbench {
    /// What the window is like now.
    fn window_state(window: &Window) -> WindowState {
        let size = window.viewport_size();
        WindowState {
            width: f32::from(size.width),
            height: f32::from(size.height),
            scale: window.scale_factor(),
            focused: window.is_window_active(),
            maximized: window.is_maximized(),
            fullscreen: window.is_fullscreen(),
        }
    }

    /// Tells the app about the window when anything it can read changed since the last time.
    pub(super) fn report_window(&mut self, window: &Window) {
        let state = Self::window_state(window);
        if self.reported_window == Some(state) {
            return;
        }
        self.reported_window = Some(state);
        self.svelte.window(&state);
    }

    /// Carries out what the app asked of the window since the last tick.
    pub(super) fn run_window_commands(&mut self, window: &mut Window) {
        let commands = std::mem::take(&mut *self.svelte.window_commands.borrow_mut());
        for command in commands {
            match command {
                WindowCommand::Title(title) => window.set_window_title(&title),
                WindowCommand::Minimize => window.minimize_window(),
                WindowCommand::Maximize => {
                    if !window.is_maximized() {
                        window.zoom_window();
                    }
                }
                WindowCommand::Unmaximize => {
                    if window.is_maximized() {
                        window.zoom_window();
                    }
                }
                WindowCommand::ToggleMaximize => window.zoom_window(),
                WindowCommand::SetFullscreen(on) => {
                    if on != window.is_fullscreen() {
                        window.toggle_fullscreen();
                    }
                }
                WindowCommand::ToggleFullscreen => window.toggle_fullscreen(),
                WindowCommand::Destroy => window.remove_window(),
            }
        }
    }

    /// Has the window ask the app before it closes, and report focus changes at once
    /// instead of at the next frame.
    pub(super) fn watch_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let this = cx.weak_entity();
        window.on_window_should_close(cx, move |_window, cx| {
            this.update(cx, |workbench, cx| workbench.close_requested(cx)).unwrap_or(true)
        });
        cx.observe_window_activation(window, |workbench, window, cx| workbench.tick(false, window, cx)).detach();
    }

    /// Whether the window may close now. The app's handlers decide; one that answers later
    /// keeps the window open, and the app closes it once the answer is yes.
    fn close_requested(&mut self, cx: &mut Context<Self>) -> bool {
        let allowed = self.svelte.close_requested();
        self.run_main_jobs(cx);
        self.flush_dirty(cx);
        cx.notify();
        allowed
    }
}
