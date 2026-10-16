#!/usr/bin/env bash
# Runs a bundle in a window on the focused monitor and saves a screenshot of that window.
#
#   scripts/shot.sh BUNDLE OUT.png [SECONDS] [host flags…]   e.g. --do press:Count
#
# HOST names another executable to run, such as an app's own binary or a packaged one.
# A BUNDLE of `-` passes no --bundle, and then no size either, for an app that has its own.
# The window opens where Hyprland puts new windows; the screenshot is of the window alone.
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

host_pid=""
cleanup() {
  kill $host_pid 2>/dev/null || true
}
trap cleanup EXIT

flags=(--bundle "$bundle" --width "${WIDTH:-1100}" --height "${HEIGHT:-760}")
if [ "$bundle" = "-" ]; then flags=(); fi
"$host" "${flags[@]}" "$@" \
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
# Where the window is now, as `x,y wxh`, read after the wait so the layout has settled.
geometry=$(hyprctl clients -j | python3 -c "import json,sys; c=next(c for c in json.load(sys.stdin) if c['address']=='$address'); print('%d,%d %dx%d' % (*c['at'], *c['size']))")
grim -g "$geometry" "$out"
echo "$out"
