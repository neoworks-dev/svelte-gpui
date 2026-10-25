import { describe, expect, test } from "bun:test";
import {
  firstEnabledIndex,
  isTypeaheadKey,
  lastEnabledIndex,
  movedIndex,
  typeaheadIndex,
  typedTextAfter,
} from "../packages/svelte-gpui-ui/src/components/listNavigation";
import { menuActions, menuRows, type MenuEntry } from "../packages/svelte-gpui-ui/src/components/menuModel";
import { parsePlacement } from "../packages/svelte-gpui-ui/src/components/placement";
import {
  scrollToReveal,
  selectedValuesOf,
  toggledValues,
  triggerTextOf,
  visibleOptionsOf,
  type SelectOption,
} from "../packages/svelte-gpui-ui/src/components/selectModel";
import { dismissDistance, dragOffset, dragOutcome } from "../packages/svelte-gpui-ui/src/components/sheetDrag";

const options: SelectOption[] = [
  { value: "apple", label: "Apple" },
  { value: "banana", label: "Banana", disabled: true },
  { value: "cherry", label: "Cherry" },
];

describe("placement", () => {
  test("a side with an alignment splits, and a bare side is centred", () => {
    expect(parsePlacement("bottom-start")).toEqual({ side: "bottom", align: "start" });
    expect(parsePlacement("top-end")).toEqual({ side: "top", align: "end" });
    expect(parsePlacement("left")).toEqual({ side: "left", align: "center" });
  });
});

describe("menu rows", () => {
  const entries: MenuEntry[] = [
    { type: "heading", label: "Edit" },
    { label: "Copy" },
    { type: "divider" },
    { label: "Delete", danger: true },
  ];

  test("only items that can be chosen get an action index", () => {
    expect(menuRows(entries).map((row) => row.actionIndex)).toEqual([-1, 0, -1, 1]);
    expect(menuActions(entries).map((item) => item.label)).toEqual(["Copy", "Delete"]);
  });
});

describe("list navigation", () => {
  const disabled = [true, false, true, false, false];

  test("Down and Up skip disabled rows and wrap", () => {
    expect(movedIndex("ArrowDown", 1, disabled)).toBe(3);
    expect(movedIndex("ArrowDown", 4, disabled)).toBe(1);
    expect(movedIndex("ArrowUp", 1, disabled)).toBe(4);
  });

  test("Up with nothing highlighted starts from the end", () => {
    expect(movedIndex("ArrowUp", -1, disabled)).toBe(4);
    expect(movedIndex("ArrowDown", -1, disabled)).toBe(1);
  });

  test("Home and End go to the first and last row that can be chosen", () => {
    expect(movedIndex("Home", 4, disabled)).toBe(1);
    expect(movedIndex("End", 1, disabled)).toBe(4);
    expect(firstEnabledIndex(disabled)).toBe(1);
    expect(lastEnabledIndex(disabled)).toBe(4);
    expect(firstEnabledIndex([true, true])).toBe(-1);
    expect(lastEnabledIndex([true, true])).toBe(-1);
  });

  test("other keys and empty lists do not move", () => {
    expect(movedIndex("a", 1, disabled)).toBeUndefined();
    expect(movedIndex("ArrowDown", -1, [])).toBeUndefined();
  });

  test("typeahead finds the next enabled label after the highlight, wrapping", () => {
    const labels = ["Apple", "Avocado", "Banana", "Apricot"];
    const none = [false, true, false, false];
    expect(typeaheadIndex("a", 0, labels, none)).toBe(3);
    expect(typeaheadIndex("a", 3, labels, none)).toBe(0);
    expect(typeaheadIndex("ban", 0, labels, none)).toBe(2);
    expect(typeaheadIndex("z", 0, labels, none)).toBeUndefined();
  });

  test("typed text keeps growing until a pause, and shortcuts are not typeahead", () => {
    expect(typedTextAfter("a", 1000, "p", 1300)).toBe("ap");
    expect(typedTextAfter("ap", 1000, "p", 2000)).toBe("p");
    expect(isTypeaheadKey("a", false, false, false)).toBe(true);
    expect(isTypeaheadKey("a", true, false, false)).toBe(false);
    expect(isTypeaheadKey(" ", false, false, false)).toBe(false);
    expect(isTypeaheadKey("Enter", false, false, false)).toBe(false);
  });
});

describe("select values", () => {
  test("a single value, an empty value and a list all read as a list of values", () => {
    expect(selectedValuesOf("apple")).toEqual(["apple"]);
    expect(selectedValuesOf("")).toEqual([]);
    expect(selectedValuesOf(["apple", "cherry"])).toEqual(["apple", "cherry"]);
  });

  test("toggling keeps the order of the options", () => {
    expect(toggledValues(options, ["cherry"], "apple")).toEqual(["apple", "cherry"]);
    expect(toggledValues(options, ["apple", "cherry"], "apple")).toEqual(["cherry"]);
  });

  test("the filter applies only with a filter and a query", () => {
    const startsWith = (option: SelectOption, query: string) => option.label.toLowerCase().startsWith(query.toLowerCase());
    expect(visibleOptionsOf(options, undefined, "a")).toHaveLength(3);
    expect(visibleOptionsOf(options, startsWith, "  ")).toHaveLength(3);
    expect(visibleOptionsOf(options, startsWith, "ch").map((option) => option.value)).toEqual(["cherry"]);
  });

  test("the trigger shows the chosen labels or the placeholder", () => {
    expect(triggerTextOf([], "Select")).toBe("Select");
    expect(triggerTextOf([options[0], options[2]], "Select")).toBe("Apple, Cherry");
  });

  test("scrolling reveals a row with the least movement", () => {
    const heights = [40, 40, 40, 40, 40];
    expect(scrollToReveal(heights, 1, 0, 100)).toBeUndefined();
    expect(scrollToReveal(heights, 4, 0, 100)).toBe(100);
    expect(scrollToReveal(heights, 0, 80, 100)).toBe(0);
  });
});

describe("sheet drag", () => {
  test("the sheet follows the pointer down and never moves up", () => {
    expect(dragOffset(100, 160)).toBe(60);
    expect(dragOffset(100, 40)).toBe(0);
  });

  test("a long drag closes, a short one settles, and no movement is a press", () => {
    expect(dragOutcome(150, dismissDistance + 1)).toBe("close");
    expect(dragOutcome(60, 60)).toBe("settle");
    expect(dragOutcome(2, 0)).toBe("press");
  });
});
