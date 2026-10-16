<!--
  Motion three ways: the host eases colour and opacity (`transition` in a style),
  Svelte's own transitions play when blocks come and go, and Svelte's Tween and
  Spring drive any number in a style.
-->
<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { Spring, Tween } from "svelte/motion";
  import { cubicInOut } from "svelte/easing";
  import { Button, Presence, Text, View } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import { button, buttonActive, buttonHover, caption, colors, mono } from "../theme";

  const swatches = [colors.accent, colors.blue, colors.green, colors.amber, colors.pink];

  let shown = $state(true);
  let notes = $state<{ id: number; text: string }[]>([]);
  let nextNote = 1;

  function addNote(): void {
    notes = [{ id: nextNote, text: `Note ${nextNote}` }, ...notes].slice(0, 4);
    nextNote++;
  }

  const progress = new Tween(0, { duration: 900, easing: cubicInOut });
  const ball = new Spring({ x: 40, y: 60 }, { stiffness: 0.08, damping: 0.35 });
  const area = { width: 520, height: 160 };
</script>

<View style={{ gap: 16 }}>
  <Card title="Eased colour and opacity" note="style.transition makes the host ease hover and every change of colour.">
    <View style={{ direction: "row", gap: 10 }}>
      {#each swatches as swatch (swatch)}
        <View
          style={{ flex: 1, height: 56, radius: 10, background: colors.raised, borderWidth: 1, borderColor: colors.border, transition: { duration: 350, easing: "ease-out" } }}
          hover={{ background: swatch, borderColor: swatch }}
          onhover={() => {}}
        />
      {/each}
    </View>
  </Card>

  <View style={{ direction: "row", gap: 16, align: "start" }}>
    <Card title="Svelte transitions" note="fly and fade from svelte/transition, played by the runtime against the host's clock." style={{ flex: 1 }}>
      <View style={{ direction: "row", gap: 8 }}>
        <Button style={button} hover={buttonHover} active={buttonActive} onpress={addNote}><Text>Add a note</Text></Button>
        <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => (shown = !shown)}>
          <Text>{shown ? "Hide" : "Show"} the box</Text>
        </Button>
      </View>
      <View style={{ height: 40 }}>
        {#if shown}
          <Presence in={[fade, { duration: 300 }]} out={[fade, { duration: 300 }]}>
            <View style={{ padding: 10, radius: 8, background: colors.accentSoft }}>
              <Text style={{ color: colors.accentText }}>Fades in and out</Text>
            </View>
          </Presence>
        {/if}
      </View>
      <View style={{ gap: 6, minHeight: 140 }}>
        {#each notes as note (note.id)}
          <Presence in={[fly, { x: -24, duration: 300 }]} out={[fade, { duration: 200 }]}>
            <View style={{ direction: "row", justify: "between", align: "center", padding: 8, radius: 6, background: colors.raised }}>
              <Text>{note.text}</Text>
              <Button style={{ color: colors.faint }} hover={{ color: colors.red }} onpress={() => (notes = notes.filter((other) => other !== note))}>
                <Text>remove</Text>
              </Button>
            </View>
          </Presence>
        {/each}
      </View>
    </Card>

    <Card title="Tween" note="A number eased over time, written into a width." style={{ flex: 1 }}>
      <View style={{ height: 10, radius: 5, background: colors.window, overflow: "hidden" }}>
        <View style={{ width: `${progress.current}%`, height: 10, radius: 5, background: colors.green }} />
      </View>
      <Text style={{ ...mono, color: colors.dim }}>{progress.current.toFixed(1)}%</Text>
      <View style={{ direction: "row", gap: 8 }}>
        {#each [0, 33, 66, 100] as target (target)}
          <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => progress.set(target)}>
            <Text>{target}%</Text>
          </Button>
        {/each}
      </View>
    </Card>
  </View>

  <Card title="Spring" note="Move the pointer over the area; the ball follows on a spring.">
    <View
      style={{ width: area.width, height: area.height, radius: 8, background: colors.window, borderWidth: 1, borderColor: colors.border, overflow: "hidden" }}
      onmove={(event) => ball.set({ x: event.x, y: event.y })}
    >
      <View
        style={{
          position: "absolute",
          left: ball.current.x - 14,
          top: ball.current.y - 14,
          size: 28,
          radius: 14,
          background: colors.accent,
          shadow: { y: 4, blur: 14, color: "#8b5cf680" },
        }}
      />
      <Text style={{ ...caption, padding: 8 }}>{Math.round(ball.current.x)}, {Math.round(ball.current.y)}</Text>
    </View>
  </Card>
</View>
