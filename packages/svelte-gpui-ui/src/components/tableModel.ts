export interface TableColumn {
  /** The key of the value to show from each row. */
  key: string;
  /** The heading of the column. */
  label: string;
  /** Which side the column's text sits on. Numeric columns sit on the end. */
  align?: "start" | "end";
  /** Right-aligns the values, as numbers are read from their last digit. */
  numeric?: boolean;
  /** A fixed width in pixels. Columns without one share the rest in proportion to their longest text. */
  width?: number;
}

export type TableRow = Record<string, string | number>;

/** A column's text never counts as shorter or longer than this, so a column of single digits keeps room to breathe. */
const shortestWeightedLength = 4;
const longestWeightedLength = 28;

export function isEndAligned(column: TableColumn): boolean {
  return column.numeric === true || column.align === "end";
}

export function cellText(row: TableRow, column: TableColumn): string {
  const value = row[column.key];
  if (value === undefined) {
    return "";
  }
  return String(value);
}

function longestTextLength(column: TableColumn, rows: TableRow[]): number {
  const lengths = rows.map((row) => cellText(row, column).length);
  return Math.max(column.label.length, ...lengths);
}

/**
 * How much of the free width each column takes, in the order of `columns`, or undefined for a column of fixed
 * width. Rows are laid out one by one, so a column's width is decided here and not by the cells it holds.
 */
export function columnWeights(columns: TableColumn[], rows: TableRow[]): (number | undefined)[] {
  return columns.map((column) => {
    if (column.width !== undefined) {
      return undefined;
    }
    const length = longestTextLength(column, rows);
    return Math.min(longestWeightedLength, Math.max(shortestWeightedLength, length));
  });
}
