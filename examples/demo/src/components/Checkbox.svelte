<!-- A checkbox, built from primitives: there is no native one. -->
<script lang="ts">
  import { Button, Icon, Text, View } from "@neoworks/svelte-gpui/ui";
  import check from "@phosphor-icons/core/bold/check-bold.svg";
  import { colors } from "../theme";

  let { checked = $bindable(false), label }: { checked?: boolean; label: string } = $props();
</script>

<Button
  style={{ gap: 8, radius: 4 }}
  focus={{ background: colors.raised }}
  onpress={() => (checked = !checked)}
>
  <View
    style={{
      size: 16,
      radius: 4,
      borderWidth: 1,
      borderColor: checked ? colors.accent : colors.strongBorder,
      background: checked ? colors.accent : colors.window,
      align: "center",
      justify: "center",
      transition: 120,
    }}
  >
    {#if checked}<Icon svg={check} size={10} color="#ffffff" />{/if}
  </View>
  <Text>{label}</Text>
</Button>
