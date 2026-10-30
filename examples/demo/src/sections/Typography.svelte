<!-- Text: one paragraph per `Text`, with nested `Text` for runs that look different. -->
<script lang="ts">
  import { Icon, Text, View } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import Slider from "../components/Slider.svelte";
  import { caption, colors, mono } from "../theme";

  let width = $state(420);
  const inlineWidth = 360;
  const star = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2l3 6.9 7.5.7-5.7 5 1.7 7.4L12 18l-6.5 4 1.7-7.4-5.7-5 7.5-.7z"/></svg>`;

  const sizes = [11, 13, 15, 18, 24, 32];
  const weights = [300, 400, 500, 600, 700, 800] as const;

  const code = `<script lang="ts">
  import { Button, Text } from "@neoworks/svelte-gpui/ui";
  let count = $state(0);
<\/script>

<Button onpress={() => count++}>
  <Text>Pressed {count} times</Text>
</Button>`;
</script>

<View style={{ gap: 16 }}>
  <View style={{ direction: "row", gap: 16 }}>
    <Card title="Sizes" style={{ flex: 1 }}>
      {#each sizes as size (size)}
        <View style={{ direction: "row", align: "center", gap: 10 }}>
          <Text style={{ ...mono, color: colors.faint, width: 28 }}>{size}</Text>
          <Text style={{ fontSize: size }}>The quick brown fox</Text>
        </View>
      {/each}
    </Card>
    <Card title="Weights and faces" style={{ flex: 1 }}>
      {#each weights as weight (weight)}
        <Text style={{ fontSize: 15, fontWeight: weight }}>Geist {weight}</Text>
      {/each}
      <Text style={{ fontFamily: "mono" }}>JetBrains Mono for code</Text>
    </Card>
  </View>

  <Card title="Runs" note="Nested Text shares one paragraph, so it wraps as one.">
    <Text style={{ fontSize: 15, lineHeight: 24 }}>
      Svelte compiles the component, <Text style={{ fontWeight: "bold" }}>QuickJS</Text> runs it, and
      <Text style={{ color: colors.accentText }}>GPUI</Text> draws it. Runs can be
      <Text style={{ italic: true }}>italic</Text>, <Text style={{ underline: true }}>underlined</Text>,
      <Text style={{ strikethrough: true, color: colors.faint }}>struck out</Text>,
      <Text style={{ fontFamily: "mono", background: colors.raised, color: colors.pink }}> monospaced </Text>, or
      <Text style={{ uppercase: true, fontSize: 12, color: colors.green }}>shouting</Text>, all in the same line.
    </Text>
  </Card>

  <Card title="Inline boxes" note="Other elements in a Text sit in its lines and wrap with the words.">
    <View style={{ width: inlineWidth, padding: 12, radius: 8, background: colors.window }}>
      <Text style={{ fontSize: 15, lineHeight: 24 }}>
        An icon <Icon svg={star} color={colors.amber} /> sits on the baseline like a word, and a badge
        <View style={{ width: 44, height: 18, radius: 9, background: colors.accent, align: "center", justify: "center" }}
          ><Text style={{ fontSize: 11 }}>new</Text></View
        > wraps with the text. A taller box makes its line taller
        <Icon svg={star} size={40} color={colors.green} style={{ verticalAlign: -10 }} /> and
        <Text style={{ italic: true }}>verticalAlign</Text> lowers it, as a formula's depth hangs below the line.
      </Text>
    </View>
  </Card>

  <Card title="Wrapping" note="Drag to change the column's width.">
    <Slider bind:value={width} min={160} max={640} width={260} />
    <View style={{ direction: "row", gap: 16, align: "start" }}>
      <View style={{ width, gap: 10, padding: 12, radius: 8, background: colors.window }}>
        <Text style={caption}>wraps</Text>
        <Text>
          A paragraph wraps at word boundaries to fit its box, however wide the box is, and the box grows to hold it.
        </Text>
        <Text style={caption}>truncate</Text>
        <Text style={{ truncate: true }}>
          One line only, cut off with an ellipsis where it runs out of room, however long it is.
        </Text>
        <Text style={caption}>maxLines: 2</Text>
        <Text style={{ maxLines: 2 }}>
          At most two lines. Anything past the second line is not drawn, so a long description stays a preview of itself
          and the layout below it does not move.
        </Text>
      </View>
    </View>
  </Card>

  <Card title="pre" note="Whitespace and line breaks kept as written.">
    <View style={{ padding: 12, radius: 8, background: colors.window }}>
      <Text pre style={{ ...mono, color: colors.dim, lineHeight: 18 }}>{code}</Text>
    </View>
  </Card>
</View>
