#!/usr/bin/env bash
# Runs a bundle on an output of its own on the offscreen display and saves a screenshot of it.
# The offscreen display is a headless sway that draws to no screen, so nothing on the real
# monitors changes and nothing takes focus. It comes from ~/.local/bin/offscreen-display (a
# user service) and its helper offscreen-output; `offscreen-view` shows it in a VNC viewer.
#
#   scripts/shot.sh BUNDLE OUT.png [SECONDS] [host flags…]   e.g. --do press:Count
#
# It runs this checkout's host, built by scripts/host.sh. HOST names another executable to
# run instead, such as an app's own binary or a packaged one.
# A BUNDLE of `-` passes no --bundle, for an app that has its own size.
# The window fills its output, which is WIDTH x HEIGHT (default 1100x760), and the output is
# removed afterwards, so runs can go side by side. The host's log goes to
# /tmp/svelte-gpui-shot-<output>.log.
#
# A step that matched nothing or could not be read is listed after the screenshot's path, and
# the script then exits with 1. A host that quit by itself (`--exit-after-ms`) before the
# screenshot leaves none; the script then reports its summary and exits with its status.
set -euo pipefail

bundle=$1
out=$2
seconds=${3:-3}
shift 2
if [ $# -gt 0 ]; then shift; fi
root=$(cd "$(dirname "$0")/.." && pwd)
host=${HOST:-$root/target/release/svelte-gpui}
if [ ! -x "$host" ]; then
  echo "no host at $host; build this checkout's with scripts/host.sh" >&2
  exit 1
fi
export WAYLAND_DISPLAY=wayland-offscreen DISPLAY=:99
unset HYPRLAND_INSTANCE_SIGNATURE
width=${WIDTH:-1100}
height=${HEIGHT:-760}
output=$(offscreen-output create "${width}x${height}")
log=/tmp/svelte-gpui-shot-$output.log

host_pid=""
cleanup() {
  kill $host_pid 2>/dev/null || true
  offscreen-output remove "$output"
}
trap cleanup EXIT

flags=(--bundle "$bundle" --width "$width" --height "$height")
if [ "$bundle" = "-" ]; then flags=(); fi
"$host" "${flags[@]}" "$@" >"$log" 2>&1 &
host_pid=$!

if ! offscreen-output adopt "$output" "$host_pid"; then
  echo "no window appeared; see $log" >&2
  exit 1
fi
sleep "$seconds"
status=0
if kill -0 "$host_pid" 2>/dev/null; then
  grim -o "$output" "$out"
  echo "$out"
  failed=$(grep -E '^step [0-9]+ failed: ' "$log" || true)
  if [ -n "$failed" ]; then
    echo "steps that failed:" >&2
    echo "$failed" >&2
    status=1
  fi
else
  wait "$host_pid" || status=$?
  echo "the host quit by itself with status $status before the screenshot, so there is none; see $log" >&2
  grep -E '^steps: |^  step [0-9]+ ' "$log" >&2 || true
fi
exit "$status"
