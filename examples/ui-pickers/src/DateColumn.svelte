<!-- Date pickers in each variant and state. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, DatePicker, typeStyle } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  let titleDate: Date | null = $state(new Date(2026, 9, 14));
  let formattedDate: Date | null = $state(new Date(2027, 2, 5));
  let emptyTitleDate: Date | null = $state(null);
  let limitedDate: Date | null = $state(new Date(2026, 9, 14));
  let emptyFieldDate: Date | null = $state(null);
  let filledFieldDate: Date | null = $state(new Date(2025, 9, 14));
  let invalidFieldDate: Date | null = $state(null);

  function describe(date: Date | null): string {
    if (date === null) {
      return "none";
    }
    return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;
  }
</script>

<View style={{ gap: 18 }}>
  <Section title="Field">
    <DatePicker variant="field" fieldLabel="Date of birth" value={emptyFieldDate} onChange={(date) => (emptyFieldDate = date)} supportingText="Type it as mm/dd/yyyy." />
    <DatePicker variant="field" fieldLabel="Start date" value={filledFieldDate} onChange={(date) => (filledFieldDate = date)} />
    <DatePicker variant="field" fieldLabel="Due date" value={invalidFieldDate} onChange={(date) => (invalidFieldDate = date)} error supportingText="Choose a day after the start." />
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>
      Chosen: {describe(emptyFieldDate)}, {describe(filledFieldDate)}, {describe(invalidFieldDate)}
    </Text>
  </Section>
  <Section title="Title">
    <DatePicker value={titleDate} onChange={(date) => (titleDate = date)} label="Trip date" />
    <DatePicker value={emptyTitleDate} onChange={(date) => (emptyTitleDate = date)} />
    <DatePicker
      value={limitedDate}
      onChange={(date) => (limitedDate = date)}
      label="Between the 10th and 20th"
      weekStartsOn={0}
      min={new Date(2026, 9, 10)}
      max={new Date(2026, 9, 20)}
    />
    <DatePicker value={formattedDate} onChange={(date) => (formattedDate = date)} />
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>
      Chosen: {describe(titleDate)}, {describe(emptyTitleDate)}, {describe(limitedDate)}
    </Text>
  </Section>
</View>
