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
# A window rule sends windows with the app id `svelte-gpui-shot` to it, silently.
# The host's log goes to /tmp/svelte-gpui-host.log.
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
app_id=svelte-gpui-shot

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
  >/tmp/svelte-gpui-host.log 2>&1 &
host_pid=$!

address=""
for _ in $(seq 100); do
  address=$(hyprctl clients -j | python3 -c "import json,sys; print(next((c['address'] for c in json.load(sys.stdin) if c['pid']==$host_pid),''))")
  if [ -n "$address" ]; then break; fi
  sleep 0.1
done
if [ -z "$address" ]; then
  echo "no window appeared; see /tmp/svelte-gpui-host.log" >&2
  exit 1
fi
sleep "$seconds"
grim -o "$output" "$out"
echo "$out"
