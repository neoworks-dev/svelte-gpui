<!--
  Rows that ease over two seconds when the pointer is over them, slow enough to see where they pass through. Each is
  a way an app asks for a transition: a background that comes from a `hover` style, one the app switches itself
  from `onhover`, a text colour that only the `hover` style names (the base colour is inherited), and a
  background that starts out `transparent`.
-->
<script lang="ts">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";

  const duration = 2000;
  let over = $state(false);
</script>

<View style={{ height: 500, padding: 16, gap: 8, background: "#111114", color: "#777777" }}>
  <View
    style={{ padding: 12, radius: 6, background: "#222228", transition: duration }}
    hover={{ background: "#2255ff" }}
  >
    <Text>Background from a hover style</Text>
  </View>
  <View
    style={{ padding: 12, radius: 6, background: over ? "#2255ff" : "#222228", transition: duration }}
    onhover={(hovered: boolean) => (over = hovered)}
  >
    <Text>Background switched by the app</Text>
  </View>
  <Button style={{ padding: 12, radius: 6, background: "#222228", transition: duration }} hover={{ color: "#ffffff" }}>
    <Text>Text colour from a hover style, base inherited</Text>
  </Button>
  <View
    style={{ padding: 12, radius: 6, background: "transparent", transition: duration }}
    hover={{ background: "#ffffff" }}
  >
    <Text>Transparent to white</Text>
  </View>
</View>
