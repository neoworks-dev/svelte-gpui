//! The link to Grove's main process: a unix socket carrying one JSON object per
//! line, in both directions. The app writes requests through `host.ipc`; what comes
//! back, replies and pushed events alike, is queued here by a reader thread and
//! handed to the app by the window's next tick.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Bridge {
    writer: Arc<Mutex<UnixStream>>,
    incoming: Arc<Mutex<VecDeque<String>>>,
    closed: Arc<AtomicBool>,
}

impl Bridge {
    /// Connects, retrying for a few seconds while the main process comes up.
    pub fn connect(path: &str) -> Option<Self> {
        let mut stream = None;
        for _ in 0..100 {
            match UnixStream::connect(path) {
                Ok(connected) => {
                    stream = Some(connected);
                    break;
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
            }
        }
        let stream = stream?;
        let reader = stream.try_clone().ok()?;
        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let closed = Arc::new(AtomicBool::new(false));
        let queue = incoming.clone();
        let flag = closed.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(reader).lines() {
                let Ok(line) = line else {
                    break;
                };
                queue.lock().expect("bridge queue").push_back(line);
            }
            flag.store(true, Ordering::Relaxed);
        });
        Some(Self {
            writer: Arc::new(Mutex::new(stream)),
            incoming,
            closed,
        })
    }

    /// Sends one line to the main process.
    pub fn send(&self, line: &str) {
        let mut writer = self.writer.lock().expect("bridge writer");
        if writer.write_all(line.as_bytes()).is_err() || writer.write_all(b"\n").is_err() {
            eprintln!("svelte-gpui: the main process is gone");
        }
    }

    /// Everything that has arrived since the last call.
    pub fn take(&self) -> Vec<String> {
        self.incoming.lock().expect("bridge queue").drain(..).collect()
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }
}
