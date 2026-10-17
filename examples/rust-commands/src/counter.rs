//! A view drawn by GPUI itself, which the UI shows with `element("counter")`.
//!
//! It is what an app registers when something has to be drawn by Rust: it takes props (`label`, `step`),
//! emits an event when it is pressed, answers methods (`get`, `add`, `reset`), takes keyboard focus, handles
//! some keys and leaves the rest to bubble up to the UI's `onkey`, and takes typed text through its own input
//! method handler. Its count lives here, so it survives a hot reload of the UI.

use std::ops::Range;

use serde_json::{Value, json};
use svelte_gpui::gpui::{
    App, Bounds, Context, ElementInputHandler, EntityInputHandler, FocusHandle, FontWeight, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, Pixels, Point, Render, Subscription, UTF16Selection, Window, canvas, div, prelude::*,
    px, rgb,
};
use svelte_gpui::{AppElement, CommandError, NativeHost};

pub struct Counter {
    host: NativeHost,
    focus: FocusHandle,
    label: String,
    step: i64,
    count: i64,
    /// Text the input method typed, and the part of it still being composed, in bytes.
    typed: String,
    marked: Option<Range<usize>>,
    /// Redraw when focus moves, since the view is cached and would keep the border it last drew.
    focus_subscriptions: Vec<Subscription>,
}

impl Counter {
    pub fn new(host: NativeHost, cx: &mut Context<Self>) -> Self {
        Counter {
            host,
            focus: cx.focus_handle(),
            label: "Counter".to_string(),
            step: 1,
            count: 0,
            typed: String::new(),
            marked: None,
            focus_subscriptions: Vec::new(),
        }
    }

    /// A press, from the pointer or from Enter: counts up by the step and tells the UI.
    fn press(&mut self, cx: &mut Context<Self>) {
        self.count += self.step;
        self.host.emit("pressed", json!({ "count": self.count }));
        cx.notify();
    }
}

impl AppElement for Counter {
    fn set_prop(&mut self, name: &str, value: Value, _cx: &mut Context<Self>) {
        match name {
            "label" => self.label = value.as_str().unwrap_or("Counter").to_string(),
            "step" => self.step = value.as_i64().unwrap_or(1),
            _ => {}
        }
    }

    fn call(&mut self, method: &str, args: Value, _cx: &mut Context<Self>) -> Result<Value, CommandError> {
        match method {
            "get" => Ok(json!(self.count)),
            "add" => {
                let amount = args[0].as_i64().ok_or("add needs a number")?;
                self.count += amount;
                Ok(json!(self.count))
            }
            "reset" => Ok(json!(std::mem::take(&mut self.count))),
            _ => Err(CommandError::with_code("UnknownMethod", format!("a counter has no method {method:?}"))),
        }
    }

    fn focus_handle(&self, _cx: &App) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }
}

impl Render for Counter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_subscriptions.is_empty() {
            let moved = |_: &mut Counter, _: &mut Window, cx: &mut Context<Counter>| cx.notify();
            self.focus_subscriptions.push(cx.on_focus_in(&self.focus, window, moved));
            self.focus_subscriptions
                .push(cx.on_focus_out(&self.focus, window, move |this, _, window, cx| moved(this, window, cx)));
        }
        let border = if self.focus.is_focused(window) { rgb(0xf97316) } else { rgb(0x3a3a42) };
        let entity = cx.entity();
        let focus = self.focus.clone();
        div()
            .id("counter")
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap_1()
            .bg(rgb(0x1f1f24))
            .border_1()
            .border_color(border)
            .rounded_lg()
            .text_color(rgb(0xf4f4f5))
            .text_size(px(13.0))
            // Keys the view handles stop here; the others bubble to the UI's `onkey` and to `window`.
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                match event.keystroke.key.as_str() {
                    "enter" | "space" => this.press(cx),
                    "up" => this.count += this.step,
                    "down" => this.count -= this.step,
                    _ => return,
                }
                cx.notify();
                cx.stop_propagation();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus);
                    this.press(cx);
                }),
            )
            .child(div().text_color(rgb(0xa1a1aa)).child(self.label.clone()))
            .child(div().text_size(px(40.0)).font_weight(FontWeight::BOLD).child(self.count.to_string()))
            .child(div().text_color(rgb(0x6b6b74)).child(format!("step {}, typed \"{}\"", self.step, self.typed)))
            // The input method types into whatever is focused, through the handler registered here.
            .child(
                canvas(
                    |bounds, _, _| bounds,
                    move |bounds, _, window, cx| {
                        window.handle_input(&focus, ElementInputHandler::new(bounds, entity.clone()), cx);
                    },
                )
                .absolute()
                .size_full(),
            )
    }
}

/// Typing: text and the part of it being composed, counted in UTF-16 units as input methods count.
impl EntityInputHandler for Counter {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let (start, end) = (byte_offset(&self.typed, range.start), byte_offset(&self.typed, range.end));
        *adjusted = Some(utf16_offset(&self.typed, start)..utf16_offset(&self.typed, end));
        Some(self.typed[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let end = utf16_offset(&self.typed, self.typed.len());
        Some(UTF16Selection { range: end..end, reversed: false })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        let marked = self.marked.clone()?;
        Some(utf16_offset(&self.typed, marked.start)..utf16_offset(&self.typed, marked.end))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked = None;
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace(range, text, false);
        self.host.emit("typed", json!({ "text": self.typed }));
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace(range, new_text, true);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _range: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        Some(element_bounds)
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

impl Counter {
    /// Puts `text` where the input method says (the part being composed by default, else the end).
    fn replace(&mut self, range: Option<Range<usize>>, text: &str, mark: bool) {
        let bytes = match range {
            Some(range) => byte_offset(&self.typed, range.start)..byte_offset(&self.typed, range.end),
            None => self.marked.clone().unwrap_or(self.typed.len()..self.typed.len()),
        };
        self.typed.replace_range(bytes.clone(), text);
        self.marked = mark.then(|| bytes.start..bytes.start + text.len());
    }
}

/// The byte offset of a UTF-16 offset in `text`.
fn byte_offset(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (index, character) in text.char_indices() {
        if units >= utf16 {
            return index;
        }
        units += character.len_utf16();
    }
    text.len()
}

/// The UTF-16 offset of a byte offset in `text`.
fn utf16_offset(text: &str, byte: usize) -> usize {
    text[..byte].encode_utf16().count()
}
