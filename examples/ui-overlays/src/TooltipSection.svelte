<!-- Tooltips: a short label, and a rich card that stays open while the pointer is on it. -->
<script lang="ts">
  import { View } from "@neoworks/svelte-gpui/ui";
  import { Button, Tooltip } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";
</script>

<Section title="Tooltip">
  <View style={{ gap: 16, align: "start" }}>
    <Tooltip text="Moves the file to the bin" delay={100}>
      <Button>Hover for a tip</Button>
    </Tooltip>
    <Tooltip title="Sync paused" text="Photos upload again once you are back on Wi-Fi." placement="bottom-start" delay={100}>
      <Button>Hover for details</Button>
      {#snippet actions()}
        <Button size="sm" variant="primary">Sync now</Button>
        <Button size="sm" variant="ghost">Settings</Button>
      {/snippet}
    </Tooltip>
  </View>
</Section>
