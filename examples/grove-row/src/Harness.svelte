<script lang="ts">
  // Grove's own row, fed Grove's own item shape, in a pane-sized column.
  import GithubItemRow from '@grove/src/renderer/src/kernel/plugins/github/GithubItemRow.svelte';
  import data from './items.json';

  let selected = $state<number | null>(null);
  let checked = $state<number[]>([]);

  function toggle(number: number): void {
    checked = checked.includes(number) ? checked.filter((n) => n !== number) : [...checked, number];
  }
</script>

<div class="flex h-full flex-col overflow-y-auto bg-surface text-default">
  {#each data.items as item (item.kind + item.number)}
    <GithubItemRow
      item={item as any}
      selected={selected === item.number}
      checked={checked.includes(item.number)}
      isViewer={item.author === data.viewer}
      onselect={() => (selected = item.number)}
      ontoggle={() => toggle(item.number)}
    />
  {/each}
</div>
