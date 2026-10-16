// What a tree tells the snippet that draws a row.

/** What the row's snippet is told about the row it draws. */
export interface TreeRow {
  /** How many branches above it, 0 for the top level. */
  depth: number;
  /** Its place among the rows now showing. */
  index: number;
  /** It can be expanded. */
  branch: boolean;
  expanded: boolean;
  selected: boolean;
  /** Expands or collapses it. */
  toggle(): void;
  /** Selects it. */
  select(): void;
}
