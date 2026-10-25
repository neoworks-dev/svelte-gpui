export interface SearchSuggestion {
  /** Text put into the search field when the suggestion is chosen. */
  value: string;
  /** What is shown. Defaults to `value`. */
  label?: string;
  /** SVG markup of an icon before the label. */
  icon?: string;
  supportingText?: string;
}

export interface SegmentedOption {
  value: string;
  /** Text of the segment. Leave it out for an icon-only segment. */
  label?: string;
  /** SVG markup of the segment's icon. */
  icon?: string;
  /** SVG markup of the icon while the segment is on, such as the filled weight of `icon`. */
  selectedIcon?: string;
  disabled?: boolean;
}

export interface TabItem {
  id: string;
  label: string;
  /** SVG markup drawn above the label of a primary tab. */
  icon?: string;
  /** SVG markup drawn while the tab is active, such as the filled weight of `icon`. */
  selectedIcon?: string;
  /** A count shown after the label, such as unread items. */
  count?: number;
  disabled?: boolean;
}
