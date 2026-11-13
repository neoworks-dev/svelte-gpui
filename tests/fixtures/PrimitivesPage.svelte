<!-- An app that uses the primitives that `definePrimitive` makes, for tests/primitives.test.ts. -->
<script lang="ts">
  import { Button, Icon, Image, Overlay, Text, View } from "@neoworks/svelte-gpui/ui";

  let width = $state(100);
  let withPress = $state(true);
  let disabled = $state(false);
  let showButton = $state(true);
  let size = $state(16);
  let pressed = $state(0);
  let loaded = $state(0);
  let view = $state<{ focus(): void; blur(): void }>();

  // What the test drives, since nothing can click here.
  (globalThis as any).__page = {
    setWidth: (value: number) => (width = value),
    setPress: (value: boolean) => (withPress = value),
    setDisabled: (value: boolean) => (disabled = value),
    setSize: (value: number) => (size = value),
    hideButton: () => (showButton = false),
    focusView: () => view!.focus(),
    pressed: () => pressed,
    loaded: () => loaded,
  };

  function countPress(): void {
    pressed += 1;
  }
</script>

<View bind:this={view} style={{ width }} hover={{ background: "#fff" }} focusable onpress={withPress ? countPress : undefined}>
  <Text pre style={{ fontSize: 12 }}>hello</Text>
  {#if showButton}
    <Button {disabled} disabledStyle={{ opacity: 0.5 }} onpress={countPress} onhover={(hovered) => (loaded += hovered ? 1 : 0)}>
      <Text>press me</Text>
    </Button>
  {/if}
  <Icon svg="<svg/>" {size} color="#abc" />
  <Image src="cover.png" fit="cover" onload={() => (loaded += 1)} />
  <Overlay side="top" offset={4} at={[1, 2]} matchWidth />
</View>
