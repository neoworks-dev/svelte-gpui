// A node tree with exactly the surface Svelte's client runtime touches, and
// nothing else. Unlike svelte-gtk4 every element and every text node is also a
// node in the host's tree: the bridge sends what changes, and the host draws it.
//
// `firstChild` and `nextSibling` have to be accessors on the prototype:
// `init_operations()` pulls their getters off `Node.prototype` once and calls
// them unbound for the rest of the program's life.

import { mirror } from "./hooks";

export const ELEMENT_NODE = 1;
export const TEXT_NODE = 3;
export const COMMENT_NODE = 8;
export const DOCUMENT_FRAGMENT_NODE = 11;

// Nodes are announced to the host when they are created, but a template is
// parsed once and only ever cloned, so the parse itself must announce nothing.
let building = true;

export function setBuilding(value: boolean): boolean {
  const previous = building;
  building = value;
  return previous;
}

export function isBuilding(): boolean {
  return building;
}

/** What the bridge does for a node: announce it, and hear when it changes. */
export interface NodeBackend {
  created(node: SNode): void;
  attributeChanged(node: SElement, name: string, value: unknown): void;
  styleChanged(node: SElement, property: string, value: string | null): void;
  listenerAdded(node: SElement, type: string, handler: EventHandler): void;
  listenerRemoved(node: SElement, type: string): void;
  focus(node: SElement): void;
  scrollTo(node: SElement, left: number, top: number): void;
  select(node: SElement, start: number, end: number): void;
  /** A field's selection, and where its cursor is among the rows, right now; nothing if it is not a drawn field. */
  fieldState(node: SElement): number[];
}

export type EventHandler = (event: any) => void;

const noBackend: NodeBackend = {
  created() {},
  attributeChanged() {},
  styleChanged() {},
  listenerAdded() {},
  listenerRemoved() {},
  focus() {},
  scrollTo() {},
  select() {},
  fieldState: () => [],
};

let backend = noBackend;

export function setBackend(next: NodeBackend): void {
  backend = next;
}

export class SNode {
  nodeType = 0;
  nodeName = "#node";
  parentNode: SNode | null = null;
  /** The host's id for this node, or 0 when it is only a mould or a comment. */
  hostId = 0;

  firstChildNode: SNode | null = null;
  lastChildNode: SNode | null = null;
  nextSiblingNode: SNode | null = null;
  prevSiblingNode: SNode | null = null;

  get firstChild(): SNode | null {
    return this.firstChildNode;
  }

  get lastChild(): SNode | null {
    return this.lastChildNode;
  }

  get nextSibling(): SNode | null {
    return this.nextSiblingNode;
  }

  get previousSibling(): SNode | null {
    return this.prevSiblingNode;
  }

  get childNodes(): SNode[] {
    const out: SNode[] = [];
    let child = this.firstChildNode;
    while (child !== null) {
      out.push(child);
      child = child.nextSiblingNode;
    }
    return out;
  }

  /** The element children, in order. */
  get children(): SNode[] {
    return this.childNodes.filter((child) => child.nodeType === ELEMENT_NODE);
  }

  get childElementCount(): number {
    return this.children.length;
  }

  get firstElementChild(): SNode | null {
    return this.children[0] ?? null;
  }

  get lastElementChild(): SNode | null {
    const all = this.children;
    return all[all.length - 1] ?? null;
  }

  get parentElement(): SNode | null {
    const parent = this.parentNode;
    if (parent !== null && parent.nodeType === ELEMENT_NODE) {
      return parent;
    }
    return null;
  }

  get nextElementSibling(): SNode | null {
    for (let sibling = this.nextSiblingNode; sibling !== null; sibling = sibling.nextSiblingNode) {
      if (sibling.nodeType === ELEMENT_NODE) {
        return sibling;
      }
    }
    return null;
  }

  get previousElementSibling(): SNode | null {
    for (let sibling = this.prevSiblingNode; sibling !== null; sibling = sibling.prevSiblingNode) {
      if (sibling.nodeType === ELEMENT_NODE) {
        return sibling;
      }
    }
    return null;
  }

  get isConnected(): boolean {
    let node: SNode = this;
    while (node.parentNode !== null) {
      node = node.parentNode;
    }
    return node.nodeName === "#root";
  }

  appendChild<T extends SNode>(node: T): T {
    return this.insertBefore(node, null);
  }

  // A component whose template is nothing but anchors is built into a fragment
  // the runtime fills through `append` rather than `appendChild`.
  append(...nodes: SNode[]): void {
    for (const node of nodes) {
      this.insertBefore(node, null);
    }
  }

  insertBefore<T extends SNode>(node: T, ref: SNode | null): T {
    if (node.nodeType === DOCUMENT_FRAGMENT_NODE) {
      this.insertFragment(node, ref);
      return node;
    }
    if (node.parentNode !== null) {
      node.parentNode.removeChild(node);
    }
    this.link(node, ref);
    mirror.inserted(node, ref);
    return node;
  }

  removeChild<T extends SNode>(node: T): T {
    if (node.parentNode !== this) {
      return node;
    }
    this.unlink(node);
    mirror.removed(node, this);
    return node;
  }

  remove(): void {
    if (this.parentNode !== null) {
      this.parentNode.removeChild(this);
    }
  }

  before(...nodes: SNode[]): void {
    const parent = this.parentNode;
    if (parent === null) {
      return;
    }
    for (const node of nodes) {
      parent.insertBefore(node, this);
    }
  }

  after(...nodes: SNode[]): void {
    const parent = this.parentNode;
    if (parent === null) {
      return;
    }
    let ref = this.nextSiblingNode;
    for (const node of nodes) {
      parent.insertBefore(node, ref);
      ref = node.nextSiblingNode;
    }
  }

  replaceWith(...nodes: SNode[]): void {
    this.before(...nodes);
    this.remove();
  }

  contains(node: SNode | null): boolean {
    let candidate = node;
    while (candidate !== null) {
      if (candidate === this) {
        return true;
      }
      candidate = candidate.parentNode;
    }
    return false;
  }

  cloneNode(_deep?: boolean): SNode {
    throw new Error(`cloneNode is not implemented for ${this.nodeName}`);
  }

  get textContent(): string {
    let out = "";
    let child = this.firstChildNode;
    while (child !== null) {
      out += child.textContent;
      child = child.nextSiblingNode;
    }
    return out;
  }

  set textContent(value: string) {
    while (this.firstChildNode !== null) {
      this.removeChild(this.firstChildNode);
    }
    if (value !== "") {
      this.appendChild(new SText(value));
    }
  }

  // Listeners live on elements only; any node can still be asked to take one.
  addEventListener(..._args: unknown[]): void {}
  removeEventListener(..._args: unknown[]): void {}
  dispatchEvent(_event: unknown): boolean {
    return true;
  }

  protected insertFragment(fragment: SNode, ref: SNode | null): void {
    let child = fragment.firstChildNode;
    while (child !== null) {
      const next = child.nextSiblingNode;
      this.insertBefore(child, ref);
      child = next;
    }
  }

  protected link(node: SNode, ref: SNode | null): void {
    node.parentNode = this;
    node.nextSiblingNode = ref;
    if (ref === null) {
      node.prevSiblingNode = this.lastChildNode;
      this.lastChildNode = node;
    } else {
      node.prevSiblingNode = ref.prevSiblingNode;
      ref.prevSiblingNode = node;
    }
    if (node.prevSiblingNode === null) {
      this.firstChildNode = node;
    } else {
      node.prevSiblingNode.nextSiblingNode = node;
    }
  }

  protected unlink(node: SNode): void {
    if (node.prevSiblingNode === null) {
      this.firstChildNode = node.nextSiblingNode;
    } else {
      node.prevSiblingNode.nextSiblingNode = node.nextSiblingNode;
    }
    if (node.nextSiblingNode === null) {
      this.lastChildNode = node.prevSiblingNode;
    } else {
      node.nextSiblingNode.prevSiblingNode = node.prevSiblingNode;
    }
    node.parentNode = null;
    node.nextSiblingNode = null;
    node.prevSiblingNode = null;
  }
}

export class SText extends SNode {
  override nodeType = TEXT_NODE;
  override nodeName = "#text";
  data: string;

  constructor(data = "") {
    super();
    this.data = data;
    if (isBuilding()) {
      backend.created(this);
    }
  }

  get nodeValue(): string {
    return this.data;
  }

  set nodeValue(value: string) {
    if (value === this.data) {
      return;
    }
    this.data = value;
    mirror.textChanged(this);
  }

  override get textContent(): string {
    return this.data;
  }

  override set textContent(value: string) {
    this.nodeValue = value;
  }

  override cloneNode(): SText {
    return new SText(this.data);
  }
}

export class SComment extends SNode {
  override nodeType = COMMENT_NODE;
  override nodeName = "#comment";
  data: string;

  constructor(data = "") {
    super();
    this.data = data;
  }

  get nodeValue(): string {
    return this.data;
  }

  set nodeValue(value: string) {
    this.data = value;
  }

  override cloneNode(): SComment {
    return new SComment(this.data);
  }
}

export class SFragment extends SNode {
  override nodeType = DOCUMENT_FRAGMENT_NODE;
  override nodeName = "#document-fragment";

  override cloneNode(deep?: boolean): SFragment {
    const copy = new SFragment();
    if (deep !== true) {
      return copy;
    }
    let child = this.firstChildNode;
    while (child !== null) {
      copy.appendChild(child.cloneNode(true));
      child = child.nextSiblingNode;
    }
    return copy;
  }
}

/** `element.style`: the inline declarations Svelte writes with `style:` and `style=`. */
export class SStyle {
  readonly declarations = new Map<string, string>();

  constructor(private readonly owner: SElement) {}

  setProperty(name: string, value: string | null): void {
    if (value === null || value === "") {
      this.removeProperty(name);
      return;
    }
    if (this.declarations.get(name) === value) {
      return;
    }
    this.declarations.set(name, value);
    backend.styleChanged(this.owner, name, value);
  }

  getPropertyValue(name: string): string {
    const value = this.declarations.get(name);
    if (value === undefined) {
      return "";
    }
    return value;
  }

  removeProperty(name: string): string {
    const previous = this.getPropertyValue(name);
    if (this.declarations.delete(name)) {
      backend.styleChanged(this.owner, name, null);
    }
    return previous;
  }

  get cssText(): string {
    const parts: string[] = [];
    for (const [name, value] of this.declarations) {
      parts.push(`${name}: ${value}`);
    }
    return parts.join("; ");
  }

  set cssText(text: string) {
    for (const name of [...this.declarations.keys()]) {
      this.removeProperty(name);
    }
    for (const declaration of text.split(";")) {
      const colon = declaration.indexOf(":");
      if (colon === -1) {
        continue;
      }
      this.setProperty(
        declaration.slice(0, colon).trim(),
        declaration.slice(colon + 1).trim(),
      );
    }
  }
}

export class SElement extends SNode {
  override nodeType = ELEMENT_NODE;
  /** The lowercase name, which is what the host and selectors use. */
  readonly localName: string;

  /** The name as the DOM reports an HTML element's: uppercase. */
  get tagName(): string {
    return this.localName.toUpperCase();
  }

  readonly attributes = new Map<string, unknown>();
  readonly style: SStyle;
  readonly listeners = new Map<string, Set<EventHandler>>();
  content: SFragment | null = null;

  constructor(tagName: string) {
    super();
    this.localName = tagName;
    this.nodeName = tagName.toUpperCase();
    this.style = new SStyle(this);
    if (tagName === "template") {
      this.content = new SFragment();
    }
    if (isBuilding()) {
      backend.created(this);
    }
  }

  getAttribute(name: string): unknown {
    const value = this.attributes.get(name);
    if (value === undefined) {
      return null;
    }
    return value;
  }

  hasAttribute(name: string): boolean {
    return this.attributes.has(name);
  }

  setAttribute(name: string, value: unknown): void {
    if (name === "style" && typeof value === "string") {
      this.style.cssText = value;
      return;
    }
    if (this.attributes.get(name) === value) {
      return;
    }
    this.attributes.set(name, value);
    backend.attributeChanged(this, name, value);
  }

  removeAttribute(name: string): void {
    this.attributes.delete(name);
    backend.attributeChanged(this, name, null);
  }

  set innerHTML(html: string) {
    const target = this.content === null ? this : this.content;
    while (target.firstChild !== null) {
      target.removeChild(target.firstChild);
    }
    parseInto(target, html);
  }

  /** Layout as the host last measured it; zero until the host reports. */
  clientWidth = 0;
  clientHeight = 0;
  offsetWidth = 0;
  offsetHeight = 0;
  /** Where the host last measured the element in the window. */
  boxLeft = 0;
  boxTop = 0;
  scrollWidth = 0;
  scrollHeight = 0;
  scrollLeftValue = 0;
  scrollTopValue = 0;

  get scrollTop(): number {
    return this.scrollTopValue;
  }

  set scrollTop(value: number) {
    this.scrollTopValue = value;
    backend.scrollTo(this, this.scrollLeftValue, value);
  }

  get scrollLeft(): number {
    return this.scrollLeftValue;
  }

  set scrollLeft(value: number) {
    this.scrollLeftValue = value;
    backend.scrollTo(this, value, this.scrollTopValue);
  }

  focus(): void {
    backend.focus(this);
  }

  blur(): void {}

  /** Selects part of a `g-input`'s text, in UTF-16 offsets. */
  setSelectionRange(start: number, end: number): void {
    backend.select(this, start, end);
  }

  /**
   * A `g-input`'s selection in UTF-16 offsets, and whether the cursor is on its first and last row
   * as drawn, where a line that wraps is several rows. Null until the field has been drawn.
   */
  fieldState(): { start: number; end: number; firstRow: boolean; lastRow: boolean } | null {
    const [start, end, row, rows] = backend.fieldState(this as unknown as SElement);
    if (start === undefined) {
      return null;
    }
    return { start, end, firstRow: row === 0, lastRow: row === rows - 1 };
  }

  scrollTo(first: number | { left?: number; top?: number }, second?: number): void {
    if (typeof first === "object") {
      this.scrollLeft = first.left ?? this.scrollLeftValue;
      this.scrollTop = first.top ?? this.scrollTopValue;
      return;
    }
    this.scrollLeft = first;
    this.scrollTop = second ?? this.scrollTopValue;
  }

  scrollBy(first: number | { left?: number; top?: number }, second?: number): void {
    if (typeof first === "object") {
      this.scrollTo(this.scrollLeftValue + (first.left ?? 0), this.scrollTopValue + (first.top ?? 0));
      return;
    }
    this.scrollTo(this.scrollLeftValue + first, this.scrollTopValue + (second ?? 0));
  }

  scrollIntoView(): void {}

  getBoundingClientRect() {
    const width = this.offsetWidth;
    const height = this.offsetHeight;
    const left = this.boxLeft;
    const top = this.boxTop;
    return { x: left, y: top, top, left, right: left + width, bottom: top + height, width, height };
  }

  override addEventListener(type: string, handler: EventHandler): void {
    let handlers = this.listeners.get(type);
    if (handlers === undefined) {
      handlers = new Set();
      this.listeners.set(type, handlers);
    }
    handlers.add(handler);
    backend.listenerAdded(this, type, handler);
  }

  override removeEventListener(type: string, handler: EventHandler): void {
    this.listeners.get(type)?.delete(handler);
    backend.listenerRemoved(this, type);
  }

  /** Runs the listeners of an event the app made up, bubbling if it asked to. */
  override dispatchEvent(event: any): boolean {
    event.target = this;
    for (let node: SNode | null = this; node !== null; node = node.parentNode) {
      if (!(node instanceof SElement)) {
        continue;
      }
      event.currentTarget = node;
      for (const handler of [...(node.listeners.get(event.type) ?? [])]) {
        handler.call(node, event);
      }
      if (!event.bubbles || event.cancelBubble) {
        break;
      }
    }
    return !event.defaultPrevented;
  }

  override cloneNode(deep?: boolean): SElement {
    const copy = new SElement(this.localName);
    for (const [name, value] of this.attributes) {
      copy.setAttribute(name, value);
    }
    for (const [name, value] of this.style.declarations) {
      copy.style.setProperty(name, value);
    }
    if (deep !== true) {
      return copy;
    }
    let child = this.firstChildNode;
    while (child !== null) {
      copy.appendChild(child.cloneNode(true));
      child = child.nextSiblingNode;
    }
    return copy;
  }
}

// Assigned by parse.ts to keep the parser out of this module's imports.
export let parseInto: (target: SNode, html: string) => void = () => {};

export function setParser(fn: (target: SNode, html: string) => void): void {
  parseInto = fn;
}
