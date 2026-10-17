//! Background calls coming back to the app, and the ones that must run on the main thread.

use std::path::PathBuf;

use gpui::{Context, PathPromptOptions};
use serde_json::{Value, json};

use super::Workbench;
use crate::io::{Completion, Settle, Sink};
use crate::tree::NodeId;

impl Workbench {
    /// Hands a finished background call to the app, and draws what that changed.
    pub fn deliver(&mut self, completion: Completion, cx: &mut Context<Self>) {
        if completion.settle == Settle::Element {
            let payload = serde_json::from_str(&completion.json).unwrap_or_default();
            self.deliver_element_event(completion.call as NodeId, &payload, cx);
            return;
        }
        self.svelte.deliver(completion);
        self.run_main_jobs(cx);
        self.run_element_calls(cx);
        self.flush_dirty(cx);
        cx.notify();
    }

    /// Runs the requests that need GPUI, such as file dialogs, which the app queued
    /// through `host.io` since the last time.
    pub(super) fn run_main_jobs(&mut self, cx: &mut Context<Self>) {
        let jobs = std::mem::take(&mut *self.svelte.io.main_jobs.borrow_mut());
        let sink = self.svelte.io.sink();
        for job in jobs {
            match job.op.as_str() {
                "fs.pickFiles" => self.pick_files(job.call, &job.args, sink.clone(), cx),
                "fs.pickSavePath" => self.pick_save_path(job.call, &job.args, sink.clone(), cx),
                _ => sink.reject(job.call, format!("svelte-gpui: unknown operation {}", job.op)),
            }
        }
    }

    /// Opens the system's dialog for choosing files or directories. What the user chooses
    /// is allowed for the rest of the session, even outside the app's file limit, and the
    /// call settles with the chosen paths, or `null` if the user cancelled.
    fn pick_files(&mut self, call: u64, args: &Value, sink: Sink, cx: &mut Context<Self>) {
        let directories = args["directories"].as_bool().unwrap_or(false);
        let options = PathPromptOptions {
            files: !directories,
            directories,
            multiple: args["multiple"].as_bool().unwrap_or(false),
            prompt: args["prompt"].as_str().map(|prompt| prompt.to_string().into()),
        };
        let files = self.svelte.io.files.clone();
        let receiver = cx.prompt_for_paths(options);
        cx.spawn(async move |_, _| match receiver.await {
            Ok(Ok(Some(paths))) => {
                paths.iter().for_each(|path| files.grant_picked(path));
                let paths: Vec<_> = paths.iter().map(|path| path.to_string_lossy()).collect();
                sink.resolve(call, json!({ "paths": paths }));
            }
            Ok(Ok(None)) => sink.resolve(call, json!({ "paths": null })),
            Ok(Err(error)) => sink.reject_with_code(call, "Other", &format!("cannot open the dialog: {error}")),
            Err(_) => sink.reject_with_code(call, "Other", "the dialog went away"),
        })
        .detach();
    }

    /// Opens the system's dialog for choosing where to save. `directory` is where it
    /// starts, the home directory by default, and `name` is the suggested file name. The
    /// chosen path is allowed for the rest of the session.
    fn pick_save_path(&mut self, call: u64, args: &Value, sink: Sink, cx: &mut Context<Self>) {
        let files = self.svelte.io.files.clone();
        let start: PathBuf = match args["directory"].as_str() {
            Some(directory) => files.directories().expand(directory),
            None => files.directories().home.clone(),
        };
        let name = args["name"].as_str().map(str::to_string);
        let receiver = cx.prompt_for_new_path(&start, name.as_deref());
        cx.spawn(async move |_, _| match receiver.await {
            Ok(Ok(Some(path))) => {
                files.grant_picked(&path);
                sink.resolve(call, json!({ "path": path.to_string_lossy() }));
            }
            Ok(Ok(None)) => sink.resolve(call, json!({ "path": null })),
            Ok(Err(error)) => sink.reject_with_code(call, "Other", &format!("cannot open the dialog: {error}")),
            Err(_) => sink.reject_with_code(call, "Other", "the dialog went away"),
        })
        .detach();
    }
}
