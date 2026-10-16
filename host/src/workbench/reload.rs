//! Hot reload: replacing the running app with a new build of the bundle, in the same window.

use std::time::Instant;

use gpui::{Context, Window};

use super::Workbench;
use crate::runtime::{App as Svelte, Script};

impl Workbench {
    /// The bundle, or the build's report of why it has no new bundle, changed on disk. A failed
    /// build leaves the running app as it is and shows the error over it; the next good build
    /// removes the error and the new bundle starts.
    pub(crate) fn bundle_changed(
        &mut self,
        bundle: &str,
        allow_files: &[String],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match std::fs::read_to_string(format!("{bundle}.error")) {
            Ok(text) if !text.trim().is_empty() => self.show_build_error(text, cx),
            _ => {
                self.clear_build_error();
                self.reload(bundle, allow_files, window, cx);
            }
        }
    }

    /// Starts the bundle again and puts it in the place of the running app. The window keeps
    /// its size, position and focus; everything that belonged to the old app is dropped, and
    /// so is whatever it still had in flight, such as sockets and watches.
    ///
    /// A bundle that fails to run does not stop the host: the window shows the error until
    /// the next good build.
    pub(crate) fn reload(&mut self, bundle: &str, allow_files: &[String], window: &mut Window, cx: &mut Context<Self>) {
        let started = Instant::now();
        let commands = self.svelte.commands.clone();
        let next = Svelte::start_with(bundle, Script::Development(bundle.to_string()), allow_files, commands);
        let failed = next.load_error.is_some();
        self.replace_app(next, window, cx);
        println!(
            "reloaded in {:.0} ms, {} nodes{}",
            started.elapsed().as_secs_f64() * 1000.0,
            self.svelte.tree.borrow().nodes.len(),
            if failed { ", with an error" } else { "" }
        );
    }

    /// Swaps in a new app and forgets the old one's state.
    fn replace_app(&mut self, next: Svelte, window: &mut Window, cx: &mut Context<Self>) {
        self.release_canvases(window);
        self.generation += 1;
        // The old app, its QuickJS runtime and its element tree, are dropped here, and with
        // them the sockets and watches its `Io` owns.
        self.svelte = next;
        // The new app reads the fields through its own handle.
        self.interaction.fields = self.svelte.fields.clone();
        self.interaction.reset();
        self.interaction.images.reset();
        window.focus(&self.interaction.root_focus);
        self.forget_old_state();
        self.listen_for_io(cx);
        if let Some(title) = self.svelte.window_settings.borrow().as_ref().and_then(|settings| settings.title.clone()) {
            window.set_window_title(&title);
        }
        self.tick(false, window, cx);
        cx.notify();
    }

    /// Gives the window the images of every canvas the old app drew.
    fn release_canvases(&mut self, window: &mut Window) {
        let mut tree = self.svelte.tree.borrow_mut();
        let dropped = std::mem::take(&mut tree.dropped_canvases);
        for surface in dropped.iter().chain(tree.canvases.values()) {
            for image in surface.borrow_mut().release() {
                window.drop_image(image).ok();
            }
        }
    }

    /// Empties what the window kept about elements of the old tree, whose ids the new tree
    /// starts over on.
    fn forget_old_state(&mut self) {
        self.seen_version = 0;
        self.errors_seen = 0;
        self.errors_dismissed = false;
        self.focused = None;
        self.last_scroll.clear();
        self.sent_sizes.clear();
        self.announced_images.clear();
        self.queued.clear();
        self.poked = None;
        self.captured = None;
        // The new app has not heard what the window is like yet.
        self.reported_window = None;
    }

    /// Hands the app what its background calls produce, as it arrives, for as long as this
    /// run of the app lasts. A completion that arrives after a reload belongs to the old
    /// app and is dropped.
    pub(super) fn listen_for_io(&mut self, cx: &mut Context<Self>) {
        let Some(mut receiver) = self.svelte.io.take_receiver() else {
            return;
        };
        let generation = self.generation;
        cx.spawn(async move |this, cx| {
            use futures::StreamExt;
            while let Some(completion) = receiver.next().await {
                let delivered = this.update(cx, |workbench, cx| {
                    if workbench.generation != generation {
                        return false;
                    }
                    workbench.deliver(completion, cx);
                    true
                });
                if !matches!(delivered, Ok(true)) {
                    return;
                }
            }
        })
        .detach();
    }
}
