<!-- Table, StatusBadge and IconCircle. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import {
    currentTheme,
    IconCircle,
    StatusBadge,
    Table,
    typeStyle,
    type StatusTone,
    type TableColumn,
    type TableRow,
  } from "@neoworks/svelte-gpui-ui";
  import checkCircle from "@phosphor-icons/core/bold/check-circle-bold.svg";
  import clock from "@phosphor-icons/core/bold/clock-bold.svg";
  import info from "@phosphor-icons/core/bold/info-bold.svg";
  import warning from "@phosphor-icons/core/bold/warning-bold.svg";
  import xCircle from "@phosphor-icons/core/bold/x-circle-bold.svg";
  import folder from "@phosphor-icons/core/bold/folder-bold.svg";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  const columns: TableColumn[] = [
    { key: "plan", label: "Plan" },
    { key: "storage", label: "Storage", align: "end" },
    { key: "price", label: "Price per month", numeric: true },
  ];
  const rows: TableRow[] = [
    { plan: "Free", storage: "5 GB", price: "0.00" },
    { plan: "Personal", storage: "200 GB", price: "2.99" },
    { plan: "Family with six members", storage: "2 TB", price: "9.99" },
  ];
  const fixedColumns: TableColumn[] = [
    { key: "name", label: "Name", width: 160 },
    { key: "size", label: "Size", numeric: true, width: 90 },
    { key: "kind", label: "Kind" },
  ];
  const fixedRows: TableRow[] = [
    { name: "holiday.jpg", size: 4210, kind: "Image" },
    { name: "report.pdf", size: 312, kind: "Document" },
    { name: "song.flac", size: 28104, kind: "Audio" },
    { name: "notes.txt", size: 2, kind: "Text" },
  ];

  const tones: StatusTone[] = ["success", "warning", "danger", "info", "neutral"];
  const legacyTones: StatusTone[] = ["green", "amber", "red", "blue", "violet"];
  const toneIcons = [checkCircle, warning, xCircle, info, clock];
</script>

<View style={{ gap: 18 }}>
  <Section title="Table, comfortable with a row header">
    <Table {columns} {rows} rowHeader caption="Prices include tax." />
  </Section>

  <Section title="Table, compact with fixed widths">
    <Table columns={fixedColumns} rows={fixedRows} density="compact" caption="Sizes in kilobytes." />
  </Section>

  <Section title="Status badge">
    <View style={{ direction: "row", gap: 8, wrap: true, align: "center" }}>
      {#each tones as tone (tone)}
        <StatusBadge {tone}>{tone}</StatusBadge>
      {/each}
    </View>
    <View style={{ direction: "row", gap: 8, wrap: true, align: "center" }}>
      {#each tones as tone (tone)}
        <StatusBadge {tone} dot>{tone}</StatusBadge>
      {/each}
    </View>
    <View style={{ direction: "row", gap: 8, wrap: true, align: "center" }}>
      {#each tones as tone, index (tone)}
        <StatusBadge {tone} icon={toneIcons[index]}>{tone}</StatusBadge>
      {/each}
    </View>
    <View style={{ direction: "row", gap: 8, wrap: true, align: "center" }}>
      {#each legacyTones as tone (tone)}
        <StatusBadge {tone}>{tone}</StatusBadge>
      {/each}
    </View>
  </Section>

  <Section title="Icon circle, small, medium and large">
    {#each [tones, legacyTones] as group, groupIndex (groupIndex)}
      <View style={{ direction: "row", gap: 10, align: "center", wrap: true }}>
        {#each group as tone (tone)}
          <IconCircle icon={folder} {tone} size="sm" />
          <IconCircle icon={folder} {tone} />
          <IconCircle icon={folder} {tone} size="lg" />
        {/each}
      </View>
    {/each}
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>The second row uses the older colour names.</Text>
  </Section>
</View>
