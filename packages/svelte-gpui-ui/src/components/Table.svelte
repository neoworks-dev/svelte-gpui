<!--
  A table of text, with a heading row and a line between rows. Rows are laid out one by one, so columns share the
  width in proportion to their longest text; give a column a `width` to fix it.
-->
<script lang="ts">
  import { Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import {
    tableBodyRowLook,
    tableCaptionLook,
    tableCellBoxLook,
    tableCellTextLook,
    tableHeaderRowLook,
    tableHeaderTextLook,
    tableLook,
    type TableDensity,
  } from "./table-look";
  import { cellText, columnWeights, type TableColumn, type TableRow } from "./tableModel";

  interface Props {
    /** The columns, in the order they are shown. */
    columns: TableColumn[];
    /** One object per row, holding a value for each column key. */
    rows: TableRow[];
    /** A short line under the table that says what it shows. */
    caption?: string;
    /** `compact` tightens the rows for tables with many of them. */
    density?: TableDensity;
    /** Makes the first column the label of each row, in a heavier weight. */
    rowHeader?: boolean;
    /** Layout only, such as width or margins. */
    style?: Style;
  }

  let { columns, rows, caption, density = "comfortable", rowHeader = false, style }: Props = $props();

  const theme = $derived(currentTheme());
  const weights = $derived(columnWeights(columns, rows));

  function cellOptions(column: TableColumn, columnIndex: number) {
    return { column, weight: weights[columnIndex], columnIndex, columnCount: columns.length, density };
  }
</script>

<View style={{ ...tableLook(theme), ...style }}>
  <View style={tableHeaderRowLook(theme)}>
    {#each columns as column, columnIndex (column.key)}
      <View style={tableCellBoxLook(cellOptions(column, columnIndex))}>
        <Text style={tableHeaderTextLook(theme, column)}>{column.label}</Text>
      </View>
    {/each}
  </View>
  {#each rows as row, rowIndex (rowIndex)}
    <View style={tableBodyRowLook(theme, rowIndex)}>
      {#each columns as column, columnIndex (column.key)}
        <View style={tableCellBoxLook(cellOptions(column, columnIndex))}>
          <Text style={tableCellTextLook(theme, column, rowHeader && columnIndex === 0)}>{cellText(row, column)}</Text>
        </View>
      {/each}
    </View>
  {/each}
  {#if caption}
    <Text style={tableCaptionLook(theme)}>{caption}</Text>
  {/if}
</View>
