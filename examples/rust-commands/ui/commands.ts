// What the Rust side (src/main.rs) offers, written down once so `invoke` and `listen`
// check names, arguments and results.

export interface Process {
  pid: number;
  name: string;
  memoryKb: number;
}

export interface Tick {
  time: number;
  load: [number, number, number];
  memoryUsedMb: number;
}

declare module "@neoworks/svelte-gpui" {
  interface CommandMap {
    greet: { args: { name: string }; result: string };
    hash_file: { args: { path: string }; result: { sha256: string; size: number; millis: number } };
    list_processes: { args: { limit: number }; result: Process[] };
    reverse_bytes: { args: null; result: { length: number; checksum: number } };
    wait: { args: { millis: number }; result: string };
    fail: { args: null; result: never };
  }
  interface EventMap {
    tick: Tick;
  }
}
