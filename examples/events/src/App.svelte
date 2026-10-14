<script lang="ts">
  // Every kind of input the host reports, each with a readout, so a screenshot
  // shows which ones reached Svelte.
  import CircleIcon from 'phosphor-svelte/lib/CircleIcon';
  import ChatCircleIcon from 'phosphor-svelte/lib/ChatCircleIcon';
  import GitMergeIcon from 'phosphor-svelte/lib/GitMergeIcon';

  let hover = $state('out');
  let hoverCount = $state(0);
  let down = $state('none');
  let up = $state(0);
  let menu = $state(0);
  let wheel = $state(0);
  let scrollTop = $state(0);
  let boxWidth = $state(0);
  let focused = $state('nothing');
  let lastKey = $state('none');
  let globalKey = $state('none');
  let windowMoves = $state(0);
  let windowReleases = $state(0);
  let lastWindowPoint = $state('none');
  let windowWidth = $state(0);
  let imageNote = $state('waiting');
  let brokenNote = $state('waiting');
  let remoteNote = $state('waiting');
  let clicks = $state(0);

  const rows = Array.from({ length: 40 }, (_, index) => 'Row ' + (index + 1));
  const png = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAIAAABvFaqvAAAEIklEQVR4nAXBIRREQABAQZlMJZOpZPJmMpX3fjuZSiZvJlPJZCqZ7GaUD+VFuVFOlB1lRZlRRhSJ0qO0KDVKiVKgZCgJikCJUAIUD8VBsVAMFA1F+VBf1Bv1RN1RV9QZdUSVqD1qi1qjlqgFaoaaoArUCDVA9VAdVAvVQNVQlQ/9Rb/RT/QdfUWf0Ud0id6jt+g1eoleoGfoCbpAj9ADdA/dQbfQDXQNXfkwX8wb88TcMVfMGXPElJg9ZotZY5aYBWaGmWAKzAgzwPQwHUwL08DUMJUP+8W+sU/sHXvFnrFHbIndY7fYNXaJXWBn2Am2wI6wA2wP28G2sA1sDVv5cF/cG/fE3XFX3Bl3xJW4PW6LW+OWuAVuhpvgCtwIN8D1cB1cC9fA1XCVD//Fv/FP/B1/xZ/xR3yJ3+O3+DV+iV/gZ/gJvsCP8AN8D9/Bt/ANfA1f+QhfwpvwJNwJV8KZcCSUhD1hS1gTloQFYUaYEArCiDAg9AgdQovQINQIlQ/xIm7EidgRK2JGjAiJ6BEtokaUiAKRIRKEQESIAOEhHISFMBAaQvmIX+Kb+CTeiVfimXgklsQ9cUtcE5fEBXFGnBAL4og4IPaIHWKL2CDWiJWP9CW9SU/SnXQlnUlHUknak7akNWlJWpBmpAmpII1IA1KP1CG1SA1SjVT5yF/ym/wk38lX8pl8JJfkPXlLXpOX5AV5Rp6QC/KIPCD3yB1yi9wg18iVj9/L7+Z38tv5rfxmfiM/ya/n1/Kr+ZX8Cn4Zv4Sf4BfxC/h5/Bx+Fj+Dn8ZP+aheqpvqpNqpVqqZaqSSVD1VS1VTlVQFVUaVUAmqiCqg8qgcKovKoNKolI/mpblpTpqdZqWZaUYaSdPTtDQ1TUlT0GQ0CY2giWgCGo/GobFoDBqNRvnoXrqb7qTb6Va6mW6kk3Q9XUtX05V0BV1Gl9AJuoguoPPoHDqLzqDT6JQP+SJv5InckStyRo5IieyRLbJGlsgCmSETpEBGyADpIR2khTSQGlL5GF6Gm+Fk2BlWhplhZJAMPUPLUDOUDAVDxpAwCIaIIWDwGBwGi8Fg0BiUj+lluplOpp1pZZqZRibJ1DO1TDVTyVQwZUwJk2CKmAImj8lhspgMJo1J+VhelpvlZNlZVpaZZWSRLD1Ly1KzlCwFS8aSsAiWiCVg8VgcFovFYNFYlI/tZbvZTradbWWb2UY2ydaztWw1W8lWsGVsCZtgi9gCNo/NYbPYDDaNTfk4Xo6b4+TYOVaOmWPkkBw9R8tRc5QcBUfGkXAIjogj4PA4HA6Lw+DQOJSP6+W6uU6unWvlmrlGLsnVc7VcNVfJVXBlXAmX4Iq4Ai6Py+GyuAwujUv5eF6em+fk2XlWnpln5JE8PU/LU/OUPAVPxpPwCJ6IJ+DxeBwei8fg0Xj+y5pL7sqYPzQAAAAASUVORK5CYII=';
</script>

<svelte:window
  onkeydown={(event) => (globalKey = event.key)}
  onmousemove={(event) => { windowMoves += 1; lastWindowPoint = event.clientX + ',' + event.clientY; }}
  onmouseup={() => (windowReleases += 1)}
  bind:innerWidth={windowWidth}
/>

<div class="flex h-full flex-col gap-2 bg-surface p-3 text-xs text-default">
  <div class="flex items-center gap-3">
    <div
      class="rounded-md border border-line px-2 py-1 hover:bg-hover"
      onmouseenter={() => { hover = 'in'; hoverCount += 1; }}
      onmouseleave={() => (hover = 'out')}
    >
      Hover me: <span class="font-mono text-green">{hover}</span>
      <span class="text-dim">({hoverCount})</span>
    </div>
    <div
      class="rounded-md border border-line px-2 py-1"
      onmousedown={(event) => (down = String(event.button))}
      onmouseup={() => (up += 1)}
      oncontextmenu={() => (menu += 1)}
      onclick={() => (clicks += 1)}
    >
      mouse: down <span class="font-mono text-green">{down}</span>
      up <span class="font-mono text-green">{up}</span>
      menu <span class="font-mono text-green">{menu}</span>
      click <span class="font-mono text-green">{clicks}</span>
    </div>
  </div>

  <div class="flex items-center gap-3">
    <div class="rounded-md border border-line px-2 py-1" onwheel={(event) => (wheel += event.deltaY)}>
      Wheel over me: <span class="font-mono text-green">{wheel}</span>
    </div>
    <div
      class="rounded-md border border-line px-2 py-1"
      tabindex="0"
      onfocus={() => (focused = 'focus box')}
      onblur={() => (focused = 'nothing')}
      onkeydown={(event) => (lastKey = event.key)}
    >
      Focus me and type: <span class="font-mono text-green">{lastKey}</span>
    </div>
  </div>

  <div class="text-dim">
    focused: <span class="font-mono text-green">{focused}</span>
    window key: <span class="font-mono text-green">{globalKey}</span>
    window moves <span class="font-mono text-green">{windowMoves}</span> releases <span class="font-mono text-green">{windowReleases}</span> at <span class="font-mono text-green">{lastWindowPoint}</span>
    window width: <span class="font-mono text-green">{windowWidth}</span>
  </div>

  <div class="flex items-center gap-3">
    <CircleIcon size={14} class="text-green" />
    <GitMergeIcon size={14} class="text-violet" />
    <ChatCircleIcon size={14} class="text-dim" />
    <img src={png} alt="" class="size-6 rounded-full" onload={() => (imageNote = 'loaded')} />
    <img src="https://github.com/Letsmoe.png?size=40" alt="" class="size-6 rounded-full object-cover" onload={() => (remoteNote = 'loaded')} onerror={() => (remoteNote = 'failed')} />
    <img src="/nowhere/missing.png" alt="" class="size-6" onerror={() => (brokenNote = 'failed')} />
    <span class="text-dim">
      data url: <span class="font-mono text-green">{imageNote}</span>
      missing: <span class="font-mono text-green">{brokenNote}</span>
      remote: <span class="font-mono text-green">{remoteNote}</span>
    </span>
  </div>

  <div class="text-dim">
    scrollTop <span class="font-mono text-green">{Math.round(scrollTop)}</span>
    box width <span class="font-mono text-green">{Math.round(boxWidth)}</span>
  </div>
  <div
    class="min-h-0 flex-1 overflow-y-auto rounded-md border border-line"
    bind:clientWidth={boxWidth}
    onscroll={(event) => (scrollTop = event.currentTarget.scrollTop)}
  >
    {#each rows as row (row)}
      <div class="border-b border-line px-2 py-1.5 hover:bg-hover">{row}</div>
    {/each}
  </div>
</div>
