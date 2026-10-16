<script lang="ts" generics="T extends string">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  import { colors } from "../theme";

  let { options, value = $bindable() }: { options: T[]; value: T } = $props();
</script>

<View
  style={{
    direction: "row",
    padding: 2,
    gap: 2,
    radius: 7,
    background: colors.window,
    borderWidth: 1,
    borderColor: colors.border,
    alignSelf: "start",
  }}
>
  {#each options as option (option)}
    {@const chosen = option === value}
    <Button
      style={{
        paddingX: 10,
        paddingY: 4,
        radius: 5,
        background: chosen ? colors.raised : "transparent",
        color: chosen ? colors.text : colors.dim,
        transition: 120,
      }}
      hover={chosen ? undefined : { color: colors.text }}
      onpress={() => (value = option)}
    >
      <Text style={{ fontSize: 12 }}>{option}</Text>
    </Button>
  {/each}
</View>
