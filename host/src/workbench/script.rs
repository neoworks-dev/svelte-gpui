//! Scripted steps (`--do press:Save`), for testing where nobody can click.

use gpui::{Context, Keystroke, Window};

use super::Workbench;
use crate::render::{self};
use crate::tree::{Command, Kind, NodeId, ROOT};

/// A step taken apart: `ACTION:TARGET:ARGUMENT`.
#[derive(Debug, PartialEq)]
struct Parts {
    action: String,
    target: String,
    argument: String,
}

/// Steps whose argument is never free text, so it is the part after the last colon and the target
/// may contain colons as they are.
const LAST_COLON: [&str; 5] = ["key", "scroll", "down", "move", "up"];

/// Splits a step at its colons. `\:` is a colon that does not split, and `\\` is a backslash. Of the
/// steps that take an argument, `key`, `scroll`, `down`, `move` and `up` end in it, so a target
/// such as `echo #1: hello` needs no escape there; for the others (`type`, `compose`, `commit`,
/// `drop`, `dragover`) the target ends at its first colon and the rest is the argument, which
/// may hold colons.
fn split_step(step: &str) -> Parts {
    let mut fields = vec![String::new()];
    let mut characters = step.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\\' if matches!(characters.peek(), Some(':' | '\\')) => {
                fields.last_mut().unwrap().push(characters.next().unwrap());
            }
            ':' => fields.push(String::new()),
            other => fields.last_mut().unwrap().push(other),
        }
    }
    let mut fields = fields.into_iter();
    let action = fields.next().unwrap_or_default();
    let rest: Vec<String> = fields.collect();
    if LAST_COLON.contains(&action.as_str()) && rest.len() >= 2 {
        let (argument, target) = rest.split_last().unwrap();
        return Parts { action, target: target.join(":"), argument: argument.clone() };
    }
    let target = rest.first().cloned().unwrap_or_default();
    let argument = rest.iter().skip(1).cloned().collect::<Vec<_>>().join(":");
    Parts { action, target, argument }
}

/// What the scripted steps did: how many ran, and which of them could not be carried out.
#[derive(Default)]
pub(crate) struct StepLog {
    ran: usize,
    /// `(number, step, why)` for each step that matched nothing or could not be read.
    failures: Vec<(usize, String, String)>,
}

impl StepLog {
    /// Counts a step that is about to run, and gives its number, from 1.
    fn begin(&mut self) -> usize {
        self.ran += 1;
        self.ran
    }

    /// Whether the run went wrong: a step failed, or the run ended before all of them had run.
    pub(crate) fn failed(&self, total: usize) -> bool {
        !self.failures.is_empty() || self.ran < total
    }

    /// What the summary prints about the steps, nothing when there were none.
    pub(crate) fn summary(&self, total: usize) -> Vec<String> {
        if total == 0 {
            return Vec::new();
        }
        let mut line = format!("steps: {} of {total} ran, {} failed", self.ran, self.failures.len());
        if self.ran < total {
            line += &format!(", {} never ran because the run ended first", total - self.ran);
        }
        let mut lines = vec![line];
        for (number, step, why) in &self.failures {
            lines.push(format!("  step {number} {step:?}: {why}"));
        }
        lines
    }
}

/// The two or three numbers of a pointer step, `x,y` and optionally the button. Nothing at all is `0,0`.
fn pointer_argument(argument: &str) -> Result<(f32, f32, i32), String> {
    if argument.is_empty() {
        return Ok((0.0, 0.0, 0));
    }
    let numbers: Vec<&str> = argument.split(',').map(str::trim).collect();
    let number =
        |text: &str| text.parse::<f32>().map_err(|_| format!("cannot read {text:?} as a number in {argument:?}"));
    match numbers.as_slice() {
        [x, y] => Ok((number(x)?, number(y)?, 0)),
        [x, y, button] => Ok((number(x)?, number(y)?, number(button)? as i32)),
        _ => Err(format!("expected x,y or x,y,button, not {argument:?}")),
    }
}

impl Workbench {
    /// What the scripted steps did so far, for the summary.
    pub(crate) fn step_summary(&self, total: usize) -> Vec<String> {
        self.steps.summary(total)
    }

    /// Whether a scripted step failed, or the run ended before they had all run.
    pub(crate) fn steps_failed(&self, total: usize) -> bool {
        self.steps.failed(total)
    }

    /// One scripted step: a click, a focus, or a scroll of the nearest scrollable ancestor. A step that
    /// matches nothing or cannot be read is written down and printed as it happens, and counted in the
    /// summary at the end of a run.
    pub(crate) fn run_step(&mut self, step: &str, window: &mut Window, cx: &mut Context<Self>) {
        let number = self.steps.begin();
        if let Err(why) = self.carry_out_step(step, window, cx) {
            eprintln!("step {number} failed: {step}: {why}");
            self.steps.failures.push((number, step.to_string(), why));
        }
    }

    fn carry_out_step(&mut self, step: &str, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        let Parts { action, target: needle, argument } = split_step(step);
        let (action, needle, argument) = (action.as_str(), needle.as_str(), argument.as_str());
        if action == "wait" {
            return Ok(());
        }
        if action == "dismiss" {
            self.dismiss_errors(cx);
            return Ok(());
        }
        if action == "frames" {
            let frames = needle.parse().map_err(|_| format!("cannot read {needle:?} as a number of frames"))?;
            self.frame_times.clear();
            self.frame_target = frames;
            cx.notify();
            return Ok(());
        }
        if matches!(action, "press" | "click" | "hover" | "unhover" | "focus" | "poke") && !argument.is_empty() {
            return Err(format!(
                "{action} takes no argument, but got {argument:?}; a colon in the target is written \\:"
            ));
        }
        let Some(id) = self.find_node(needle) else {
            return Err(format!("nothing matches {needle:?}"));
        };
        if action == "hover" || action == "unhover" {
            let target = self.listening_ancestor(id, "hover");
            self.hover_changed(target, action == "hover", cx);
            return Ok(());
        }
        if action == "poke" {
            self.poked = Some(id);
            return Ok(());
        }
        if action == "drop" || action == "dragover" {
            self.simulate_drag(id, argument, action == "drop", cx);
            return Ok(());
        }
        if matches!(action, "down" | "move" | "up") {
            let (_, _, button) = pointer_argument(argument)?;
            let id = if action == "down" {
                // A press goes to the nearest element that takes this button, as a real one does.
                self.pressed_ancestor(id, button)
            } else {
                self.listening_ancestor(id, action)
            };
            self.run_pointer_step(action, id, if argument.is_empty() { "0,0" } else { argument }, cx);
            self.dirty_all_views(cx);
            cx.notify();
            return Ok(());
        }
        match action {
            "click" | "press" => {
                let target = self.listening_ancestor(id, "press");
                // A real click goes down first, which overlays opened elsewhere hear as an outside press.
                let middle = self.interaction.bounds.borrow().get(&target).map(|bounds| bounds.center());
                if let Some(middle) = middle {
                    self.simulate_press_outside(middle, 0, cx);
                }
                self.emit(target, "press", "{\"x\":0,\"y\":0,\"button\":0,\"clicks\":1}".to_string(), cx);
            }
            "focus" => {
                let command = Command::Focus(self.focusable_ancestor(id));
                if !self.carry_out(&command, window, cx) {
                    self.queued.push((command, 20));
                }
            }
            "type" => {
                let command = Command::Focus(id);
                self.carry_out(&command, window, cx);
                for character in argument.chars() {
                    let text = character.to_string();
                    let keystroke =
                        Keystroke { modifiers: Default::default(), key: text.clone(), key_char: Some(text) };
                    self.handle_key(id, &keystroke, false, true, cx);
                }
            }
            "compose" | "commit" => {
                let command = Command::Focus(id);
                self.carry_out(&command, window, cx);
                let text = argument;
                if action == "compose" {
                    <Self as gpui::EntityInputHandler>::replace_and_mark_text_in_range(
                        self, None, text, None, window, cx,
                    );
                } else {
                    <Self as gpui::EntityInputHandler>::replace_text_in_range(self, None, text, window, cx);
                }
            }
            "key" => {
                let keystroke =
                    Keystroke::parse(argument).map_err(|_| format!("cannot read {argument:?} as a keystroke"))?;
                self.handle_key(id, &keystroke, false, true, cx);
            }
            "scroll" => {
                let pixels: f32 =
                    argument.parse().map_err(|_| format!("cannot read {argument:?} as a number of pixels"))?;
                let target = self.scrollable_ancestor(id);
                self.queued.push((Command::ScrollTo(target, 0.0, pixels), 20));
            }
            _ => return Err(format!("unknown step {action:?}")),
        }
        self.dirty_all_views(cx);
        cx.notify();
        Ok(())
    }

    /// `down` presses on an element, which then holds the pointer; `move` and `up` go to the
    /// element holding it. The argument is the position in the window as `x,y`, and optionally the button after it as `x,y,2`.
    fn run_pointer_step(&mut self, action: &str, id: NodeId, at: &str, cx: &mut Context<Self>) {
        let mut numbers = at.split(',').map(|number| number.trim().parse::<f32>().unwrap_or(0.0));
        let (x, y) = (numbers.next().unwrap_or(0.0), numbers.next().unwrap_or(0.0));
        // A third number names the button: 0 left (the default), 1 middle, 2 right.
        let button = numbers.next().unwrap_or(0.0) as i32;
        let position = gpui::point(gpui::px(x), gpui::px(y));
        let target = self.captured.unwrap_or(id);
        let detail = render::pointer_detail(target, position, Default::default(), button, 1, self);
        match action {
            "down" => {
                self.simulate_press_outside(position, button, cx);
                self.captured = Some(id);
                self.emit(id, "down", detail, cx);
            }
            "move" => {
                self.emit(target, "move", detail, cx);
            }
            _ => {
                self.captured = None;
                self.emit(target, "up", detail, cx);
            }
        }
    }

    /// The closest element at or above `id` that listens for `event`, as a real click would find.
    fn listening_ancestor(&self, id: NodeId, event: &str) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            let node = tree.nodes.get(&candidate);
            if node.is_some_and(|node| node.listeners.contains(event)) {
                return candidate;
            }
            current = node.and_then(|node| node.parent);
        }
        id
    }

    /// The closest element at or above `id` that listens for `down` and takes this button.
    fn pressed_ancestor(&self, id: NodeId, button: i32) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            let node = tree.nodes.get(&candidate);
            if node.is_some_and(|node| node.listeners.contains("down") && render::takes_button(node, button)) {
                return candidate;
            }
            current = node.and_then(|node| node.parent);
        }
        id
    }

    /// The closest element at or above `id` that can take focus, as pressing it would give focus to.
    fn focusable_ancestor(&self, id: NodeId) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let handles = self.interaction.focus.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            if handles.contains_key(&candidate) {
                return candidate;
            }
            current = tree.nodes.get(&candidate).and_then(|node| node.parent);
        }
        id
    }

    /// The closest element at or above `id` that scrolls.
    fn scrollable_ancestor(&self, id: NodeId) -> NodeId {
        let tree = self.svelte.tree.borrow();
        let handles = self.interaction.scroll.borrow();
        let mut current = Some(id);
        while let Some(candidate) = current {
            if handles.contains_key(&candidate) {
                return candidate;
            }
            current = tree.nodes.get(&candidate).and_then(|node| node.parent);
        }
        id
    }

    /// An element by its text, or by `@attribute=value` such as `@placeholder=Your name`.
    fn find_node(&self, needle: &str) -> Option<NodeId> {
        let Some(query) = needle.strip_prefix('@') else {
            return self.find_by_text(needle);
        };
        let (name, value) = query.split_once('=')?;
        let tree = self.svelte.tree.borrow();
        let mut matches: Vec<NodeId> = tree
            .nodes
            .iter()
            .filter(|(_, node)| match (&node.kind, name) {
                (Kind::Element { tag }, "tag") => tag == value,
                _ => node.attributes.get(name).map(String::as_str) == Some(value),
            })
            .map(|(id, _)| *id)
            .collect();
        matches.sort();
        matches.first().copied()
    }

    /// The deepest element whose text contains `needle`, for scripted clicks.
    fn find_by_text(&self, needle: &str) -> Option<NodeId> {
        let tree = self.svelte.tree.borrow();
        let mut current = ROOT;
        if !tree.text_content(current).contains(needle) {
            return None;
        }
        loop {
            let node = tree.nodes.get(&current)?;
            let next = node.children.iter().find(|child| {
                matches!(tree.nodes.get(child).map(|n| &n.kind), Some(Kind::Element { .. }))
                    && tree.text_content(**child).contains(needle)
            });
            match next {
                Some(child) => current = *child,
                None => return Some(current),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(step: &str) -> (String, String, String) {
        let Parts { action, target, argument } = split_step(step);
        (action, target, argument)
    }

    fn parts(action: &str, target: &str, argument: &str) -> (String, String, String) {
        (action.to_string(), target.to_string(), argument.to_string())
    }

    #[test]
    fn plain_steps_split_as_before() {
        assert_eq!(split("press:Save"), parts("press", "Save", ""));
        assert_eq!(split("wait"), parts("wait", "", ""));
        assert_eq!(split("type:@placeholder=Name:Ada"), parts("type", "@placeholder=Name", "Ada"));
        assert_eq!(split("type:Name:a:b"), parts("type", "Name", "a:b"));
        assert_eq!(split("down:Canvas:10,20,2"), parts("down", "Canvas", "10,20,2"));
        assert_eq!(split("frames:30"), parts("frames", "30", ""));
    }

    #[test]
    fn a_target_with_a_colon_splits_from_the_end_for_steps_that_end_in_their_argument() {
        assert_eq!(split("key:echo #1: hello:down"), parts("key", "echo #1: hello", "down"));
        assert_eq!(split("scroll:Time 12:30:400"), parts("scroll", "Time 12:30", "400"));
        assert_eq!(split("down:12:30:5,6"), parts("down", "12:30", "5,6"));
    }

    #[test]
    fn an_escaped_colon_does_not_split() {
        assert_eq!(split(r"press:echo #1\: hello"), parts("press", "echo #1: hello", ""));
        assert_eq!(split(r"type:Time 12\:30:ok"), parts("type", "Time 12:30", "ok"));
        assert_eq!(split(r"key:a\:b:enter"), parts("key", "a:b", "enter"));
        assert_eq!(split(r"press:@href=http\://x"), parts("press", "@href=http://x", ""));
    }

    #[test]
    fn a_backslash_before_a_backslash_or_colon_is_one_backslash() {
        assert_eq!(split(r"press:a\\b"), parts("press", r"a\b", ""));
        assert_eq!(split(r"press:a\\:b"), parts("press", r"a\", "b"));
        assert_eq!(split(r"press:C:\dir"), parts("press", "C", r"\dir"));
    }

    #[test]
    fn a_pointer_argument_is_two_or_three_numbers() {
        assert_eq!(pointer_argument(""), Ok((0.0, 0.0, 0)));
        assert_eq!(pointer_argument("10,20"), Ok((10.0, 20.0, 0)));
        assert_eq!(pointer_argument(" 10 , 20 , 2 "), Ok((10.0, 20.0, 2)));
    }

    #[test]
    fn a_pointer_argument_that_cannot_be_read_is_an_error() {
        assert!(pointer_argument("10").is_err());
        assert!(pointer_argument("ten,20").is_err());
        assert!(pointer_argument("1,2,3,4").is_err());
        assert!(pointer_argument("1,2,right").is_err());
    }

    #[test]
    fn a_run_without_steps_has_nothing_to_report() {
        let log = StepLog::default();
        assert!(log.summary(0).is_empty());
        assert!(!log.failed(0));
    }

    #[test]
    fn a_failed_step_is_listed_and_fails_the_run() {
        let mut log = StepLog::default();
        assert_eq!(log.begin(), 1);
        assert_eq!(log.begin(), 2);
        log.failures.push((2, "press:Save".to_string(), "nothing matches \"Save\"".to_string()));
        assert!(log.failed(2));
        assert_eq!(
            log.summary(2),
            vec![
                "steps: 2 of 2 ran, 1 failed".to_string(),
                "  step 2 \"press:Save\": nothing matches \"Save\"".to_string()
            ]
        );
    }

    #[test]
    fn steps_the_run_ended_before_reaching_fail_it() {
        let mut log = StepLog::default();
        log.begin();
        assert!(log.failed(3));
        assert_eq!(log.summary(3), vec!["steps: 1 of 3 ran, 0 failed, 2 never ran because the run ended first"]);
    }

    #[test]
    fn steps_that_all_ran_do_not_fail_it() {
        let mut log = StepLog::default();
        log.begin();
        log.begin();
        assert!(!log.failed(2));
        assert_eq!(log.summary(2), vec!["steps: 2 of 2 ran, 0 failed"]);
    }
}
