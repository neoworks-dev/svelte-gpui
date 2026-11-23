<!--
  Text that can be selected. A plain paragraph, one with an inline box (the star), a one-line paragraph whose
  icon says what it copies as, a button whose label is selectable and must still be pressed, text that is not
  selectable and sits between selectable paragraphs, a field to paste what was copied into, and a list of 300
  selectable rows, of which the host builds only a screenful, so rows come and go as it scrolls.
-->
<script lang="ts">
  import { clipboard } from "@neoworks/svelte-gpui";
  import { Button, Icon, Input, List, Text, View } from "@neoworks/svelte-gpui/ui";

  const star =
    '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2l3 7 7 .6-5.4 4.8 1.7 7.2L12 17.8 5.7 21.6l1.7-7.2L2 9.6 9 9z"/></svg>';

  let pressed = $state(0);
  let pasted = $state("");
  const notes = Array.from({ length: 300 }, (_, index) => `Note ${index + 1}: a line of text in a list that scrolls`);
</script>

<View style={{ height: 600, padding: 20, gap: 14, background: "#111114", color: "#e4e4e7", fontSize: 15, lineHeight: 22 }}>
  <Text selectable>
    A plain paragraph that wraps over several lines, with a <Text style={{ fontWeight: "bold", color: "#c4b5fd" }}>bold
    run</Text> inside it. Drag across it, double-click a word, or triple-click to take all of it.
  </Text>

  <Text selectable>
    A paragraph with a star <Icon svg={star} style={{ color: "#facc15" }} /> in the middle of its line, which wraps
    with the words, so the selection runs across it and the line below.
  </Text>

  <Text selectable>Rated <Icon svg={star} copyText="(star)" style={{ color: "#facc15" }} /> out of five.</Text>

  <Button onpress={() => pressed++} style={{ padding: 10, radius: 6, background: "#27272a", alignSelf: "start" }}>
    <Text selectable>Press this label, selectable or not</Text>
  </Button>
  <View style={{ direction: "row", align: "center", gap: 12 }}>
    <Text>Pressed {pressed} times.</Text>
    <Button
      onpress={async () => (pasted = await clipboard.read())}
      style={{ paddingX: 8, paddingY: 2, radius: 6, background: "#27272a" }}
    >
      <Text>Read the clipboard into the field</Text>
    </Button>
  </View>

  <Text>This paragraph is not selectable, and its pointer is the arrow.</Text>

  <Text selectable>A last selectable paragraph after the one that is not.</Text>

  <Input value={pasted} oninput={(value: string) => (pasted = value)} placeholder="Paste here"
    style={{ padding: 8, background: "#1c1c21", radius: 6 }} />

  <List items={notes} rowHeight={22} style={{ height: 120, background: "#18181b", radius: 6 }}>
    {#snippet row(note)}
      <View style={{ height: 22, paddingX: 8 }}><Text selectable>{note}</Text></View>
    {/snippet}
  </List>
</View>
