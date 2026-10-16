<!--
  A slider. Pressing the track holds the pointer: `onmove` and `onup` keep coming
  while it is dragged, even outside the track, until the button is released.
-->
<script lang="ts">
  import { Text, View, type PointerEvent } from "@neoworks/svelte-gpui/ui";
  import { colors } from "../theme";

  interface Props {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    width?: number;
  }

  let { value = $bindable(50), min = 0, max = 100, step = 1, width = 220 }: Props = $props();

  let dragging = $state(false);
  const share = $derived((value - min) / (max - min));

  function moveTo(event: PointerEvent): void {
    const raw = min + (Math.min(Math.max(event.x, 0), width) / width) * (max - min);
    value = Math.round(raw / step) * step;
  }
</script>

<View style={{ direction: "row", align: "center", gap: 12 }}>
  <View
    style={{ width, height: 20, justify: "center", cursor: "pointer" }}
    ondown={(event) => {
      dragging = true;
      moveTo(event);
    }}
    onmove={(event) => dragging && moveTo(event)}
    onup={() => (dragging = false)}
  >
    <View style={{ height: 4, radius: 2, background: colors.strongBorder }}>
      <View style={{ width: share * width, height: 4, radius: 2, background: colors.accent }} />
    </View>
    <View
      style={{
        position: "absolute",
        left: share * width - 8,
        top: 2,
        size: 16,
        radius: 8,
        background: "#ffffff",
        borderWidth: dragging ? 3 : 0,
        borderColor: colors.accent,
        shadow: { y: 1, blur: 3, color: "#00000080" },
      }}
    />
  </View>
  <Text style={{ fontFamily: "mono", fontSize: 12, color: colors.dim, width: 32 }}>{value}</Text>
</View>
