import { describe, expect, test } from "bun:test";
import { staysOpen } from "../packages/svelte-gpui-ui/src/components/disclosureGroupContext";
import {
  listItemLook,
  listItemSupportingTextLook,
  listLook,
  listRowsLook,
} from "../packages/svelte-gpui-ui/src/components/list-look";
import { iconCircleLook, statusBadgeLook } from "../packages/svelte-gpui-ui/src/components/status-look";
import { semanticToneOf, toneColors } from "../packages/svelte-gpui-ui/src/components/status-tone";
import { tableBodyRowLook, tableCellBoxLook } from "../packages/svelte-gpui-ui/src/components/table-look";
import {
  cellText,
  columnWeights,
  isEndAligned,
  type TableColumn,
  type TableRow,
} from "../packages/svelte-gpui-ui/src/components/tableModel";
import { themes } from "../packages/svelte-gpui-ui/src/theme";

const light = themes.light;

describe("table columns", () => {
  const columns: TableColumn[] = [
    { key: "name", label: "Name" },
    { key: "size", label: "Size", numeric: true },
    { key: "note", label: "Note", align: "end" },
    { key: "kind", label: "Kind", width: 90 },
  ];
  const rows: TableRow[] = [
    { name: "holiday-photos-2026.zip", size: 4210, note: "ok", kind: "Archive" },
    { name: "a.txt", size: 2 },
  ];

  test("numeric and end-aligned columns sit on the end, others on the start", () => {
    expect(isEndAligned(columns[0])).toBe(false);
    expect(isEndAligned(columns[1])).toBe(true);
    expect(isEndAligned(columns[2])).toBe(true);
  });

  test("a missing value is an empty cell and a number is its text", () => {
    expect(cellText(rows[1], columns[3])).toBe("");
    expect(cellText(rows[0], columns[1])).toBe("4210");
  });

  test("columns share the width by their longest text, between a floor and a ceiling, and a fixed width takes no share", () => {
    const weights = columnWeights(columns, rows);
    expect(weights[0]).toBe("holiday-photos-2026.zip".length);
    expect(weights[1]).toBe(4);
    expect(weights[3]).toBeUndefined();
    const long = columnWeights([{ key: "name", label: "Name" }], [{ name: "x".repeat(80) }]);
    expect(long[0]).toBe(28);
  });

  test("the first cell starts at the edge, the last ends at it, and fixed columns do not shrink", () => {
    const options = { column: columns[0], weight: 10, columnCount: 4, density: "comfortable" as const };
    expect(tableCellBoxLook({ ...options, columnIndex: 0 })).toMatchObject({ paddingLeft: 0, paddingRight: 16, flex: 10 });
    expect(tableCellBoxLook({ ...options, columnIndex: 3 })).toMatchObject({ paddingLeft: 16, paddingRight: 0 });
    const fixed = tableCellBoxLook({ ...options, column: columns[3], weight: undefined, columnIndex: 3 });
    expect(fixed).toMatchObject({ width: 90, shrink: 0 });
    expect(tableCellBoxLook({ ...options, columnIndex: 1, density: "compact" }).paddingY).toBe(8);
  });

  test("every body row but the first has a line above it", () => {
    expect(tableBodyRowLook(light, 0).borderTopWidth).toBeUndefined();
    expect(tableBodyRowLook(light, 1).borderTopWidth).toBe(1);
  });
});

describe("disclosure group", () => {
  test("outside an exclusive group, or with nothing opened, every disclosure stays open", () => {
    expect(staysOpen(false, "b", "a")).toBe(true);
    expect(staysOpen(true, undefined, "a")).toBe(true);
  });

  test("in an exclusive group only the disclosure that opened last stays open", () => {
    expect(staysOpen(true, "b", "b")).toBe(true);
    expect(staysOpen(true, "b", "a")).toBe(false);
  });
});

describe("status tones", () => {
  test("the older colour names are aliases of the semantic tones", () => {
    expect(semanticToneOf("green")).toBe("success");
    expect(semanticToneOf("red")).toBe("danger");
    expect(semanticToneOf("amber")).toBe("warning");
    expect(semanticToneOf("blue")).toBe("info");
    expect(semanticToneOf("violet")).toBe("neutral");
    expect(toneColors(light, "green")).toEqual(toneColors(light, "success"));
  });

  test("a tone is a soft fill with the tone's own text colour, in both themes", () => {
    for (const theme of [light, themes.dark]) {
      expect(toneColors(theme, "danger")).toMatchObject({ background: theme.colors.dangerSoft, color: theme.colors.danger });
      expect(toneColors(theme, "neutral")).toMatchObject({ background: theme.colors.sunken, color: theme.colors.muted });
    }
  });

  test("badge and circle take the tone's fill, and the circle's size sets its box", () => {
    expect(statusBadgeLook(light, "info")).toMatchObject({ background: light.colors.infoSoft, height: 24 });
    expect(iconCircleLook(light, "success", "sm")).toMatchObject({ size: 32, radius: 12 });
    expect(iconCircleLook(light, "success", "lg")).toMatchObject({ size: 48, radius: 16 });
  });
});

describe("list", () => {
  const rest = { interactive: false, selected: false, disabled: false, dividers: false };

  test("a card is an outlined clipped surface, a plain list is only a column", () => {
    expect(listLook(light, "card", false)).toMatchObject({ overflow: "hidden", borderWidth: 1, background: light.colors.surface });
    expect(listLook(light, "plain", false).borderWidth).toBeUndefined();
    expect(listLook(light, "plain", false).overflow).toBeUndefined();
  });

  test("dividers pull the rows up by the line of the first one and clip it", () => {
    expect(listRowsLook(true).marginTop).toBe(-1);
    expect(listRowsLook(false).marginTop).toBeUndefined();
    expect(listLook(light, "plain", true).overflow).toBe("hidden");
    expect(listItemLook(light, { ...rest, dividers: true }).style.borderTopWidth).toBe(1);
  });

  test("only a pressable row has hover and press states", () => {
    expect(listItemLook(light, rest).hover).toBeUndefined();
    expect(listItemLook(light, { ...rest, interactive: true }).hover).toBeDefined();
  });

  test("a selected row washes over its own fill and an unselected one over what is behind it", () => {
    const selected = listItemLook(light, { ...rest, interactive: true, selected: true });
    expect(selected.style.background).toBe(light.colors.sunken);
    expect(selected.hover?.background).toMatch(/^#[0-9a-f]{6}$/);
    expect(selected.hover?.background).not.toBe(light.colors.sunken);
    const plain = listItemLook(light, { ...rest, interactive: true });
    expect(plain.hover?.background).toBe(`${light.colors.text}0a`);
    expect(plain.active?.background).toBe(`${light.colors.text}14`);
  });

  test("a disabled row fades", () => {
    expect(listItemLook(light, { ...rest, disabled: true }).style.opacity).toBe(0.4);
  });

  test("one line of supporting text is cut with an ellipsis, more are clamped", () => {
    expect(listItemSupportingTextLook(light, 1)).toMatchObject({ truncate: true });
    expect(listItemSupportingTextLook(light, 1).maxLines).toBeUndefined();
    expect(listItemSupportingTextLook(light, 3)).toMatchObject({ maxLines: 3 });
    expect(listItemSupportingTextLook(light, 3).truncate).toBeUndefined();
  });
});
