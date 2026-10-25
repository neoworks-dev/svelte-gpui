import type { Style } from "@neoworks/svelte-gpui/ui";
import { typeStyle, type Theme } from "../theme";
import { isEndAligned, type TableColumn } from "./tableModel";

export type TableDensity = "comfortable" | "compact";

const cellPaddingY: Record<TableDensity, number> = { comfortable: 12, compact: 8 };
const cellPaddingX = 16;

export function tableLook(theme: Theme): Style {
  return { direction: "column", color: theme.colors.text };
}

export function tableHeaderRowLook(theme: Theme): Style {
  return { direction: "row", borderBottomWidth: 1, borderColor: theme.colors.lineStrong };
}

/** Every body row but the first has a line above it. */
export function tableBodyRowLook(theme: Theme, rowIndex: number): Style {
  if (rowIndex === 0) {
    return { direction: "row" };
  }
  return { direction: "row", borderTopWidth: 1, borderColor: theme.colors.line };
}

function columnSizeStyle(weight: number | undefined, column: TableColumn): Style {
  if (weight === undefined) {
    return { width: column.width, shrink: 0 };
  }
  return { flex: weight, minWidth: 0 };
}

/** The first column starts at the table's edge and the last ends at it, so the table lines up with the page. */
function cellPadding(columnIndex: number, columnCount: number, density: TableDensity): Style {
  const style: Style = { paddingY: cellPaddingY[density], paddingLeft: cellPaddingX, paddingRight: cellPaddingX };
  if (columnIndex === 0) {
    style.paddingLeft = 0;
  }
  if (columnIndex === columnCount - 1) {
    style.paddingRight = 0;
  }
  return style;
}

export interface TableCellOptions {
  column: TableColumn;
  weight: number | undefined;
  columnIndex: number;
  columnCount: number;
  density: TableDensity;
}

export function tableCellBoxLook(options: TableCellOptions): Style {
  return {
    ...columnSizeStyle(options.weight, options.column),
    ...cellPadding(options.columnIndex, options.columnCount, options.density),
  };
}

function alignedText(column: TableColumn): Style {
  if (isEndAligned(column)) {
    return { textAlign: "right" };
  }
  return { textAlign: "left" };
}

export function tableHeaderTextLook(theme: Theme, column: TableColumn): Style {
  return { ...typeStyle("caption"), fontWeight: 500, color: theme.colors.muted, nowrap: true, ...alignedText(column) };
}

export function tableCellTextLook(theme: Theme, column: TableColumn, isRowHeader: boolean): Style {
  const style: Style = { ...typeStyle("bodySmall"), color: theme.colors.text, ...alignedText(column) };
  if (isRowHeader) {
    style.fontWeight = 500;
  }
  return style;
}

export function tableCaptionLook(theme: Theme): Style {
  return { ...typeStyle("caption"), color: theme.colors.muted, paddingTop: 12 };
}
