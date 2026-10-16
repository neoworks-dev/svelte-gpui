# Motion

There are four ways to animate, from cheapest to most flexible:

| Want | Use |
|---|---|
| A colour or opacity change to ease, including on hover | `style.transition` |
| Something to fade or slide in and out as it mounts and unmounts | `Presence` with a Svelte transition |
| A number to move smoothly to a new value | `Tween` or `Spring` from `svelte/motion` |
| Anything else, every frame | `requestAnimationFrame` |

All of them run at the display's refresh rate: the host advances the app's clock on
every display frame while something animates, and stops when nothing does.

## Eased styles

```svelte
<View style={{ background: selected ? "#8b5cf622" : "transparent", transition: 150 }} hover={{ background: "#26262c" }} />
```

The host eases `background`, `color`, `borderColor` and `opacity` towards whatever the
style says now. It does this in Rust, without running JavaScript per frame. See
[Transitions](styles.md#transitions).

## Svelte transitions with Presence

Svelte's `in:`, `out:` and `transition:` directives only work on elements, and an app
here only uses components. `Presence` is a `View` that carries them:

```svelte
<script lang="ts">
  import { Presence, Text } from "@neoworks/svelte-gpui/ui";
  import { fade, fly, slide } from "svelte/transition";
</script>

{#if toast}
  <Presence in={[fly, { y: 12, duration: 200 }]} out={[fade, { duration: 150 }]}>
    <Text>{toast}</Text>
  </Presence>
{/if}
```

`in` and `out` take a transition function or `[function, params]`. Leave one out and
that direction is instant. Inside a keyed `{#each}`, wrap each row in a `Presence`.

The runtime plays the transition's CSS keyframes against the host's clock, and the host
draws the properties it knows:

| Transition | Draws |
|---|---|
| `fade` | Fully. |
| `fly` | Fully: the offset and the fade. |
| `slide` | Vertically (the default `axis: "y"`): height, padding, margins and clipping. |
| `scale` | Only its fade; scaling is not drawn. |
| `blur`, `draw` | No. |
| Your own, with `css` | `opacity` and `transform: translate(…)`, `translateX` and `translateY`. |
| Your own, with `tick` | Anything, since your code sets state each frame. |

`animate:flip` needs an element in a keyed each block and does not apply here.

## Tween and Spring

`Tween` and `Spring` from `svelte/motion` work as they do on the web. Read `.current`
inside a style:

```svelte
<script lang="ts">
  import { Spring, Tween } from "svelte/motion";
  import { cubicOut } from "svelte/easing";

  const width = new Tween(0, { duration: 400, easing: cubicOut });
  const ball = new Spring({ x: 0, y: 0 }, { stiffness: 0.1, damping: 0.4 });
</script>

<View style={{ width: `${width.current}%`, height: 4, background: "#8b5cf6" }} />
<View
  style={{ flex: 1 }}
  onmove={(e) => ball.set({ x: e.x, y: e.y })}
>
  <View style={{ position: "absolute", size: 20, radius: 10, background: "#fff", translate: [ball.current.x - 10, ball.current.y - 10] }} />
</View>
```

Use `translate` to move things. It changes position after layout, so nothing else
moves. `prefersReducedMotion` from `svelte/motion` is always `false`.

## Animation frames

`requestAnimationFrame` and `cancelAnimationFrame` exist and fire once per display
frame. `performance.now()` is the host's clock. A loop that draws on a `Canvas` each
frame:

```svelte
<script lang="ts">
  let pad: Canvas;
  let t = 0;

  $effect(() => {
    let handle = requestAnimationFrame(function step() {
      t += 1;
      pad.fillCircle(200 + Math.cos(t / 30) * 100, 100 + Math.sin(t / 30) * 60, 2, "#8b5cf6");
      handle = requestAnimationFrame(step);
    });
    return () => cancelAnimationFrame(handle);
  });
</script>
```

`setTimeout` and `setInterval` run off the same clock, and fire on time even when no
frames are being drawn.

## What it costs

`frameStats()` from `@neoworks/svelte-gpui` gives the frames drawn in the last second
and the average time building them. The demo shows it in its sidebar. An idle window
draws almost nothing; a running animation draws at the display's rate.
