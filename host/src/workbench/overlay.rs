//! Errors, while developing: an overlay over the app with what went wrong and where, and the
//! log file they go to instead when the window is not the place for them.

use std::io::Write;
use std::path::PathBuf;

use gpui::{AnyElement, Context, FontWeight, IntoElement, Rgba, Window, div, prelude::*, px, rgb, rgba};

use super::Workbench;
use crate::errors::{AppError, CodeLine, ErrorKind, Frame, Report};
use crate::style::{MONO, SANS};

/// Where the app's errors go.
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorOutput {
    /// An overlay over the app, until the user dismisses it or the next good reload clears it.
    Overlay,
    /// A file, as for an app that is installed and has no developer in front of it.
    Log(PathBuf),
}

const BACKDROP: u32 = 0x000000c8;
const PANEL: u32 = 0x1c1012;
const RED: u32 = 0xfca5a5;
const TEXT: u32 = 0xe4e4e7;
const DIM: u32 = 0x8b8b94;
const BUTTON: u32 = 0x3f3f46;

/// How large the error log may grow, in bytes, before it is started again.
const LOG_LIMIT: u64 = 1_000_000;

/// How many frames from libraries in a row are folded into one line.
const FOLDED_RUN: usize = 3;

impl Workbench {
    /// Chooses where errors go. The app's first errors, such as a bundle that did not run, are
    /// already waiting in its list and are handled on the next tick.
    pub fn send_errors_to(&mut self, output: ErrorOutput) {
        self.error_output = output;
    }

    /// Notices errors the app ran into since the last tick: they are written to the log, or the
    /// overlay is shown again for a new one.
    pub(super) fn report_errors(&mut self, cx: &mut Context<Self>) {
        let (total, unlogged) = {
            let mut list = self.svelte.errors.borrow_mut();
            (list.total, std::mem::take(&mut list.unlogged))
        };
        if let ErrorOutput::Log(path) = &self.error_output {
            append_to_log(path, &unlogged);
        }
        if total != self.errors_seen {
            self.errors_seen = total;
            self.errors_dismissed = false;
            cx.notify();
        }
    }

    /// Shows why the build failed. The app keeps running under it, as it was.
    pub(super) fn show_build_error(&mut self, text: String, cx: &mut Context<Self>) {
        eprintln!("svelte-gpui: the build failed:\n{text}");
        let report = Report { message: text.trim_end().to_string(), stack: String::new() };
        self.build_error = Some(AppError::explain(ErrorKind::Build, report, None, std::path::Path::new("")));
        self.errors_dismissed = false;
        cx.notify();
    }

    /// Forgets a build error, because a build since then worked.
    pub(super) fn clear_build_error(&mut self) {
        self.build_error = None;
    }

    /// Everything the overlay lists: the build's error first, then the app's.
    fn listed_errors(&self) -> Vec<AppError> {
        let mut errors: Vec<AppError> = self.build_error.iter().cloned().collect();
        errors.extend(self.svelte.errors.borrow().errors.iter().cloned());
        errors
    }

    /// Whether the overlay is on screen.
    pub(super) fn errors_visible(&self) -> bool {
        self.error_output == ErrorOutput::Overlay && !self.errors_dismissed && !self.listed_errors().is_empty()
    }

    /// Hides the overlay until another error happens.
    pub(super) fn dismiss_errors(&mut self, cx: &mut Context<Self>) {
        self.errors_dismissed = true;
        cx.notify();
    }

    /// The overlay, when there are errors to show.
    pub(super) fn error_overlay(&self, _window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.errors_visible() {
            return None;
        }
        let errors = self.listed_errors();
        let occurrences: u32 = errors.iter().map(|error| error.count).sum();
        let title = match (errors.len(), occurrences) {
            (1, 1) => "1 error".to_string(),
            (1, times) => format!("1 error, {times} times"),
            (count, _) => format!("{count} errors"),
        };
        let copied = errors.iter().map(AppError::text).collect::<Vec<_>>().join("\n");
        let header = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                div().flex_1().text_size(px(16.0)).font_weight(FontWeight::SEMIBOLD).text_color(rgb(RED)).child(title),
            )
            .child(button("copy-errors", "Copy").on_click(cx.listener(move |workbench, _, _, cx| {
                workbench.write_clipboard(copied.clone(), cx);
            })))
            .child(button("dismiss-errors", "Dismiss").on_click(cx.listener(|workbench, _, _, cx| {
                workbench.dismiss_errors(cx);
            })));
        let mut panel = div()
            .id("errors")
            .w_full()
            .max_w(px(920.0))
            .flex_shrink()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .p(px(18.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(rgb(0x7f1d1d))
            .bg(rgb(PANEL))
            .font_family(SANS)
            .text_size(px(13.0))
            .text_color(rgb(TEXT))
            .child(header);
        for error in &errors {
            panel = panel.child(error_card(error));
        }
        let hint = div()
            .flex_shrink_0()
            .text_size(px(12.0))
            .text_color(rgb(DIM))
            .child("Esc dismisses. A good build clears this by itself.");
        let backdrop = div().absolute().inset_0().occlude().p(px(24.0)).flex().flex_col().items_center().gap(px(10.0));
        Some(backdrop.bg(rgba(BACKDROP)).child(panel).child(hint).into_any_element())
    }
}

/// A small button in the overlay's own look.
fn button(id: &'static str, label: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .px(px(12.0))
        .py(px(5.0))
        .rounded(px(6.0))
        .bg(rgb(BUTTON))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x52525b)))
        .child(label)
}

/// One error: what kind, the message, where it came from, the code, and the stack.
fn error_card(error: &AppError) -> gpui::Div {
    let mut heading = div().flex().items_center().gap(px(8.0)).text_size(px(12.0)).text_color(rgb(DIM));
    heading = heading.child(error.kind.label());
    if error.count > 1 {
        heading = heading.child(
            div()
                .px(px(6.0))
                .rounded(px(8.0))
                .bg(rgb(0x7f1d1d))
                .text_color(rgb(RED))
                .child(format!("x{}", error.count)),
        );
    }
    // The build's report is a page of text; an app error is one line that stands out.
    let message = match error.kind {
        ErrorKind::Build => div().font_family(MONO).text_size(px(12.0)).child(error.message.clone()),
        _ => div().font_family(MONO).text_size(px(14.0)).font_weight(FontWeight::SEMIBOLD).child(error.message.clone()),
    };
    let mut card = div().flex().flex_col().gap(px(8.0)).child(heading).child(message);
    if let Some(component) = &error.component {
        card = card.child(div().text_color(rgb(0xc4b5fd)).child(format!("in {component}")));
    }
    if !error.code.is_empty() {
        card = card.child(code_block(&error.code));
    }
    if !error.frames.is_empty() {
        card = card.child(stack(&error.frames));
    }
    if error.kind == ErrorKind::Bundle && error.frames.is_empty() {
        card = card.child(
            div()
                .text_color(rgb(DIM))
                .child("The bundle is the file the build wrote; fix the build's error first if it printed one."),
        );
    }
    card
}

/// The source lines around where the error was thrown, with the throwing line marked.
fn code_block(lines: &[CodeLine]) -> gpui::Div {
    let mut block =
        div().flex().flex_col().p(px(8.0)).rounded(px(6.0)).bg(rgb(0x120a0c)).font_family(MONO).text_size(px(12.0));
    for line in lines {
        let colour: Rgba = if line.current { rgb(TEXT) } else { rgb(DIM) };
        let mut row = div().flex().gap(px(10.0)).text_color(colour);
        if line.current {
            row = row.bg(rgba(0x7f1d1d99));
        }
        let number = div().w(px(34.0)).flex_shrink_0().child(format!("{}", line.number));
        row = row.child(number).child(div().child(line.text.replace('\t', "  ")));
        block = block.child(row);
    }
    block
}

/// The stack, one frame to a line. The library's and Svelte's own frames are dimmed, and a long
/// run of them is folded into one line.
fn stack(frames: &[Frame]) -> gpui::Div {
    let mut list = div().flex().flex_col().gap(px(2.0)).font_family(MONO).text_size(px(12.0));
    let mut index = 0;
    while index < frames.len() {
        let run = frames[index..].iter().take_while(|frame| !frame.in_app).count();
        if run >= FOLDED_RUN {
            list = list.child(div().text_color(rgb(DIM)).child(format!("... {run} frames in Svelte and the runtime")));
            index += run;
            continue;
        }
        let frame = &frames[index];
        let colour = if frame.in_app { rgb(TEXT) } else { rgb(DIM) };
        list = list.child(
            div()
                .flex()
                .gap(px(10.0))
                .text_color(colour)
                .child(div().min_w(px(120.0)).child(frame.function.clone()))
                .child(div().child(frame.place.clone())),
        );
        index += 1;
    }
    list
}

/// Adds errors to the log file, creating its directory when needed. A log that cannot be
/// written is reported once on standard error and the errors are lost.
fn append_to_log(path: &PathBuf, errors: &[AppError]) {
    if errors.is_empty() {
        return;
    }
    // A log that grew past its limit is kept as `.old`, replacing the one before it.
    if std::fs::metadata(path).is_ok_and(|metadata| metadata.len() > LOG_LIMIT) {
        let _ = std::fs::rename(path, path.with_extension("log.old"));
    }
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|_| std::fs::OpenOptions::new().create(true).append(true).open(path))
        .and_then(|mut file| {
            let stamp =
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |time| time.as_secs());
            errors.iter().try_for_each(|error| write!(file, "[unix time {stamp}] {}\n", error.text()))
        });
    if let Err(problem) = written {
        eprintln!("svelte-gpui: cannot write the error log {}: {problem}", path.display());
    }
}
