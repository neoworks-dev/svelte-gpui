import { beforeAll, describe, expect, test } from "bun:test";
import "../runtime/dom/globals";
import { SElement } from "../runtime/dom/nodes";
import { adoptRoot, flush, install } from "../runtime/host/bridge";

/** What the host says about each field, by node id: start, end, the cursor's row and the row count. */
const states = new Map<number, number[]>();
(globalThis as any).host = { apply: () => {}, fieldState: (id: number) => states.get(id) ?? [] };

let field: SElement;

beforeAll(() => {
  install();
  const root = new SElement("#root");
  adoptRoot(root);
  field = new SElement("g-input");
  root.appendChild(field);
  flush();
});

describe("fieldState()", () => {
  test("is null for a field the host has not drawn", () => {
    expect(field.fieldState()).toBeNull();
  });

  test("gives the selection as the host has it", () => {
    states.set(field.hostId, [3, 8, 0, 1]);
    expect(field.fieldState()).toEqual({ start: 3, end: 8, firstRow: true, lastRow: true });
  });

  test("says which rows the cursor is on", () => {
    states.set(field.hostId, [10, 10, 0, 3]);
    expect(field.fieldState()).toMatchObject({ firstRow: true, lastRow: false });
    states.set(field.hostId, [20, 20, 1, 3]);
    expect(field.fieldState()).toMatchObject({ firstRow: false, lastRow: false });
    states.set(field.hostId, [30, 30, 2, 3]);
    expect(field.fieldState()).toMatchObject({ firstRow: false, lastRow: true });
  });
});
