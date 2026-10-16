#!/usr/bin/env bash
# Shows on an issue that its change works: a comment, as neoworks-bot, with what changed
# and the screenshots that prove it.
#
#   scripts/evidence.sh ISSUE BODY [SCREENSHOT…]
#   scripts/evidence.sh 32 /tmp/body.md /tmp/before.png /tmp/after.png
#
# BODY is a file, or the text itself. Each screenshot is committed to the repo's
# `evidence` branch under a timestamp, so two runs never overwrite each other, and
# shown in the comment from there. The comment ends with the commit it was verified on.
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "usage: scripts/evidence.sh ISSUE BODY [SCREENSHOT…]" >&2
  exit 2
fi
issue=$1
body=$2
shift 2
repo=neoworks-dev/svelte-gpui
branch=evidence

if [ -f "$body" ]; then body=$(cat "$body"); fi
comment="$body"

for shot in "$@"; do
  if [ ! -f "$shot" ]; then
    echo "no such screenshot: $shot" >&2
    exit 1
  fi
  name="$(date +%Y-%m-%d-%H%M%S)-$issue-$(basename "$shot")"
  # The request goes in on stdin: a full-window screenshot as an argument is past
  # Linux's 128 KiB limit on a single argument.
  python3 -c '
import base64, json, sys
print(json.dumps({"message": f"Screenshot for #{sys.argv[1]}", "branch": sys.argv[2],
                  "content": base64.b64encode(open(sys.argv[3], "rb").read()).decode()}))
' "$issue" "$branch" "$shot" | gh bot api --method PUT "repos/$repo/contents/shots/$name" --input - >/dev/null
  comment+=$'\n\n'"![$(basename "$shot" .png)](https://raw.githubusercontent.com/$repo/$branch/shots/$name)"
done

commit=$(git rev-parse --short HEAD)
where=$(git branch --show-current)
comment+=$'\n\n---\n\n'"*Verified on \`$commit\` (\`$where\`).*"
gh bot issue comment "$issue" --repo "$repo" --body "$comment"
