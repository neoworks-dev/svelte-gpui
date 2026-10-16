<!--
  The pointer: positions inside the element, buttons, and capture. A press on the
  pad holds the pointer until release, so a stroke that leaves the pad goes on.
-->
<script lang="ts">
  import { Button, Canvas, Text, View, type PointerEvent } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import { caption, colors, mono } from "../theme";

  const palette = [colors.accent, colors.blue, colors.green, colors.amber, colors.pink, colors.text];
  const width = 560;
  const height = 300;

  let pad: Canvas;
  let color = $state(colors.accent);
  let brush = $state(6);
  let drawing = false;
  let strokes = $state(0);
  let points = $state(0);

  let hovering = $state(false);
  let where = $state({ x: 0, y: 0 });
  let wheel = $state(0);

  function start(event: PointerEvent): void {
    drawing = true;
    strokes++;
    points++;
    pad.begin(event.x, event.y, { color, width: brush });
  }

  function move(event: PointerEvent): void {
    where = { x: Math.round(event.x), y: Math.round(event.y) };
    if (drawing) {
      points++;
      pad.lineTo(event.x, event.y);
    }
  }

  function stop(): void {
    drawing = false;
    pad.end();
  }
</script>

<View style={{ gap: 16 }}>
  <Card title="Drawing" note="A Canvas: the host paints each stroke into pixels as the pointer moves, so a long drawing costs no more than a short one.">
    <View style={{ direction: "row", gap: 8, align: "center" }}>
      {#each palette as swatch (swatch)}
        <Button
          style={{ size: 22, radius: 11, background: swatch, borderWidth: 2, borderColor: swatch === color ? colors.text : colors.surface }}
          onpress={() => (color = swatch)}
        />
      {/each}
      <View style={{ width: 1, height: 20, background: colors.border, marginX: 4 }} />
      {#each [3, 6, 12] as size (size)}
        <Button
          style={{ size: 28, radius: 6, align: "center", justify: "center", background: size === brush ? colors.raised : "transparent" }}
          hover={{ background: colors.raised }}
          onpress={() => (brush = size)}
        >
          <View style={{ size, radius: size / 2, background: colors.dim }} />
        </Button>
      {/each}
      <View style={{ flex: 1 }} />
      <Text style={{ ...mono, color: colors.faint }}>{strokes} strokes, {points} points</Text>
      <Button
        style={{ paddingX: 10, paddingY: 5, radius: 6, background: colors.raised }}
        hover={{ background: colors.hover }}
        onpress={() => {
          pad.clear();
          strokes = 0;
          points = 0;
        }}
      >
        <Text>Clear</Text>
      </Button>
    </View>
    <View style={{ width, height, radius: 8, background: colors.window, borderWidth: 1, borderColor: colors.border, overflow: "hidden" }}>
      <Canvas
        bind:this={pad}
        style={{ flex: 1, cursor: "crosshair" }}
        ondown={start}
        onmove={move}
        onup={stop}
        onhover={(inside) => (hovering = inside)}
      />
      {#if strokes === 0}
        <View style={{ position: "absolute", inset: 0, align: "center", justify: "center" }}>
          <Text style={caption}>Draw here</Text>
        </View>
      {/if}
    </View>
  </Card>

  <View style={{ direction: "row", gap: 16 }}>
    <Card title="Hover and position" style={{ flex: 1 }}>
      <Text style={{ ...mono, color: hovering ? colors.green : colors.faint }}>
        {hovering ? `over the pad at ${where.x}, ${where.y}` : "not over the pad"}
      </Text>
    </Card>
    <Card title="Wheel" note="Scroll over the box." style={{ flex: 1 }}>
      <View
        style={{ height: 48, radius: 8, background: colors.window, align: "center", justify: "center" }}
        onwheel={(event) => (wheel = Math.round(wheel + event.dy))}
      >
        <Text style={mono}>{wheel} px</Text>
      </View>
    </Card>
  </View>
</View>
