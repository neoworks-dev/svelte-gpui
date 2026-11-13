<!--
  Linear gradients as a `background`: the four sides and an angle, stops that do not span the box, a fade to
  `transparent` over a page colour, a tall low-contrast wash, and a gradient that eases between two colours on hover.
-->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";

  const tile = { width: 120, height: 80, radius: 8, padding: 8, color: "#ffffff" };
</script>

<View style={{ padding: 16, gap: 12, background: "#111114", color: "#a1a1aa" }}>
  <View style={{ direction: "row", gap: 12 }}>
    <View style={{ ...tile, background: { angle: "to bottom", from: "#ff3366", to: "#3366ff" } }}>
      <Text>to bottom</Text>
    </View>
    <View style={{ ...tile, background: { angle: "to top", from: "#ff3366", to: "#3366ff" } }}>
      <Text>to top</Text>
    </View>
    <View style={{ ...tile, background: { angle: "to right", from: "#ff3366", to: "#3366ff" } }}>
      <Text>to right</Text>
    </View>
    <View style={{ ...tile, background: { angle: "to left", from: "#ff3366", to: "#3366ff" } }}>
      <Text>to left</Text>
    </View>
    <View style={{ ...tile, background: { angle: 45, from: "#ff3366", to: "#3366ff" } }}>
      <Text>45 degrees</Text>
    </View>
  </View>

  <View style={{ direction: "row", gap: 12 }}>
    <View
      style={{
        ...tile,
        width: 180,
        background: { angle: "to right", from: { color: "#22cc88", at: 0.3 }, to: { color: "#aa22cc", at: 0.7 } },
      }}
    >
      <Text>stops at 0.3 and 0.7</Text>
    </View>
    <View style={{ ...tile, width: 180, background: { from: "#ff8800", to: "transparent" } }}>
      <Text>orange to transparent</Text>
    </View>
    <View
      style={{
        ...tile,
        width: 180,
        background: "#222228",
        transition: 1500,
      }}
      hover={{ background: { angle: "to right", from: "#ff3366", to: "#3366ff" } }}
    >
      <Text>solid to gradient on hover</Text>
    </View>
  </View>

  <View style={{ height: 300, radius: 8, padding: 12, background: { from: "#5b3a8c", to: "#111114" } }}>
    <Text>A tall, low-contrast wash</Text>
  </View>
</View>
