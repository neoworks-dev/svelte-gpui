# svelte-gpui — working notes

Native apps from Svelte 5 components, run in QuickJS and drawn by a Rust GPUI host.
See README.md. svelte-gpui is its own project; Grove is one app that may use it.

- Apps are built from primitives (`runtime/ui/*.svelte`), each rendering one host
  tag (`g-view`, `g-text`, …) that `host/src/render/` draws. There is no HTML and
  no CSS: a tag the host does not know is not drawn. Host tags are prefixed because
  Svelte puts `<text>` and `<image>` in the SVG namespace.
- Styles are typed objects (`runtime/ui/style.ts`) sent as JSON in the `look`
  attribute; `host/src/style/parse.rs` reads the same property names. Keep the two in step.
- Operation codes live only in `protocol.json`; `host/build.rs` generates the Rust
  constants and `runtime/host/bridge.ts` imports the file.
- Host event names (`press`, `edit`, `key`, `down`, …) avoid the ones Svelte
  delegates (`click`, `input`, `keydown`, …), so handlers sit on their own element.
  Components pass handlers through an attribute spread so an absent one adds no
  listener: a listener is what gives an element its GPUI handler.
- `firstChild` and `nextSibling` must be prototype accessors in `nodes.ts`:
  Svelte's `init_operations()` pulls the getters off `Node.prototype` once.
- The host build reuses the GPUI artifacts of the grove `gpui-spike` worktree via
  `CARGO_TARGET_DIR`; `host/vendor/xattr` patches a `libc` incompatibility.
- Test with `scripts/shot.sh`, which opens the window on a headless Hyprland output
  without focus. Never open a test window any other way: it takes the user's focus.
  Scripted steps run 0.9 s apart, so wait long enough before the
  screenshot.
- The app's clock ticks on display frames while anything animates and on timers
  otherwise (`host/src/workbench/clock.rs`). Never pace it with a fixed interval: that
  caps motion below the display's refresh rate.
- The host's Rust is not formatted crate-wide; format only files you touch, with
  `rustfmt --edition 2024 --config max_width=120,use_small_heuristics=Max,skip_children=true`.
- `bun run typecheck` runs tsc and svelte-check.
- Network and files go through one path: `host.io(op, call, args, bytes)` from
  `runtime/io/channel.ts` to `host/src/io/`, where each area (`fetch.rs`,
  `websocket.rs`, `files.rs`) runs work on threads and answers through a `Sink`.
  Results wake the window at once (`Workbench::deliver`); never poll for them. Work
  that needs GPUI (file dialogs) is a `MainJob`, run in `workbench/io.rs`.
- User docs live in `docs/`, and the agent skill in `.claude/skills/svelte-gpui/`.
  When a primitive, style property, event or step changes, update both.

## Git

Commit the worktree first if it's dirty, then write your changes. Short, to-the-point commit title; a body explaining the change when the title doesn't carry it; ask if you're unsure what to write. On a feature branch, only commit what that branch is for.

No trailers, ever: no `Co-Authored-By` on a commit, no "Generated with Claude Code" on a PR.

## Writing

Commits and issues carry only what matters. Say the thing, explain what a reader won't see for themselves, stop. No restating the diff, no summarising what you just said, no section that exists because the format seemed to want one.

Write issues and their comments the way you'd explain it to a colleague, in complete sentences.

- Start with a 2–3 sentence summary: what happened, why, and the fix.
- Use short headings, with short paragraphs of normal prose under them.
- Never join ideas with arrows, slashes, colons or dashes. Write "first X, then Y" instead of "X → Y", and "A and B" instead of "A / B".
- Use at most two code identifiers per sentence. Say what each one does the first time it appears.
- Use bullets only for genuinely separate items, and make each bullet a full sentence.
- Use tables only for numbers or timelines.
- Use one date format everywhere: 2026-10-07 18:28.
- Put error messages, paths and commands in code formatting, and long logs in a collapsed `<details>` block.
- Put side findings in a short "Out of scope" section at the end.

## How we work

One agent at a time, with me giving feedback as it goes. Keep each step small enough for me to read in one sitting, and stop to show me rather than piling up work I then have to catch up on.

`main` is what I've reviewed. It stays checked out in the repo root: never switch branches there.

Straight to `main` in the root, no branch: typos, one-liners, and anything that only touches how we work rather than the framework — this file, `.claude/skills/`, editor config.

Everything else happens on a branch off `main`, in a worktree under `.worktrees/<branch>` (`git worktree add .worktrees/<branch> -b <branch> main`). Name it `<issue-number>-<slug>` when there is an issue (`39-droptarget-collapse`), `<slug>` when there isn't. One branch is one thing; anything found along the way gets written down as an issue and stays off the branch, unless the branch can't finish without it.

Before starting on anything, check whether it is already half-built: `git branch` and `git worktree list` for the feature, and read what is on the branch. Sessions end mid-feature, and a branch is where that work is — starting again writes it a second time and loses whatever the first attempt learned. If a branch for it exists, continue on it.

No pull requests unless I ask for one. Review happens here, on the diff, before the merge.

## Issues

Issues on `neoworks-dev/svelte-gpui` carry goals across sessions. A session ends and its context goes with it; an issue is the only thing that carries a goal to the next one. So:

- Work that finishes in this session, with me here, needs no issue.
- Work that won't finish in one session gets one, however loosely defined it still is.
- Something concrete found along the way, that isn't what we're doing now, gets one instead of being done.

Write them as soon as the list exists, not once the work starts. Issues filed from an app built on svelte-gpui (Canopy, Grove) name the app and the file there, so the problem can be rebuilt without it.

Every issue gets a type, a priority, an effort and an area. Type, priority and effort are GitHub's own issue type and the organization's issue fields, never labels.

- **Type** — `Bug` something is broken, `Feature` something new, `Task` everything else.
- **Priority** — `Urgent` drop everything, `High` next up, `Medium` normal, `Low` nice to have.
- **Effort** — `Low` an hour or two, `Medium` about a day, `High` several days or needs splitting.

`gh bot` sets all three at once, by name: `gh bot issue create --type Bug --priority High --effort Low --label area:primitives …`, and `gh bot issue edit 12 --priority Urgent` to change them later.

Area is a label, exactly one of:

- `area:primitives` — `runtime/ui/*.svelte` and how `host/src/render/` draws them: layout, events, focus
- `area:style` — `runtime/ui/style.ts` and `host/src/style/`
- `area:runtime` — QuickJS, the DOM shim, the bridge and `protocol.json`
- `area:io` — network and files, `runtime/io/` and `host/src/io/`
- `area:window` — window control, overlays, drag and drop, the clock
- `area:build` — bundling, hot reload, packaging, the host as a library
- `area:testing` — `--do` steps, `scripts/shot.sh`, `tests/` and the test runner
- `area:docs` — `docs/`, `examples/` and the agent skill

Two areas is fine when an issue genuinely spans them; three means split it.

## Done means verified

Work is done when it has been shown to work in a running app, not when the code is written. Every change goes through `scripts/shot.sh` twice: once before, showing the bug or the missing feature, and once after, showing it work, each with a screenshot. Drive the app with `--do` steps rather than describing what a click would do. The only exception is when I've said it is already tested and working.

A test under `tests/` that fails without the change and passes with it comes on top of that, not instead of it. A test that only exercises a helper, or reads the source text, says nothing about whether the primitive draws or the event fires.

This holds for every agent, and most for subagents on smaller models, where it breaks most easily. An agent that hasn't taken the screenshots hasn't finished, whatever its tests say, so give every subagent this rule in its prompt.

`bun test` and `bun run typecheck` pass on the branch either way.

Then hand it over and stop: what changed in a sentence or two, the evidence, and the branch. When it has an issue, the evidence also goes on the issue with `scripts/evidence.sh <n> <body> <before.png> <after.png>`, which puts the screenshots on the `evidence` branch and comments as the bot — that comment is what I read, so it's written for someone who wasn't in the session.

## Merging

I review the branch's diff, and it merges into `main` when I say so — never before, and never without evidence. Merge in the root with `git merge --no-ff <branch>`, so the branch stays one unit in the history; if it conflicts, resolve it in the merge commit. Push `main`. Then remove the worktree and delete the branch, and close its issue with `gh bot issue close <n>`.

## Validation

When I tell you what's happening, that's the reproduction: take it as given and go find the cause, don't re-check what I already saw. Reach for `scripts/shot.sh` when the code doesn't make the cause clear, or a fix based on reading it didn't work — guessing from source has been wrong often enough that a second guess isn't worth it.
