<!-- Time pickers and wheel columns. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, TimePicker, typeStyle, WheelColumn } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());
  const hours = Array.from({ length: 24 }, (_, index) => index);
  const years = Array.from({ length: 2100 - 1900 + 1 }, (_, index) => 1900 + index);

  let twentyFourHourTime = $state("14:30");
  let twelveHourTime = $state("09:05");
  let invalidTime = $state("00:00");
  let wheelHour = $state(7);
  let wheelYear = $state(2026);
</script>

<View style={{ gap: 18 }}>
  <Section title="Time fields">
    <TimePicker value={twentyFourHourTime} onChange={(time) => (twentyFourHourTime = time)} label="Wake-up time" />
    <TimePicker value={twelveHourTime} onChange={(time) => (twelveHourTime = time)} label="Start time" twelveHour minuteStep={5} supportingText="In your own time zone." />
    <TimePicker value={invalidTime} onChange={(time) => (invalidTime = time)} label="Closing time" error supportingText="Closing time must be after opening time." />
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>
      Chosen: {twentyFourHourTime}, {twelveHourTime}, {invalidTime}
    </Text>
  </Section>
  <Section title="Wheel columns">
    <View style={{ direction: "row", gap: 24 }}>
      <View style={{ width: 96 }}>
        <WheelColumn values={hours} value={wheelHour} onChange={(hour) => (wheelHour = hour)} label="Hours" />
      </View>
      <View style={{ width: 96 }}>
        <WheelColumn values={years} value={wheelYear} onChange={(year) => (wheelYear = year)} wrap={false} label="Year" />
      </View>
    </View>
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>Chosen: {wheelHour}, {wheelYear}</Text>
  </Section>
</View>
