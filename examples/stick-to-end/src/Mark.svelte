<!-- Adds a message to `registry` while its row exists. Draws nothing. -->
<script lang="ts">
  let { id, registry }: { id: number; registry: Set<number> } = $props();

  $effect(() => {
    registry.add(id);
    return () => registry.delete(id);
  });
</script>
