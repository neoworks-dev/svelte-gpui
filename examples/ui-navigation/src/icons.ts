import arrowLeft from "@phosphor-icons/core/regular/arrow-left.svg";
import bell from "@phosphor-icons/core/regular/bell.svg";
import bellFill from "@phosphor-icons/core/fill/bell-fill.svg";
import books from "@phosphor-icons/core/regular/books.svg";
import booksFill from "@phosphor-icons/core/fill/books-fill.svg";
import dots from "@phosphor-icons/core/regular/dots-three-vertical.svg";
import gear from "@phosphor-icons/core/regular/gear.svg";
import gearFill from "@phosphor-icons/core/fill/gear-fill.svg";
import heart from "@phosphor-icons/core/regular/heart.svg";
import heartFill from "@phosphor-icons/core/fill/heart-fill.svg";
import house from "@phosphor-icons/core/regular/house.svg";
import houseFill from "@phosphor-icons/core/fill/house-fill.svg";
import images from "@phosphor-icons/core/regular/images.svg";
import list from "@phosphor-icons/core/regular/list.svg";
import magnifyingGlass from "@phosphor-icons/core/regular/magnifying-glass.svg";
import magnifyingGlassFill from "@phosphor-icons/core/fill/magnifying-glass-fill.svg";
import pencil from "@phosphor-icons/core/regular/pencil-simple.svg";
import plus from "@phosphor-icons/core/regular/plus.svg";
import star from "@phosphor-icons/core/regular/star.svg";
import starFill from "@phosphor-icons/core/fill/star-fill.svg";
import tray from "@phosphor-icons/core/regular/tray.svg";
import trayFill from "@phosphor-icons/core/fill/tray-fill.svg";
import type { DrawerItem, NavItem } from "@neoworks/svelte-gpui-ui";

export { arrowLeft, bell, dots, heart, images, list, magnifyingGlass, pencil, plus, star };

export const navItems: NavItem[] = [
  { label: "Home", value: "/home", icon: house, selectedIcon: houseFill },
  { label: "Library", value: "/library", icon: books, selectedIcon: booksFill },
  { label: "Search", value: "/search", icon: magnifyingGlass, selectedIcon: magnifyingGlassFill },
  { label: "Inbox", value: "/inbox", icon: tray, selectedIcon: trayFill, badge: 3 },
];

export const railItems: NavItem[] = [
  ...navItems,
  { label: "Alerts", value: "/alerts", icon: bell, selectedIcon: bellFill, badge: 120 },
  { label: "Settings", value: "/settings", icon: gear, selectedIcon: gearFill },
];

export const drawerItems: DrawerItem[] = [
  { label: "Home", value: "/home", icon: house, selectedIcon: houseFill },
  { label: "Starred", value: "/starred", icon: star, selectedIcon: starFill },
  { label: "Favourites", value: "/favourites", icon: heart, selectedIcon: heartFill, badge: 12 },
];

export const pageItems: DrawerItem[] = [
  { label: "Getting started", value: "/docs/start" },
  { label: "Installation", value: "/docs/install" },
  { label: "A page with a title long enough that it has to be cut off", value: "/docs/long" },
];
