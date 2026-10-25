<!-- Selects in each variant and state. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import globeIcon from "@phosphor-icons/core/regular/globe.svg";
  import houseIcon from "@phosphor-icons/core/regular/house.svg";
  import mapPinIcon from "@phosphor-icons/core/regular/map-pin.svg";
  import { Select, currentTheme, typeStyle, type SelectOption, type SelectValue } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());
  const fruits: SelectOption[] = [
    { value: "apple", label: "Apple" },
    { value: "banana", label: "Banana" },
    { value: "cherry", label: "Cherry", disabled: true },
    { value: "damson", label: "Damson" },
    { value: "elderberry", label: "Elderberry" },
  ];
  const places: SelectOption[] = [
    { value: "home", label: "Home", icon: houseIcon, description: "Your own library" },
    { value: "shared", label: "Shared", icon: globeIcon, description: "Albums others added you to" },
    { value: "nearby", label: "Nearby", icon: mapPinIcon, description: "Photos taken within 5 km" },
  ];
  const manyOptions: SelectOption[] = Array.from({ length: 24 }, (_, index) => ({ value: `item-${index}`, label: `Option number ${index + 1}` }));

  let compact = $state<SelectValue>("banana");
  let small = $state<SelectValue>("");
  let ghost = $state<SelectValue>("apple");
  let outlined = $state<SelectValue>("");
  let filled = $state<SelectValue>("damson");
  let invalid = $state<SelectValue>("");
  let several = $state<SelectValue>(["apple", "damson"]);
  let place = $state<SelectValue>("home");
  let iconOnly = $state<SelectValue>("shared");
  let long = $state<SelectValue>("");
  let locked = $state<SelectValue>("apple");

  function containsQuery(option: SelectOption, query: string): boolean {
    return option.label.toLowerCase().includes(query.toLowerCase());
  }
</script>

<View style={{ flex: 1, minWidth: 0, gap: 14 }}>
  <Section title="Select">
    <View style={{ direction: "row", gap: 14, wrap: true, align: "start" }}>
      <Select bind:value={compact} options={fruits} style={{ width: 200 }} />
      <Select bind:value={small} options={fruits} size="sm" placeholder="Small, pick a fruit" style={{ width: 200 }} />
      <Select bind:value={ghost} options={fruits} variant="ghost" style={{ width: 160 }} />
      <Select bind:value={locked} options={fruits} disabled style={{ width: 160 }} />
    </View>
    <View style={{ direction: "row", gap: 14, wrap: true, align: "start" }}>
      <Select bind:value={outlined} options={fruits} variant="outlined" label="Favourite fruit" style={{ width: 260 }} supportingText="Pick the one you like most." />
      <Select bind:value={filled} options={fruits} variant="filled" label="Other fruit" style={{ width: 260 }} />
      <Select bind:value={invalid} options={fruits} variant="filled" label="Required fruit" error supportingText="Choose a fruit to continue." style={{ width: 260 }} />
    </View>
    <View style={{ direction: "row", gap: 14, wrap: true, align: "start" }}>
      <Select bind:value={several} options={fruits} multiple variant="outlined" label="Several fruits" style={{ width: 260 }} />
      <Select bind:value={place} options={places} filter={containsQuery} searchPlaceholder="Find a place" style={{ width: 260 }} />
      <Select bind:value={iconOnly} options={places} iconOnly variant="ghost" size="sm" style={{ width: 64 }} />
      <Select bind:value={long} options={manyOptions} placeholder="Long list" style={{ width: 200 }} />
    </View>
    <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.muted }}>Chosen: {String(compact)}, {String(several)}</Text>
  </Section>
</View>
