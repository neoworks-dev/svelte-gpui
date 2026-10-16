<!-- A switch whose knob slides: `translate` eases with a Svelte tween. -->
<script lang="ts">
  import { Tween } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  import { colors } from "../theme";

  let { on = $bindable(false), label }: { on?: boolean; label: string } = $props();

  const knob = Tween.of(() => (on ? 16 : 0), { duration: 160, easing: cubicOut });
</script>

<Button style={{ gap: 10 }} onpress={() => (on = !on)}>
  <View
    style={{
      width: 36,
      height: 20,
      radius: 10,
      padding: 2,
      background: on ? colors.accent : colors.strongBorder,
      transition: 160,
    }}
  >
    <View style={{ size: 16, radius: 8, background: "#ffffff", translate: [knob.current, 0] }} />
  </View>
  <Text>{label}</Text>
</Button>
