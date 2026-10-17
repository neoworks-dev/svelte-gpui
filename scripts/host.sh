#!/usr/bin/env bash
# Builds the host into this checkout's own `target/`, so builds from two checkouts (the
# repo root and its worktrees) never share artifacts: cargo names the host's artifacts the
# same whatever directory it is built from, so a shared target directory lets one
# checkout's build stand in for another's.
#
#   scripts/host.sh          builds target/release/svelte-gpui
#   scripts/host.sh --seed   only prepares target/, for other cargo builds (packaging)
#
# The first time, target/ is seeded with a copy of the GPUI artifacts in SHARED (the main
# checkout's target directory, by default), so a worktree does not compile GPUI again. On
# btrfs the copy is a reflink: instant, and it takes no space until a file changes.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
target=$root/target
shared=${SHARED:-$HOME/Documents/neoworks/svelte-gpui/target}

if [ ! -d "$target/release/deps" ] && [ -d "$shared/release/deps" ]; then
  echo "seeding $target from $shared" >&2
  mkdir -p "$target"
  cp -r --reflink=auto "$shared/release" "$target/"
  # What the copy knows about the host itself names the sources of whichever checkout built
  # it last, which cargo would take as up to date. Without it, the host is built from these.
  release=$target/release
  rm -rf "$release"/.fingerprint/svelte_gpui_host-* "$release"/build/svelte_gpui_host-* \
    "$release"/deps/svelte_gpui-* "$release"/deps/libsvelte_gpui-* \
    "$release"/svelte-gpui "$release"/svelte-gpui.d "$release"/libsvelte_gpui.*
fi

if [ "${1:-}" = "--seed" ]; then
  exit 0
fi
CARGO_TARGET_DIR=$target cargo build --release --manifest-path "$root/host/Cargo.toml"
