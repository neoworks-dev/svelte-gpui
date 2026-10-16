// What a menu is made of.

export interface MenuItem {
  label?: string;
  /** Called when the item is chosen, before the menu's own `onselect`. */
  onselect?: () => void;
  /** SVG markup, tinted with the label's colour. */
  icon?: string;
  /** Text shown at the end of the row, such as `"Ctrl+S"`. It does not bind the key. */
  shortcut?: string;
  disabled?: boolean;
  /** Drawn in red. */
  danger?: boolean;
  /** Shows a check mark in front of the label. */
  checked?: boolean;
  /** A line between groups instead of an item. */
  separator?: boolean;
  /** Anything the app wants back from `onselect`. */
  value?: unknown;
}
