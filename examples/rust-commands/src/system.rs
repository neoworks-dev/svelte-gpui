//! What the commands do: read files and `/proc`, which is Linux's view of the system.

use std::fs::{self, File};
use std::io::Read;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use svelte_gpui::CommandError;

/// The SHA-256 of a file, read in chunks so a large file never sits in memory.
pub fn hash_file(path: &str) -> Result<Value, CommandError> {
    let started = Instant::now();
    let mut file = File::open(path)
        .map_err(|error| CommandError::with_code(format!("{:?}", error.kind()), format!("{path}: {error}")))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    let mut size = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    let sha256: String = hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(json!({ "sha256": sha256, "size": size, "millis": started.elapsed().as_millis() as u64 }))
}

/// The processes using the most memory, from `/proc/<pid>/statm` and `comm`.
pub fn list_processes(limit: usize) -> Result<Vec<Value>, CommandError> {
    let page_kb = 4;
    let mut found: Vec<(u64, u64, String)> = Vec::new();
    for entry in fs::read_dir("/proc")?.flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|name| name.parse::<u64>().ok()) else {
            continue;
        };
        // A process can end between listing and reading; skip it.
        let (Ok(statm), Ok(name)) =
            (fs::read_to_string(entry.path().join("statm")), fs::read_to_string(entry.path().join("comm")))
        else {
            continue;
        };
        let resident: u64 = statm.split_whitespace().nth(1).and_then(|pages| pages.parse().ok()).unwrap_or(0);
        found.push((resident * page_kb, pid, name.trim().to_string()));
    }
    found.sort_by(|a, b| b.cmp(a));
    Ok(found
        .into_iter()
        .take(limit)
        .map(|(memory_kb, pid, name)| json!({ "pid": pid, "name": name, "memoryKb": memory_kb }))
        .collect())
}

/// The payload of a `tick` event: the time, the load averages and the memory in use.
pub fn tick() -> Value {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_millis() as u64).unwrap_or(0);
    let load: Vec<f64> = fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .take(3)
        .filter_map(|part| part.parse().ok())
        .collect();
    json!({ "time": time, "load": load, "memoryUsedMb": memory_used_megabytes() })
}

fn memory_used_megabytes() -> u64 {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let kilobytes = |key: &str| {
        let line = meminfo.lines().find(|line| line.starts_with(key));
        line.and_then(|line| line.split_whitespace().nth(1)).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0)
    };
    kilobytes("MemTotal:").saturating_sub(kilobytes("MemAvailable:")) / 1024
}
