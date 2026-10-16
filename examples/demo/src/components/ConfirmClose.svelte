<!--
  Asks before the window closes with an unsaved note: an overlay over the whole window,
  made of primitives. It is the last child of the root, so it draws on top.
-->
<script lang="ts">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  import { answerDiscard, closing } from "../unsaved.svelte";
  import { button, buttonActive, buttonHover, colors, dim } from "../theme";

  const discard = { ...button, background: colors.red, borderColor: colors.red, color: "#18181b" };
</script>

{#if closing.asking}
  <View
    style={{ position: "absolute", inset: 0, align: "center", justify: "center", background: "#000000aa" }}
    onpress={() => {}}
  >
    <View
      style={{ width: 380, padding: 20, gap: 16, radius: 10, background: colors.surface, borderWidth: 1, borderColor: colors.strongBorder }}
      onpress={() => {}}
    >
      <View style={{ gap: 6 }}>
        <Text style={{ fontSize: 15, fontWeight: "semibold" }}>Close without saving?</Text>
        <Text style={dim}>The note in Files has changes that are not saved. Closing now loses them.</Text>
      </View>
      <View style={{ direction: "row", justify: "end", gap: 8 }}>
        <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => answerDiscard(false)}>
          <Text>Keep editing</Text>
        </Button>
        <Button style={discard} hover={{ background: "#ef4444" }} onpress={() => answerDiscard(true)}>
          <Text>Discard and close</Text>
        </Button>
      </View>
    </View>
  </View>
{/if}
