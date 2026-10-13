<script lang="ts">
  // The smallest thing that exercises what a real component does: state, a
  // conditional, a keyed list, a class that depends on state, and a click.
  const names = ['Board: drag a card between columns', 'Board: group by Effort', 'QA sample issue'];

  let count = $state(0);
  let selected = $state<number | null>(null);
  let shown = $state(true);
</script>

<div class="flex h-full flex-col bg-surface text-default text-xs">
  <div class="flex items-center gap-1.5 border-b border-line px-2 py-1.5">
    <button class="rounded-md px-1.5 py-0.5 hover:bg-hover" onclick={() => (count += 1)}>
      Clicked <span class="font-mono text-2xs text-dim">{count}</span>
    </button>
    <button class="rounded-md px-1.5 py-0.5 text-dim hover:bg-hover" onclick={() => (shown = !shown)}>
      {shown ? 'Hide' : 'Show'} list
    </button>
  </div>

  {#if shown}
    {#each names as name, index (name)}
      <div
        class="flex cursor-pointer items-start gap-2 border-b border-line px-2 py-1.5 hover:bg-hover"
        class:bg-elevated={selected === index}
        onclick={() => (selected = index)}
      >
        <span class="mt-px text-green">●</span>
        <div class="min-w-0 flex-1">
          <div class="font-medium">{name}</div>
          <div class="mt-0.5 text-2xs text-dim">#{index + 1} · opened {index + 1}d ago</div>
        </div>
      </div>
    {/each}
  {:else}
    <p class="px-3 py-4 text-dim">Nothing here.</p>
  {/if}
</div>
