import { beforeAll, describe, expect, test } from "bun:test";
import "../runtime/dom/globals";
import { SElement } from "../runtime/dom/nodes";
import { adoptRoot, flush, install } from "../runtime/host/bridge";
import { dispatch } from "../runtime/host/events";
import protocol from "../protocol.json";

const FOCUS = protocol.operations.indexOf("focus");
const BLUR = protocol.operations.indexOf("blur");

const batches: unknown[][][] = [];
(globalThis as any).host = { apply: (batch: string) => batches.push(JSON.parse(batch)) };

let field: SElement;
let panel: SElement;
let other: SElement;

/** What the host does when focus moves: the old element hears `blur`, the new one `focus` and where it came from. */
function moveFocus(from: SElement | null, to: SElement): void {
  if (from !== null) {
    dispatch(from.hostId, "blur", {});
  }
  dispatch(to.hostId, "focus", from === null ? {} : { from: from.hostId });
}

/** The operations that `action` sent the host, as `[code, id]`. */
function sent(action: () => void): unknown[][] {
  flush();
  batches.length = 0;
  action();
  flush();
  return batches.flat().filter((op) => op[0] === FOCUS || op[0] === BLUR);
}

beforeAll(() => {
  install();
  const root = new SElement("#root");
  adoptRoot(root);
  field = new SElement("g-input");
  panel = new SElement("g-view");
  other = new SElement("g-view");
  root.appendChild(field);
  root.appendChild(panel);
  root.appendChild(other);
  flush();
});

describe("blur()", () => {
  test("hands focus back to the element that had it before", () => {
    moveFocus(null, field);
    moveFocus(field, panel);
    expect(sent(() => panel.blur())).toEqual([[FOCUS, field.hostId]]);
  });

  test("does not hand focus on again from the element it handed it to", () => {
    moveFocus(null, field);
    moveFocus(field, panel);
    panel.blur();
    moveFocus(panel, field);
    expect(sent(() => field.blur())).toEqual([[BLUR, field.hostId]]);
  });

  test("drops focus when nothing had it before", () => {
    moveFocus(null, panel);
    expect(sent(() => panel.blur())).toEqual([[BLUR, panel.hostId]]);
  });

  test("drops focus when the element that had it is gone", () => {
    moveFocus(null, other);
    moveFocus(other, panel);
    other.remove();
    expect(sent(() => panel.blur())).toEqual([[BLUR, panel.hostId]]);
  });

  test("does nothing on an element without focus", () => {
    moveFocus(null, field);
    expect(sent(() => panel.blur())).toEqual([]);
  });
});
