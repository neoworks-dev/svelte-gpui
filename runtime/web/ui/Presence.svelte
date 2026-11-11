<!--
  A view that plays Svelte transitions as it comes and goes. Svelte only takes `in:`, `out:` and
  `transition:` on elements, so this carries them for a component tree:
  `<Presence in={[fly, { x: -20 }]} out={fade}>`.
-->
<script lang="ts">
  import type { TransitionConfig } from "svelte/transition";
  import type { PresenceProps, TransitionFunction, TransitionProp } from "../../ui/props";
  import { cssText, toCss } from "./css";

  let props: PresenceProps = $props();

  const still = (): TransitionConfig => ({ duration: 0 });
  const split = (given: TransitionProp | undefined): [TransitionFunction, unknown] =>
    Array.isArray(given) ? given : [given ?? still, undefined];
  const [enter, enterParams] = $derived(split(props.in));
  const [leave, leaveParams] = $derived(split(props.out));
</script>

<div class="g-view" style={cssText(toCss(props.style))} in:enter={enterParams} out:leave={leaveParams}>
  {@render props.children?.()}
</div>
