//! Watching the bundle for hot reload: when a build writes it, the window starts it again
//! (see `Workbench::reload`).

use std::path::{Path, PathBuf};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use gpui::{App, WindowHandle};
use notify::event::EventKind;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::workbench::Workbench;

/// How long the bundle must stay untouched before it is loaded. An editor or a build tool
/// can write a file in several steps, and a script half written does not run.
const QUIET: Duration = Duration::from_millis(40);

/// A watch on the directory that holds the bundle. A build that replaces the file by
/// renaming another over it is seen too, which watching the file itself would miss.
struct BundleWatch {
    _watcher: RecommendedWatcher,
    /// One message per settled change.
    changes: UnboundedReceiver<()>,
}

impl BundleWatch {
    fn start(bundle: &Path) -> notify::Result<Self> {
        let directory = bundle.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let name = bundle.file_name().map(PathBuf::from).unwrap_or_default();
        let mut error_name = name.clone().into_os_string();
        error_name.push(".error");
        let names = [name, PathBuf::from(error_name)];
        let (raw_sender, raw) = channel();
        let mut watcher = notify::recommended_watcher(raw_sender)?;
        watcher.watch(directory, RecursiveMode::NonRecursive)?;
        let (settled_sender, changes) = unbounded();
        let bundle = bundle.to_path_buf();
        std::thread::Builder::new().name("bundle-watch".to_string()).spawn(move || {
            loop {
                // Wait for a change to the bundle, then for the writing to stop.
                match raw.recv() {
                    Ok(Ok(event)) if touches(&event, &names) => {}
                    Ok(_) => continue,
                    Err(_) => return,
                }
                loop {
                    match raw.recv_timeout(QUIET) {
                        Ok(_) => continue,
                        Err(RecvTimeoutError::Timeout) => break,
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
                // A bundle that is gone is being replaced; the next event brings it back. A
                // failed build leaves the old bundle and says why beside it.
                let failed = Path::new(&format!("{}.error", bundle.display())).exists();
                if (bundle.exists() || failed) && settled_sender.unbounded_send(()).is_err() {
                    return;
                }
            }
        })?;
        Ok(BundleWatch { _watcher: watcher, changes })
    }
}

/// Whether a notification is about the bundle or the file the build leaves its errors in (a
/// build removes that one when it succeeds), as opposed to something else in the directory,
/// such as the compiled copy beside the bundle.
fn touches(event: &notify::Event, names: &[PathBuf]) -> bool {
    let changed = matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) | EventKind::Any | EventKind::Other
    );
    changed && event.paths.iter().any(|path| names.iter().any(|name| path.file_name() == Some(name.as_os_str())))
}

/// Starts the bundle again in `window` each time a build writes it, until the window is gone.
pub fn reload_on_change(window: WindowHandle<Workbench>, bundle: PathBuf, allow_files: Vec<String>, cx: &mut App) {
    let watch = match BundleWatch::start(&bundle) {
        Ok(watch) => watch,
        Err(error) => {
            eprintln!("svelte-gpui: cannot watch {}: {error}", bundle.display());
            return;
        }
    };
    println!("watching {} for changes", bundle.display());
    // Fields are captured one by one, so the watcher has to be named to stay alive with the task.
    let BundleWatch { _watcher: watcher, mut changes } = watch;
    cx.spawn(async move |cx| {
        let _watching = watcher;
        let path = bundle.to_string_lossy().to_string();
        while changes.next().await.is_some() {
            let reloaded = cx.update(|cx| {
                window.update(cx, |workbench, window, cx| workbench.bundle_changed(&path, &allow_files, window, cx))
            });
            if !matches!(reloaded, Ok(Ok(()))) {
                return;
            }
        }
    })
    .detach();
}
