// The values and options of a select, apart from how it is drawn.

export interface SelectOption {
  value: string;
  label: string;
  /** SVG markup of an icon before the label. */
  icon?: string;
  /** A second line under the label. */
  description?: string;
  disabled?: boolean;
}

export type SelectValue = string | string[];

export type SelectFilter = (option: SelectOption, query: string) => boolean;

/** The chosen values as a list, whether the select holds one value or several. */
export function selectedValuesOf(value: SelectValue): string[] {
  if (Array.isArray(value)) {
    return value;
  }
  if (value === "") {
    return [];
  }
  return [value];
}

/** The options that pass `filter` for the typed `query`. Every option shows while there is no filter or no query. */
export function visibleOptionsOf(options: SelectOption[], filter: SelectFilter | undefined, query: string): SelectOption[] {
  if (filter === undefined || query.trim() === "") {
    return options;
  }
  return options.filter((option) => filter(option, query));
}

/** The values after picking `picked` in a select that allows several. They stay in the order of `options`. */
export function toggledValues(options: SelectOption[], chosen: string[], picked: string): string[] {
  const next = new Set(chosen);
  if (next.has(picked)) {
    next.delete(picked);
  } else {
    next.add(picked);
  }
  return options.filter((option) => next.has(option.value)).map((option) => option.value);
}

/** The text on the trigger: the chosen labels, or the placeholder while nothing is chosen. */
export function triggerTextOf(selectedOptions: SelectOption[], placeholder: string): string {
  if (selectedOptions.length === 0) {
    return placeholder;
  }
  return selectedOptions.map((option) => option.label).join(", ");
}

/** Distance from the top of the list to the top of row `index`, given each row's height. */
export function rowOffset(rowHeights: number[], index: number): number {
  return rowHeights.slice(0, index).reduce((sum, height) => sum + height, 0);
}

/** The scroll position that brings row `index` into view with the least movement, or `undefined` if it already is. */
export function scrollToReveal(rowHeights: number[], index: number, scrollTop: number, viewportHeight: number): number | undefined {
  const rowTop = rowOffset(rowHeights, index);
  const rowBottom = rowTop + rowHeights[index];
  if (rowTop < scrollTop) {
    return rowTop;
  }
  if (rowBottom > scrollTop + viewportHeight) {
    return rowBottom - viewportHeight;
  }
  return undefined;
}
