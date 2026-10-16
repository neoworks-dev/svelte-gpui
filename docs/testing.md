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
| `hover:TARGET`, `unhover:TARGET` | The pointer enters or leaves it. |
| `focus:TARGET` | Gives it focus. |
| `type:TARGET:TEXT` | Focuses it and types the text, one key at a time. |
| `key:TARGET:KEYS` | A keystroke in GPUI's notation: `enter`, `escape`, `ctrl-a`, `shift-left`, `f2`. |
| `compose:TARGET:TEXT` | Input-method composition, which is underlined and not yet committed. |
| `commit:TARGET:TEXT` | Input-method text committed. |
| `down:TARGET:X,Y`, `move:TARGET:X,Y`, `up:TARGET:X,Y` | The pointer, at window coordinates. `down` captures, as a real press does. |
| `scroll:TARGET:PIXELS` | Scrolls the nearest scrollable element at or above the target to that offset. |
| `wait` | Does nothing for one step. |
| `frames:N` | Draws N frames as fast as the display allows and prints their timing. |

A target that matches nothing prints `nothing matches "…"` to standard error.

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

It creates the output (`svelte-gpui`, or `$OUTPUT`) if it is missing, and moves the
host's window there by address so it never touches other windows. `WIDTH` and `HEIGHT`
set the window size, and the host's log goes to `/tmp/svelte-gpui-host.log`. Make
`SECONDS` long enough for every step to run: about one second per step, plus one.

To test at a higher refresh rate, set the headless output's mode:

```
hyprctl eval 'hl.monitor({output="svelte-gpui", mode="1920x1080@144", position="3440x0", scale=1})'
```

## Measuring frame rate

`frameStats()` in the app gives frames per second and the time building them. Log it
from an interval, and read the log from the host's standard error:

```ts
setInterval(() => console.log(JSON.stringify(frameStats())), 1000);
```

## Type checking

```
bunx svelte-check --tsconfig ./tsconfig.json
```

This catches unknown style properties, wrong prop names, and handlers that expect the
wrong event type.
