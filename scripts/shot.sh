#!/usr/bin/env bash
# Runs a bundle on a headless Hyprland output and saves a screenshot of it. The window opens
# there directly and never takes focus, so nothing on the real monitors changes.
#
#   scripts/shot.sh BUNDLE OUT.png [SECONDS] [host flags…]   e.g. --do press:Count
#
# HOST names another executable to run, such as an app's own binary or a packaged one.
# A BUNDLE of `-` passes no --bundle, and then no size either, for an app that has its own.
# The output (OUTPUT, by default `svelte-gpui`) is created when it does not exist:
#   hyprctl output create headless svelte-gpui
# A window rule sends windows with the app id `<output>-shot` to it, silently, so runs on
# different outputs can go side by side. The host's log goes to /tmp/<output>-host.log.
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
target=${CARGO_TARGET_DIR:-$HOME/Documents/neoworks/grove/.worktrees/gpui-spike/spike/plugin-views/target}
host=${HOST:-$target/release/svelte-gpui}
output=${OUTPUT:-svelte-gpui}
app_id=$output-shot
log=/tmp/$output-host.log

if ! hyprctl monitors -j | python3 -c "import json,sys; sys.exit(0 if any(m['name']=='$output' for m in json.load(sys.stdin)) else 1)"; then
  hyprctl output create headless "$output" >/dev/null
  sleep 0.5
fi
workspace=$(hyprctl monitors -j | python3 -c "import json,sys; print(next(m['activeWorkspace']['id'] for m in json.load(sys.stdin) if m['name']=='$output'))")
# Registered before the window exists, so it opens on the output instead of moving there.
hyprctl eval "hl.window_rule({ name = \"$app_id\", match = { class = \"^$app_id\$\" }, workspace = \"$workspace silent\", no_initial_focus = true, focus_on_activate = false })" >/dev/null

host_pid=""
cleanup() {
  kill $host_pid 2>/dev/null || true
}
trap cleanup EXIT

flags=(--bundle "$bundle" --width "${WIDTH:-1100}" --height "${HEIGHT:-760}")
if [ "$bundle" = "-" ]; then flags=(); fi
"$host" "${flags[@]}" --app-id "$app_id" "$@" \
  >"$log" 2>&1 &
host_pid=$!

address=""
for _ in $(seq 100); do
  address=$(hyprctl clients -j | python3 -c "import json,sys; print(next((c['address'] for c in json.load(sys.stdin) if c['pid']==$host_pid),''))")
  if [ -n "$address" ]; then break; fi
  sleep 0.1
done
if [ -z "$address" ]; then
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
