// What a menu is made of, and the rows derived from it.

export interface MenuItem {
  type?: "item";
  label: string;
  /** SVG markup of an icon before the label. */
  icon?: string;
  /** Keyboard shortcut shown at the end of the row, such as "Ctrl+D". Display only. */
  shortcut?: string;
  disabled?: boolean;
  /** Colours the item as destructive. */
  danger?: boolean;
  /** Makes the item a checkable option: `true` shows a check, `false` leaves the space free. */
  selected?: boolean;
  onselect?: () => void;
}

export interface MenuDivider {
  type: "divider";
}

export interface MenuHeading {
  type: "heading";
  label: string;
}

export type MenuEntry = MenuItem | MenuDivider | MenuHeading;

/** What the trigger needs to open the menu. Pass `onpress` to a button. */
export interface MenuTriggerProps {
  onpress: () => void;
  /** Whether the menu is open. */
  expanded: boolean;
}

export interface MenuRow {
  entry: MenuEntry;
  /** Position among the items that can be chosen, or -1 for a divider or heading. */
  actionIndex: number;
}

export function isAction(entry: MenuEntry): entry is MenuItem {
  return entry.type !== "divider" && entry.type !== "heading";
}

export function menuRows(entries: MenuEntry[]): MenuRow[] {
  let nextActionIndex = 0;
  return entries.map((entry) => {
    if (!isAction(entry)) {
      return { entry, actionIndex: -1 };
    }
    const row = { entry, actionIndex: nextActionIndex };
    nextActionIndex += 1;
    return row;
  });
}

export function menuActions(entries: MenuEntry[]): MenuItem[] {
  return entries.filter(isAction);
}
