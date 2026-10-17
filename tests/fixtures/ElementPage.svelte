<!-- An app that uses an element, for tests/element.test.ts. -->
<script lang="ts">
  import { element } from "@neoworks/svelte-gpui";

  const Thing = element<
    { path: string; onsave: (detail: { count: number }) => void },
    { insert(text: string): string }
  >("thing");

  let thing = $state<{ insert(text: string): Promise<string> }>();
  let path = $state("/a");
  let saved = $state("not saved");
  let answer = $state("no answer");

  // What the test drives, since nothing can click here.
  (globalThis as any).__page = {
    setPath: (value: string) => (path = value),
    insert: async (text: string) => {
      try {
        answer = await thing!.insert(text);
      } catch (error) {
        answer = `failed: ${(error as Error).message}`;
      }
    },
  };
</script>

<g-view>
  <Thing key="main" bind:this={thing} {path} onsave={(detail) => (saved = `saved ${detail.count}`)} />
  <g-text>{saved}</g-text>
  <g-text>{answer}</g-text>
</g-view>
