<!--
  `scale`: a card painted at 1.3 with its text, icon, border and shadow, next to the same card at 1 for comparison;
  a button that gives under the press and springs back (`active={{ scale }}` with `easing: "spring"`); and a
  heart that pops when pressed. The `slow` switch stretches the spring over two seconds, to see it pass.
-->
<script lang="ts">
  import { Button, Icon, Text, View } from "@neoworks/svelte-gpui/ui";
  import heart from "@phosphor-icons/core/fill/heart-fill.svg";

  const slow = true;
  const springMs = slow ? 2000 : 420;
  const spring = { duration: springMs, easing: "spring" } as const;

  let popped = $state(false);

  const card = {
    direction: "row",
    align: "center",
    gap: 8,
    paddingX: 14,
    paddingY: 10,
    radius: 10,
    background: "#26262c",
    borderWidth: 1,
    borderColor: "#4a4a55",
    shadow: { y: 6, blur: 16, color: "rgba(0, 0, 0, 0.5)" },
  } as const;

  function pop(): void {
    popped = true;
    setTimeout(() => (popped = false), 120);
  }
</script>

<View style={{ height: 420, padding: 32, gap: 40, background: "#111114", color: "#e4e4e7" }}>
  <View style={{ direction: "row", gap: 64, align: "center" }}>
    <View style={card}>
      <Icon svg={heart} size={16} color="#f43f5e" />
      <Text>Scale 1</Text>
    </View>
    <View style={{ ...card, scale: 1.3 }}>
      <Icon svg={heart} size={16} color="#f43f5e" />
      <Text>Scale 1.3</Text>
    </View>
  </View>

  <Button
    style={{ ...card, alignSelf: "start", transition: spring }}
    active={{ scale: 0.85, background: "#33333b" }}
  >
    <Text>Hold me</Text>
  </Button>

  <Button style={{ alignSelf: "start", transition: spring, scale: popped ? 1.35 : 1 }} onpress={pop}>
    <Icon svg={heart} size={28} color="#f43f5e" />
  </Button>
</View>
