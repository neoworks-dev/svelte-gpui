<script lang="ts">
  // Svelte transitions and CSS transitions, slowed down so a screenshot can catch them.
  import { fade, fly } from 'svelte/transition';

  let shown = $state(true);
  let lit = $state(false);
  let faint = $state(false);
</script>

<div class="flex h-full flex-col gap-3 bg-surface p-3 text-xs text-default">
  <div class="flex gap-2">
    <button class="rounded-md border border-line px-2 py-1 hover:bg-hover" onclick={() => (shown = !shown)}>
      Toggle the card
    </button>
    <button class="rounded-md border border-line px-2 py-1 hover:bg-hover" onclick={() => (lit = !lit)}>
      Toggle the colour
    </button>
    <button class="rounded-md border border-line px-2 py-1 hover:bg-hover" onclick={() => (faint = !faint)}>
      Toggle the opacity
    </button>
  </div>

  <div class="h-24">
    {#if shown}
      <div
        class="w-64 rounded-md border border-line bg-elevated p-3"
        in:fly={{ y: 40, duration: 3000 }}
        out:fade={{ duration: 3000 }}
      >
        <div class="font-medium">A card that flies in and fades out</div>
        <div class="mt-1 text-dim">Svelte drives it with element.animate().</div>
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-3">
    <div class="size-16 rounded-md border border-line transition-colors duration-[3000ms] {lit ? 'bg-green' : 'bg-input'}"></div>
    <div class="size-16 rounded-md bg-violet transition-opacity duration-[3000ms] {faint ? 'opacity-10' : 'opacity-100'}"></div>
    <div class="rounded-md border border-line bg-input px-3 py-2 transition-colors duration-[1500ms] hover:bg-green">
      Hover me slowly
    </div>
  </div>
</div>
