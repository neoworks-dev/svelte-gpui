<!-- Checkboxes, radios and switches. -->
<script lang="ts">
  import { View } from "@neoworks/svelte-gpui/ui";
  import { Checkbox, Radio, RadioGroup, Switch } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  let unchecked = $state(false);
  let checked = $state(true);
  let partly = $state(false);
  let partlyIndeterminate = $state(true);
  let terms = $state(false);
  let plan = $state<string | undefined>("pro");
  let size = $state<string | undefined>("m");
  let delivery = $state<string | undefined>(undefined);
  let locked = $state<string | undefined>("a");
  let off = $state(false);
  let on = $state(true);
  let withIcon = $state(true);
  let described = $state(false);
</script>

<View style={{ gap: 18 }}>
  <Section title="Checkbox">
    <View style={{ direction: "row", gap: 16, wrap: true }}>
      <Checkbox inline bind:checked={unchecked} label="Unchecked" />
      <Checkbox inline bind:checked label="Checked" />
      <Checkbox inline bind:checked={partly} bind:indeterminate={partlyIndeterminate} label="Indeterminate" />
    </View>
    <Checkbox bind:checked={terms} error label="Error" supportingText="Accept the terms to continue." />
    <View style={{ direction: "row", gap: 16, wrap: true }}>
      <Checkbox inline checked label="Small" size="sm" />
      <Checkbox inline disabled label="Disabled" />
      <Checkbox inline disabled checked label="Disabled, checked" />
    </View>
  </Section>
  <Section title="Radio">
    <View style={{ direction: "row", gap: 32, wrap: true }}>
      <RadioGroup bind:value={plan} label="Plan" style={{ width: 220 }} supportingText="Billed monthly.">
        <Radio value="free" label="Free" />
        <Radio value="pro" label="Pro" supportingText="Everything in Free, plus sync." />
        <Radio value="team" label="Team" disabled />
      </RadioGroup>
      <RadioGroup bind:value={delivery} label="Delivery" style={{ width: 160 }} error supportingText="Choose one option.">
        <Radio value="post" label="Post" />
        <Radio value="pickup" label="Pickup" />
      </RadioGroup>
    </View>
    <RadioGroup bind:value={size} label="Size" orientation="horizontal">
      <Radio value="s" label="S" />
      <Radio value="m" label="M" />
      <Radio value="l" label="L" />
    </RadioGroup>
    <RadioGroup bind:value={locked} label="Whole group disabled" disabled orientation="horizontal">
      <Radio value="a" label="Chosen" />
      <Radio value="b" label="Other" />
    </RadioGroup>
  </Section>
  <Section title="Switch">
    <View style={{ gap: 6 }}>
      <Switch bind:checked={off} label="Off" />
      <Switch bind:checked={on} label="On" />
      <Switch bind:checked={withIcon} label="On with icon" showIcon />
      <Switch bind:checked={described} label="Sync over mobile data" supportingText="Uses your data plan when Wi-Fi is not available." />
      <Switch disabled label="Disabled, off" />
      <Switch disabled checked label="Disabled, on" />
    </View>
  </Section>
</View>
