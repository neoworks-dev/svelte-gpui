<!--
  A view that plays Svelte transitions as it comes and goes. Svelte only takes
  `in:`, `out:` and `transition:` on elements, so this carries them for a
  component tree: `<Presence in={[fly, { x: -20 }]} out={fade}>`.
  The transitions write `opacity` and `transform: translate(…)` each frame, which
  the host draws; other properties they animate are not drawn.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { TransitionConfig } from "svelte/transition";
  import { encode, type Style } from "./style";

  type TransitionFunction = (
    node: any,
    params: any,
    options: { direction: "in" | "out" | "both" },
  ) => TransitionConfig | (() => TransitionConfig);

  /** A transition, or one with its parameters: `fade`, `[fly, { x: -20, duration: 300 }]`. */
  type TransitionProp = TransitionFunction | [TransitionFunction, unknown];

  interface Props {
    in?: TransitionProp;
    out?: TransitionProp;
    style?: Style;
    children?: Snippet;
  }

  let props: Props = $props();

  const still = (): TransitionConfig => ({ duration: 0 });
  const split = (given: TransitionProp | undefined): [TransitionFunction, unknown] =>
    Array.isArray(given) ? given : [given ?? still, undefined];
  const [enter, enterParams] = $derived(split(props.in));
  const [leave, leaveParams] = $derived(split(props.out));
</script>

<g-view look={encode(props.style)} in:enter={enterParams} out:leave={leaveParams}>
  {@render props.children?.()}
</g-view>
