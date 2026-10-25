<!-- Slider, search and segmented button. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, Search, SegmentedButton, Slider, typeStyle, type SearchSuggestion } from "@neoworks/svelte-gpui-ui";
  import clock from "@phosphor-icons/core/regular/clock-counter-clockwise.svg";
  import speaker from "@phosphor-icons/core/regular/speaker-high.svg";
  import list from "@phosphor-icons/core/regular/list.svg";
  import listFill from "@phosphor-icons/core/fill/list-fill.svg";
  import grid from "@phosphor-icons/core/regular/squares-four.svg";
  import gridFill from "@phosphor-icons/core/fill/squares-four-fill.svg";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  let query = $state("");
  let smallQuery = $state("neo");
  let submitted = $state("nothing yet");
  let volume = $state(35);
  let quiet = $state(20);
  let stepped = $state(3);
  let span = $state<[number, number]>([20, 70]);
  let view = $state<string | undefined>("list");
  let timeframe = $state<string | undefined>("week");
  let days = $state<string[]>(["mon", "wed"]);
  let fullWidth = $state<string | undefined>("b");
  let small = $state<string | undefined>("a");

  const suggestions: SearchSuggestion[] = [
    { value: "neoworks design system", icon: clock },
    { value: "neoworks photos", icon: clock, supportingText: "In Photos" },
    { value: "neoworks calendar", label: "Neoworks Calendar", icon: clock },
    { value: "neoworks maps" },
  ];
</script>

<View style={{ gap: 18 }}>
  <Section title="Search">
    <Search bind:value={query} placeholder="Search the library" {suggestions} heading="Recent searches" onsubmit={(text) => (submitted = text)} />
    <Search bind:value={smallQuery} placeholder="Small search" size="sm" />
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>Submitted: {submitted}</Text>
  </Section>

  <Section title="Slider">
    <Slider bind:value={volume} icon={speaker} showValue formatValue={(amount) => `${amount}%`} />
    <Slider bind:value={stepped} min={0} max={10} step={1} showTicks />
    <Slider bind:rangeValue={span} range showValue />
    <Slider bind:value={quiet} disabled />
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>
      Volume {volume}, steps {stepped}, range {span[0]} to {span[1]}
    </Text>
  </Section>

  <Section title="Segmented button">
    <SegmentedButton
      bind:value={view}
      options={[
        { value: "list", label: "List", icon: list, selectedIcon: listFill },
        { value: "grid", label: "Grid", icon: grid, selectedIcon: gridFill },
        { value: "table", label: "Table" },
      ]}
    />
    <SegmentedButton
      bind:value={timeframe}
      size="sm"
      options={[
        { value: "day", label: "Day" },
        { value: "week", label: "Week" },
        { value: "month", label: "Month", disabled: true },
        { value: "year", label: "Year" },
      ]}
    />
    <SegmentedButton
      multiple
      bind:values={days}
      options={[
        { value: "mon", label: "Mon" },
        { value: "tue", label: "Tue" },
        { value: "wed", label: "Wed" },
        { value: "thu", label: "Thu" },
      ]}
    />
    <SegmentedButton
      full
      bind:value={fullWidth}
      options={[
        { value: "a", label: "Full width" },
        { value: "b", label: "Equal segments" },
        { value: "c", label: "Stretched" },
      ]}
    />
    <SegmentedButton
      bind:value={small}
      size="sm"
      options={[
        { value: "a", icon: list, selectedIcon: listFill },
        { value: "b", icon: grid, selectedIcon: gridFill },
      ]}
    />
  </Section>
</View>
