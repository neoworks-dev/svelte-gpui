import { describe, expect, test } from "bun:test";
import { createFocusSource } from "../packages/svelte-gpui-ui/src/components/focusSource";
import { fabLook } from "../packages/svelte-gpui-ui/src/components/fab-look";
import { iconForState, isActivationKey, isCurrentItem } from "../packages/svelte-gpui-ui/src/components/navigation";
import {
  destinationIndicatorLook,
  drawerItemLook,
  drawerLook,
  glassBackground,
  navigationRailLook,
} from "../packages/svelte-gpui-ui/src/components/navigation-look";
import { tabKeyAction } from "../packages/svelte-gpui-ui/src/components/tabsLogic";
import { tabIndicatorLook, tabWashLook } from "../packages/svelte-gpui-ui/src/components/tabs-look";
import {
  hasHeadlineBlock,
  showsRowTitle,
  topAppBarHeadlineLook,
  topAppBarLook,
} from "../packages/svelte-gpui-ui/src/components/top-app-bar-look";
import type { TabItem } from "../packages/svelte-gpui-ui/src/components/types";
import { themes } from "../packages/svelte-gpui-ui/src/theme";

const light = themes.light;

describe("current item", () => {
  test("an item stays current for its own address and everything below it", () => {
    expect(isCurrentItem("/library", "/library")).toBe(true);
    expect(isCurrentItem("/library/albums/3", "/library")).toBe(true);
  });

  test("an address that only shares a prefix is another item", () => {
    expect(isCurrentItem("/library-old", "/library")).toBe(false);
    expect(isCurrentItem("/home", "/library")).toBe(false);
  });

  test("the selected icon is drawn only while active and only when there is one", () => {
    expect(iconForState({ icon: "a", selectedIcon: "b" }, true)).toBe("b");
    expect(iconForState({ icon: "a", selectedIcon: "b" }, false)).toBe("a");
    expect(iconForState({ icon: "a" }, true)).toBe("a");
    expect(iconForState({}, true)).toBeUndefined();
  });

  test("Enter and Space activate", () => {
    expect(isActivationKey("Enter")).toBe(true);
    expect(isActivationKey(" ")).toBe(true);
    expect(isActivationKey("a")).toBe(false);
  });
});

describe("tab keys", () => {
  const tabs: TabItem[] = [
    { id: "all", label: "All" },
    { id: "unread", label: "Unread" },
    { id: "archived", label: "Archived", disabled: true },
    { id: "spam", label: "Spam" },
  ];

  test("the arrows move to the next enabled tab and wrap at the ends", () => {
    expect(tabKeyAction("ArrowRight", tabs, 0)).toEqual({ kind: "move", index: 1 });
    expect(tabKeyAction("ArrowRight", tabs, 1)).toEqual({ kind: "move", index: 3 });
    expect(tabKeyAction("ArrowRight", tabs, 3)).toEqual({ kind: "move", index: 0 });
    expect(tabKeyAction("ArrowLeft", tabs, 0)).toEqual({ kind: "move", index: 3 });
    expect(tabKeyAction("ArrowLeft", tabs, 3)).toEqual({ kind: "move", index: 1 });
  });

  test("Home and End go to the first and last enabled tab", () => {
    expect(tabKeyAction("Home", tabs, 3)).toEqual({ kind: "move", index: 0 });
    expect(tabKeyAction("End", tabs, 0)).toEqual({ kind: "move", index: 3 });
  });

  test("Enter and Space choose the focused tab and other keys do nothing", () => {
    expect(tabKeyAction("Enter", tabs, 1)).toEqual({ kind: "select" });
    expect(tabKeyAction(" ", tabs, 1)).toEqual({ kind: "select" });
    expect(tabKeyAction("a", tabs, 1)).toEqual({ kind: "none" });
    expect(tabKeyAction("ArrowDown", tabs, 1)).toEqual({ kind: "none" });
  });
});

describe("focus from a press", () => {
  test("the focus a press gave is not keyboard focus, and the next one is", () => {
    const source = createFocusSource();
    source.pointerDown();
    expect(source.focused()).toBe(false);
    expect(source.focused()).toBe(true);
  });

  test("a press on something already focused does not hide the next keyboard focus", () => {
    const source = createFocusSource();
    source.pointerDown();
    source.blurred();
    expect(source.focused()).toBe(true);
  });
});

describe("top app bar", () => {
  test("only medium and large bars have a headline block, and they show the row title once scrolled", () => {
    expect(hasHeadlineBlock("small")).toBe(false);
    expect(hasHeadlineBlock("centered")).toBe(false);
    expect(hasHeadlineBlock("large")).toBe(true);
    expect(showsRowTitle("small", false)).toBe(true);
    expect(showsRowTitle("large", false)).toBe(false);
    expect(showsRowTitle("large", true)).toBe(true);
  });

  test("the bar is the canvas at the top and glass without a border once scrolled", () => {
    const resting = topAppBarLook(light, false);
    const scrolled = topAppBarLook(light, true);
    expect(resting.background).toBe(light.colors.canvas);
    expect(scrolled.background).toBe(glassBackground(light.colors.canvas));
    expect(scrolled.borderWidth).toBeUndefined();
    expect(scrolled.borderBottomWidth).toBeUndefined();
  });

  test("the headline block is hidden once scrolled", () => {
    expect(topAppBarHeadlineLook(false).display).toBeUndefined();
    expect(topAppBarHeadlineLook(true).display).toBe("none");
  });
});

describe("drawer looks", () => {
  test("a docked drawer has no background, border or shadow", () => {
    const docked = drawerLook(light, true);
    expect(docked.background).toBeUndefined();
    expect(docked.borderWidth).toBeUndefined();
    expect(docked.shadow).toBeUndefined();
  });

  test("a modal drawer is a surface with the media radius on its right and the overlay shadow", () => {
    const modal = drawerLook(light, false);
    expect(modal.background).toBe(light.colors.surface);
    expect(modal.radius).toEqual([0, light.radii.media, light.radii.media, 0]);
    expect(modal.shadow).toBeDefined();
  });

  test("docked rows are 40 high with small text, modal rows 48 with body text", () => {
    const docked = drawerItemLook(light, { docked: true, active: false }).style;
    const modal = drawerItemLook(light, { docked: false, active: false }).style;
    expect(docked.height).toBe(40);
    expect(modal.height).toBe(48);
    expect(docked.fontSize).toBeLessThan(modal.fontSize as number);
  });

  test("the active row takes the hover colour and the medium weight", () => {
    const active = drawerItemLook(light, { docked: true, active: true }).style;
    const idle = drawerItemLook(light, { docked: true, active: false }).style;
    expect(active.background).toBe(light.colors.hover);
    expect(active.fontWeight).toBe(500);
    expect(idle.background).toBe("transparent");
    expect(idle.color).toBe(light.colors.muted);
  });
});

describe("destinations, tabs and fab", () => {
  const quiet = { active: false, hovered: false, pressed: false, focused: false };

  test("the indicator shows the pressed colour while active and the hover colour while hovered", () => {
    expect(destinationIndicatorLook(light, "bar", { ...quiet, active: true }).background).toBe(light.colors.pressed);
    expect(destinationIndicatorLook(light, "bar", { ...quiet, hovered: true }).background).toBe(light.colors.hover);
    expect(destinationIndicatorLook(light, "bar", quiet).background).toBe("transparent");
  });

  test("a focused indicator carries a ring", () => {
    expect(destinationIndicatorLook(light, "rail", { ...quiet, focused: true }).shadow).toHaveLength(1);
  });

  test("the rail is 80 wide with a border on its right only", () => {
    const rail = navigationRailLook(light);
    expect(rail.width).toBe(80);
    expect(rail.borderRightWidth).toBe(1);
    expect(rail.borderWidth).toBeUndefined();
  });

  test("the tab indicator is visible only for the active tab", () => {
    expect(tabIndicatorLook(light, { variant: "primary", fullWidth: false }, true).opacity).toBe(1);
    expect(tabIndicatorLook(light, { variant: "primary", fullWidth: false }, false).opacity).toBe(0);
    expect(tabIndicatorLook(light, { variant: "secondary", fullWidth: false }, true).height).toBe(1);
  });

  test("a tab washes faintly on hover and a little more when pressed", () => {
    const state = { active: false, hovered: false, pressed: false, focused: false, hasIcon: false, disabled: false };
    expect(tabWashLook(light, state).background).toBe("transparent");
    expect(tabWashLook(light, { ...state, hovered: true }).background).toBe(`${light.colors.text}0a`);
    expect(tabWashLook(light, { ...state, pressed: true }).background).toBe(`${light.colors.text}14`);
  });

  test("a fab is 40, 56 or 96 high and extends with a padded label", () => {
    expect(fabLook(light, { variant: "primary", size: "sm", extended: false }).style.height).toBe(40);
    expect(fabLook(light, { variant: "primary", size: "md", extended: false }).style.height).toBe(56);
    expect(fabLook(light, { variant: "primary", size: "lg", extended: false }).style.height).toBe(96);
    expect(fabLook(light, { variant: "primary", size: "md", extended: true }).style.paddingX).toBe(20);
    expect(fabLook(light, { variant: "primary", size: "md", extended: false }).style.paddingX).toBeUndefined();
  });

  test("a fab rises to the overlay shadow on hover, drawn with a visible blur", () => {
    const look = fabLook(light, { variant: "surface", size: "md", extended: false });
    const hoverShadow = look.hover?.shadow as { blur: number }[];
    expect(hoverShadow.every((layer) => layer.blur >= 1)).toBe(true);
  });
});
