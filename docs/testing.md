# Testing

The host can script input and quit on its own, so an app can be checked without anyone
at the keyboard: by a test script, in CI, or by an AI agent.

## Scripted steps

`--do STEP` adds a step, and `--do` can be repeated. Steps run in order, 0.9 seconds
apart, starting 0.9 seconds after launch.

```
svelte-gpui --bundle dist/app.js --do "press:Settings" --do "type:@placeholder=Name:Ada" --do "key:@placeholder=Name:enter" --exit-after-ms 5000
```

A step is `ACTION:TARGET[:ARGUMENT]`. The target is one of:

- **Text**, such as `Settings`: the deepest element whose text contains it.
- **`@attribute=value`**: the first element with that attribute, such as
  `@placeholder=Name`.
- **`@tag=g-…`**: the first primitive of a kind (`g-view`, `g-text`, `g-button`,
  `g-input`, `g-scroll`, `g-image`, `g-icon` or `g-canvas`), such as `@tag=g-canvas`.

| Step | Does |
|---|---|
| `press:TARGET` (or `click:`) | Presses the nearest element at or above the target that listens for `press`. |
| `hover:TARGET`, `unhover:TARGET` | The pointer enters or leaves the nearest element at or above the target that listens for `hover`. |
| `focus:TARGET` | Gives focus to the nearest element at or above the target that can take it. |
| `type:TARGET:TEXT` | Focuses it and types the text, one key at a time. |
| `key:TARGET:KEYS` | A keystroke in GPUI's notation: `enter`, `escape`, `ctrl-a`, `shift-left`, `f2`. |
| `compose:TARGET:TEXT` | Input-method composition, which is underlined and not yet committed. |
| `commit:TARGET:TEXT` | Input-method text committed. |
| `down:TARGET:X,Y`, `move:TARGET:X,Y`, `up:TARGET:X,Y` | The pointer, at window coordinates. `down` captures, as a real press does, and overlays that listen for `outside` presses hear it if it is outside them. A third number names the button: `down:TARGET:X,Y,2` is a right press, which goes to the nearest element at or above the target whose `ondown` takes that button. |
| `drop:TARGET:PATH,PATH` | Files dragged from another program onto the nearest element at or above the target that listens for `drop`, and dropped there. It goes through the code a real drop takes, so the paths are allowed through the file limit as a real drop's are. |
| `dragover:TARGET:PATH,PATH` | The same, but the files are held over the element and not dropped, which shows its `ondragenter` feedback. |
| `scroll:TARGET:PIXELS` | Scrolls the nearest scrollable element at or above the target to that offset. |
| `dismiss` | Dismisses the error overlay, as its button does. |
| `wait` | Does nothing for one step. |
| `frames:N` | Draws N frames as fast as the display allows and prints their timing. |

A target that matches nothing prints `nothing matches "…"` to standard error.

### Colons in a target

A colon splits a step, so a target with one, such as `echo #1: hello` or `12:30`, needs help:

- **`\:` is a colon that does not split**, and `\\` is a backslash: `press:echo #1\: hello`. This works in every
  step. Quote the whole step so the shell leaves the backslash alone.
- **`key`, `scroll`, `down`, `move` and `up` take their argument from the last colon**, so their target can
  hold colons as it is: `key:echo #1: hello:down` presses Down on the element with that text.

In `type`, `compose`, `commit`, `drop` and `dragover` the target ends at the first colon and the rest is the
argument, which may hold colons (`type:@placeholder=Time:12:30`). Escape a colon in the target of those.

`--exit-after-ms N` quits after N ms and prints:

- memory: resident and proportional, QuickJS's share, and the malloc heap;
- event count and time;
- the batches and operations the app sent;
- frames built and their timing.

Add `--report` to print an outline of the element tree as well.

## Screenshots

`scripts/shot.sh` runs a bundle on a headless Hyprland output and saves a screenshot:

```
scripts/shot.sh BUNDLE OUT.png [SECONDS] [host flags…]
scripts/shot.sh examples/demo/dist/demo.js /tmp/demo.png 4 --do press:Lists
```

It creates the output (`svelte-gpui`, or `$OUTPUT`) if it is missing. The host runs with
`--app-id svelte-gpui-shot` (`<output>-shot` in general), and a window rule opens that
window on the output without focus, so nothing on the real monitors moves or loses the
keyboard. Runs with different outputs can go at the same time. `WIDTH` and `HEIGHT`
set the size the host asks for, and the host's log goes to `/tmp/svelte-gpui-host.log` (`/tmp/<output>-host.log`). Make
`SECONDS` long enough for every step to run: about one second per step, plus one.

`HOST` runs another executable instead of `svelte-gpui`: an app's own Rust binary, or a
packaged one. A `BUNDLE` of `-` passes no `--bundle` (and no size), for an app that
brings its own:

```
HOST=dist/package/demo/svelte-gpui-demo scripts/shot.sh - /tmp/packaged.png 3
```

## Measuring frame rate

`frameStats()` in the app gives frames per second and the time building them. Log it
from an interval, and read the log from the host's standard error:

```ts
setInterval(() => console.log(JSON.stringify(frameStats())), 1000);
```

## Errors and hot reload

The host shows errors in an overlay only with `--dev` (or `--watch`); see
[Development](getting-started.md#development-hot-reload-and-errors). To check one, press
something that throws and look at the screenshot:

```
scripts/shot.sh dist/app.js /tmp/error.png 3 --dev --do "press:Explode"
```

Without `--dev` the same run draws no overlay and appends the error to
`~/.cache/svelte-gpui/<bundle name>/errors.log`. The host also prints each error, with
its mapped stack, to its log.

To test hot reload, the host has to stay up while a build runs, which `shot.sh` does not
do. Start the host with `--watch` and `--app-id svelte-gpui-shot` in the background after one
`scripts/shot.sh` run has set up the output and the rule, run `bun build.mjs --watch`, edit
a source file, and take screenshots with `grim -o svelte-gpui out.png`. The host prints `reloaded in N ms` after each reload, so a
script can time an edit to the window updating by polling its log. On the demo that takes
about a quarter of a second, most of it the build.

## Type checking

```
bunx svelte-check --tsconfig ./tsconfig.json
```

This catches unknown style properties, wrong prop names, and handlers that expect the
wrong event type.

## Unit tests

`bun run test` runs the tests in `tests/`, which cover the arithmetic under `List` (where rows sit when
their heights differ). The host's own tests, such as the placement of overlays, are `cargo test --release --lib`
in `host/`.
