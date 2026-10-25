<!-- Progress indicators and spinners. -->
<script lang="ts">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, LoadingSpinner, ProgressIndicator, typeStyle } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  let upload = $state(35);

  function advance(): void {
    upload = Math.min(100, upload + 30);
  }
</script>

<View style={{ gap: 18 }}>
  <Section title="Linear">
    <ProgressIndicator label="Empty" value={0} />
    <ProgressIndicator label="A quarter" value={25} />
    <ProgressIndicator label="Uploading 12 photos" value={upload} />
    <ProgressIndicator label="Done" value={100} tone="accent" />
    <ProgressIndicator label="Counting" value={3} max={8} />
    <ProgressIndicator label="Waiting" />
    <ProgressIndicator label="Waiting, accent" tone="accent" />
    <Button
      onpress={advance}
      style={{ alignSelf: "start", paddingX: 14, paddingY: 8, radius: theme.radii.control, background: theme.colors.sunken, cursor: "pointer" }}
    >
      <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.text }}>Upload more</Text>
    </Button>
  </Section>
  <Section title="Circular">
    <View style={{ direction: "row", align: "center", gap: 20 }}>
      <ProgressIndicator variant="circular" label="Empty" value={0} />
      <ProgressIndicator variant="circular" label="A quarter" value={25} />
      <ProgressIndicator variant="circular" label="Sixty" value={60} />
      <ProgressIndicator variant="circular" label="Done" value={100} tone="accent" />
    </View>
    <View style={{ direction: "row", align: "center", gap: 20 }}>
      <ProgressIndicator variant="circular" label="Small" value={70} size={24} thickness={6} />
      <ProgressIndicator variant="circular" label="Large" value={70} size={72} thickness={3} />
      <ProgressIndicator variant="circular" label="Waiting" />
      <ProgressIndicator variant="circular" label="Waiting, accent" tone="accent" size={64} />
    </View>
  </Section>
  <Section title="Loading spinner">
    <View style={{ direction: "row", align: "center", gap: 20 }}>
      <LoadingSpinner size={16} />
      <LoadingSpinner />
      <LoadingSpinner size={40} thickness={4} />
      <LoadingSpinner color={theme.colors.accent} />
      <LoadingSpinner color={theme.colors.muted} size={32} />
    </View>
  </Section>
</View>
