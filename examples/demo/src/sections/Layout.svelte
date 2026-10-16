<!-- Flexbox, as GPUI lays it out: every primitive is a flex container. -->
<script lang="ts">
  import { Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import Segmented from "../components/Segmented.svelte";
  import { caption, colors, mono } from "../theme";

  type Justify = "start" | "center" | "end" | "between" | "around";
  type Align = "start" | "center" | "end" | "stretch";

  let justify = $state<Justify>("between");
  let align = $state<Align>("center");

  const box = (color: string, extra: Style = {}): Style => ({
    background: color,
    radius: 6,
    padding: 8,
    minWidth: 44,
    align: "center",
    justify: "center",
    ...extra,
  });

  const tags = ["svelte", "gpui", "quickjs", "rust", "flexbox", "runes", "native", "fast", "small", "typed styles", "no DOM", "no CSS"];
</script>

<View style={{ gap: 16 }}>
  <Card title="justify and align" note="Pick a value and the row lays itself out again.">
    <View style={{ direction: "row", gap: 16, wrap: true }}>
      <Segmented options={["start", "center", "end", "between", "around"]} bind:value={justify} />
      <Segmented options={["start", "center", "end", "stretch"]} bind:value={align} />
    </View>
    <View
      style={{
        direction: "row",
        justify,
        align,
        gap: 8,
        height: 96,
        padding: 8,
        radius: 8,
        background: colors.window,
        borderWidth: 1,
        borderColor: colors.border,
      }}
    >
      <View style={box(colors.accent, { height: align === "stretch" ? undefined : 28 })}><Text>A</Text></View>
      <View style={box(colors.blue, { height: align === "stretch" ? undefined : 52 })}><Text>B</Text></View>
      <View style={box(colors.pink, { height: align === "stretch" ? undefined : 40 })}><Text>C</Text></View>
    </View>
    <Text style={{ ...mono, color: colors.faint }}>{`{ direction: "row", justify: "${justify}", align: "${align}" }`}</Text>
  </Card>

  <View style={{ direction: "row", gap: 16 }}>
    <Card title="grow" note="flex: 1, 2 and 1 share the row." style={{ flex: 1 }}>
      <View style={{ direction: "row", gap: 6 }}>
        <View style={box(colors.raised, { flex: 1 })}><Text style={caption}>1</Text></View>
        <View style={box(colors.accentSoft, { flex: 2 })}><Text style={{ ...caption, color: colors.accentText }}>2</Text></View>
        <View style={box(colors.raised, { flex: 1 })}><Text style={caption}>1</Text></View>
      </View>
      <View style={{ direction: "row", gap: 6 }}>
        <View style={box(colors.raised, { width: 64 })}><Text style={caption}>64</Text></View>
        <View style={box(colors.accentSoft, { flex: 1 })}><Text style={{ ...caption, color: colors.accentText }}>the rest</Text></View>
        <View style={box(colors.raised, { width: "25%" })}><Text style={caption}>25%</Text></View>
      </View>
    </Card>

    <Card title="wrap" note="Children flow onto the next line when the row is full." style={{ flex: 1 }}>
      <View style={{ direction: "row", wrap: true, gap: 6 }}>
        {#each tags as tag (tag)}
          <View style={{ paddingX: 8, paddingY: 2, radius: 999, borderWidth: 1, borderColor: colors.strongBorder }}>
            <Text style={{ fontSize: 12, color: colors.dim, nowrap: true }}>{tag}</Text>
          </View>
        {/each}
      </View>
    </Card>
  </View>

  <Card title="Boxes" note="Radius per corner, borders, shadows, opacity and absolute positioning.">
    <View style={{ direction: "row", gap: 20, align: "center", wrap: true }}>
      <View style={{ size: 64, radius: 12, background: colors.raised, borderWidth: 1, borderColor: colors.strongBorder }} />
      <View style={{ size: 64, radius: [24, 4, 24, 4], background: colors.accent }} />
      <View style={{ size: 64, radius: 32, background: colors.blue, shadow: { y: 6, blur: 16, color: "#60a5fa66" } }} />
      <View style={{ size: 64, radius: 8, borderWidth: [0, 0, 4, 0], borderColor: colors.green, background: colors.raised }} />
      <View style={{ direction: "row", gap: 4 }}>
        {#each [1, 0.75, 0.5, 0.25] as opacity (opacity)}
          <View style={{ width: 14, height: 64, radius: 4, background: colors.pink, opacity }} />
        {/each}
      </View>
      <View style={{ size: 64, radius: 32, background: colors.raised, align: "center", justify: "center" }}>
        <Text style={{ fontWeight: "bold" }}>SG</Text>
        <View
          style={{
            position: "absolute",
            top: -2,
            right: -2,
            size: 18,
            radius: 9,
            background: colors.red,
            borderWidth: 2,
            borderColor: colors.surface,
            align: "center",
            justify: "center",
          }}
        >
          <Text style={{ fontSize: 10, fontWeight: "bold" }}>3</Text>
        </View>
      </View>
    </View>
  </Card>
</View>
