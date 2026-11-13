import { beforeAll, describe, expect, test } from "bun:test";
import "../runtime/dom/globals";
import { SComment, SElement, SText } from "../runtime/dom/nodes";
import { adoptRoot, flush, install, nodeFor } from "../runtime/host/bridge";

(globalThis as any).host = { apply: () => {} };

let root: SElement;

beforeAll(() => {
  install();
  root = new SElement("#root");
  adoptRoot(root);
});

describe("an element", () => {
  test("reports its name as the DOM does, in capitals", () => {
    expect(new SElement("g-view").nodeName).toBe("G-VIEW");
    expect(new SElement("g-view").tagName).toBe("G-VIEW");
    expect(new SText("a").nodeName).toBe("#text");
    expect(new SComment("a").nodeName).toBe("#comment");
  });

  test("keeps no attribute, listener or style until it is given one", () => {
    const element = new SElement("g-view");
    expect(element.hasAttribute("look")).toBe(false);
    expect(element.getAttribute("look")).toBeNull();
    expect(element.declarations).toBeNull();
    element.setAttribute("look", "{}");
    element.addEventListener("press", () => {});
    element.style.setProperty("color", "red");
    expect(element.getAttribute("look")).toBe("{}");
    expect(element.listeners.get("press")?.size).toBe(1);
    expect(element.declarations?.get("color")).toBe("red");
  });

  test("starts at no size and no scroll position, and remembers what the host reports", () => {
    const element = new SElement("g-view");
    expect(element.offsetWidth).toBe(0);
    expect(element.scrollTop).toBe(0);
    element.offsetWidth = 40;
    expect(element.offsetWidth).toBe(40);
    expect(new SElement("g-view").offsetWidth).toBe(0);
  });

  test("copies its attributes and inline style when it is cloned", () => {
    const element = new SElement("g-view");
    element.setAttribute("look", "{}");
    element.style.setProperty("color", "red");
    const copy = element.cloneNode(true);
    expect(copy.getAttribute("look")).toBe("{}");
    expect(copy.declarations?.get("color")).toBe("red");
  });
});

describe("a subtree the host dropped", () => {
  test("is cut apart, so that no node in it holds another and the engine can free each by itself", () => {
    const parent = new SElement("g-view");
    const child = new SElement("g-view");
    const text = new SText("hello");
    const anchor = new SComment("");
    child.addEventListener("press", () => {});
    child.style.setProperty("color", "red");
    child.appendChild(text);
    child.appendChild(anchor);
    parent.appendChild(child);
    root.appendChild(parent);
    flush();
    expect(nodeFor(child.hostId)).toBe(child);
    const ids = [parent.hostId, child.hostId, text.hostId];

    root.removeChild(parent);
    flush();

    for (const node of [parent, child, text, anchor]) {
      expect([node.parentNode, node.firstChildNode, node.lastChildNode, node.nextSiblingNode, node.prevSiblingNode]).toEqual([
        null,
        null,
        null,
        null,
        null,
      ]);
    }
    expect(child.listeners.size).toBe(0);
    expect(child.declarations).toBeNull();
    for (const id of ids) {
      expect(nodeFor(id)).toBeUndefined();
    }
  });

  test("is kept when it is put back before the host is told", () => {
    const parent = new SElement("g-view");
    const child = new SElement("g-view");
    parent.appendChild(child);
    root.appendChild(parent);
    flush();

    root.removeChild(parent);
    root.appendChild(parent);
    flush();

    expect(nodeFor(parent.hostId)).toBe(parent);
    expect(child.parentNode).toBe(parent);
  });
});
