<!--
  One document in the editor. The text belongs to no file, and its name's extension picks the
  language it is highlighted as.
-->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { Editor, gpuiDocumentHost, type DocumentSummary, type EditorTheme } from "@neoworks/svelte-gpui-editor";

  /** Syntax colours by highlight name; the window colours stay the view's defaults. */
  const theme: Partial<EditorTheme> = {
    syntax: {
      keyword: "#c084fc",
      function: "#60a5fa",
      string: "#4ade80",
      comment: { color: "#71717a", italic: true },
      type: "#fbbf24",
      number: "#fb923c",
    },
  };

  const SAMPLE = `/// Counts the words in a line of text.
fn count_words(line: &str) -> usize {
    line.split_whitespace().count()
}

fn main() {
    let text = "the quick brown fox";
    println!("{} words", count_words(text));
}
`;

  let summary = $state<DocumentSummary | null>(null);
  let failure = $state("");

  gpuiDocumentHost()
    .openText("sample.rs", SAMPLE)
    .then((opened) => (summary = opened))
    .catch((error) => (failure = String(error)));
</script>

<View style={{ flex: 1, background: "#0c0c0e" }}>
  <View style={{ height: 36, paddingX: 14, justify: "center", borderBottomWidth: 1, borderColor: "#26262c" }}>
    <Text style={{ fontSize: 13, color: "#a1a1aa" }}>{summary?.title ?? "Opening…"}</Text>
  </View>
  {#if failure}
    <Text style={{ padding: 14, color: "#f87171" }}>{failure}</Text>
  {/if}
  <Editor key="main" document={summary?.id ?? null} {theme} style={{ flex: 1 }} />
</View>
