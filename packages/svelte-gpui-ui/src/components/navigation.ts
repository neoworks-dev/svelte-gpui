/** A destination in a NavBar or NavigationRail. */
export interface NavItem {
  label: string;
  /** What `current` is compared against, and what `onselect` receives. A path such as `/library` lets its sub-pages keep the item active. */
  value: string;
  /** SVG markup, usually the regular weight. */
  icon: string;
  /** SVG markup drawn while the item is active, such as the filled weight of `icon`. */
  selectedIcon?: string;
  /** A count shown on the icon, such as unread items. */
  badge?: number;
}

/** A destination in the NavigationDrawer. Leave out the icon for long lists such as documentation pages. */
export interface DrawerItem extends Omit<NavItem, "icon"> {
  icon?: string;
}

export interface NavSection {
  /** A small heading above the group. */
  heading?: string;
  items: DrawerItem[];
}

/** True when `current` is the item's value or somewhere below it. */
export function isCurrentItem(current: string, value: string): boolean {
  return current === value || current.startsWith(value + "/");
}

/** The icon to draw for an item: its selected icon while active, when it has one. */
export function iconForState(item: { icon?: string; selectedIcon?: string }, active: boolean): string | undefined {
  if (active && item.selectedIcon !== undefined) {
    return item.selectedIcon;
  }
  return item.icon;
}

/** Enter and Space press the focused item, as on a button. */
export function isActivationKey(key: string): boolean {
  return key === "Enter" || key === " ";
}
