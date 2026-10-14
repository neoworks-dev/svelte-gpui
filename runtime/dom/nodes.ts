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
  focus(node: SElement): void;
  scrollTo(node: SElement, left: number, top: number): void;
  observeSize(node: SElement): void;
  select(node: SElement, start: number, end: number): void;
}

export type EventHandler = (event: any) => void;

const noBackend: NodeBackend = {
  created() {},
  attributeChanged() {},
  styleChanged() {},
  listenerAdded() {},
  focus() {},
  scrollTo() {},
  observeSize() {},
  select() {},
};

let backend = noBackend;

export function setBackend(next: NodeBackend): void {
  backend = next;
}

export class SNode {
  nodeType = 0;
  nodeName = "#node";
  parentNode: SNode | null = null;
  ownerDocument: unknown = null;
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

  /** Where `other` is relative to this node, as the DOM's bit flags: 2 before, 4 after, 8 contains, 16 contained. */
  compareDocumentPosition(other: SNode): number {
    if (other === this) {
      return 0;
    }
    const chain = (node: SNode): SNode[] => {
      const path: SNode[] = [];
      for (let current: SNode | null = node; current !== null; current = current.parentNode) {
        path.unshift(current);
      }
      return path;
    };
    const mine = chain(this);
    const theirs = chain(other);
    if (mine[0] !== theirs[0]) {
      return 1;
    }
    let depth = 0;
    while (depth < mine.length && depth < theirs.length && mine[depth] === theirs[depth]) {
      depth += 1;
    }
    if (depth === mine.length) {
      return 16 | 4;
    }
    if (depth === theirs.length) {
      return 8 | 2;
    }
    for (let sibling = mine[depth].nextSiblingNode; sibling !== null; sibling = sibling.nextSiblingNode) {
      if (sibling === theirs[depth]) {
        return 4;
      }
    }
    return 2;
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
  readonly tagName: string;
  readonly attributes = new Map<string, unknown>();
  readonly style: SStyle;
  readonly listeners = new Map<string, Set<EventHandler>>();
  content: SFragment | null = null;

  constructor(tagName: string) {
    super();
    this.tagName = tagName;
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
    this.attributes.set(name, value);
    backend.attributeChanged(this, name, value);
  }

  removeAttribute(name: string): void {
    this.attributes.delete(name);
    backend.attributeChanged(this, name, null);
  }

  /** `element.dataset`: the `data-*` attributes, by camelCase name. */
  get dataset(): Record<string, string> {
    const kebab = (name: string): string =>
      "data-" + name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
    return new Proxy({} as Record<string, string>, {
      get: (_target, name) => {
        if (typeof name !== "string") {
          return undefined;
        }
        const value = this.getAttribute(kebab(name));
        return value === null ? undefined : String(value);
      },
      set: (_target, name, value) => {
        if (typeof name === "string") {
          this.setAttribute(kebab(name), String(value));
        }
        return true;
      },
      deleteProperty: (_target, name) => {
        if (typeof name === "string") {
          this.removeAttribute(kebab(name));
        }
        return true;
      },
      has: (_target, name) => typeof name === "string" && this.hasAttribute(kebab(name)),
    });
  }

  get className(): string {
    const value = this.attributes.get("class");
    if (typeof value !== "string") {
      return "";
    }
    return value;
  }

  set className(value: string) {
    this.setAttribute("class", value);
  }

  get classList() {
    return {
      toggle: (name: string, force?: boolean) => {
        const present = this.className.split(" ").filter((n) => n.length > 0);
        const has = present.includes(name);
        const wanted = force === undefined ? !has : force;
        if (wanted === has) {
          return;
        }
        if (wanted) {
          present.push(name);
        } else {
          present.splice(present.indexOf(name), 1);
        }
        this.className = present.join(" ");
      },
      add: (name: string) => this.classList.toggle(name, true),
      remove: (name: string) => this.classList.toggle(name, false),
      contains: (name: string) => this.className.split(" ").includes(name),
    };
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

  // ---- Form controls ----------------------------------------------------
  // A control's state is an attribute, so the host sees what the app sets; what
  // the user does arrives as an event that rewrites the attribute without echoing.

  selectionStartValue = 0;
  selectionEndValue = 0;

  get value(): string {
    if (this.tagName === "select") {
      const chosen = this.selectedOptions[0] ?? this.options[0];
      return chosen === undefined ? "" : chosen.value;
    }
    const stored = this.attributes.get("value");
    if (stored !== undefined && stored !== null) {
      return String(stored);
    }
    if (this.tagName === "option" || this.tagName === "textarea") {
      return this.textContent.trim();
    }
    return "";
  }

  set value(next: unknown) {
    const text = next === null || next === undefined ? "" : String(next);
    if (this.tagName === "select") {
      const match = this.options.find((option) => option.value === text);
      this.selectedIndex = match === undefined ? -1 : this.options.indexOf(match);
      return;
    }
    if (this.attributes.get("value") === text) {
      return;
    }
    this.setAttribute("value", text);
  }

  get defaultValue(): string {
    return this.textContent;
  }

  set defaultValue(next: unknown) {
    this.setAttribute("defaultvalue", String(next));
  }

  get checked(): boolean {
    const stored = this.attributes.get("checked");
    return stored === true || stored === "true" || stored === "";
  }

  set checked(next: unknown) {
    this.setAttribute("checked", Boolean(next));
  }

  get defaultChecked(): boolean {
    return this.hasAttribute("defaultchecked");
  }

  set defaultChecked(next: unknown) {
    this.setAttribute("defaultchecked", Boolean(next));
  }

  get selected(): boolean {
    const stored = this.attributes.get("selected");
    return stored === true || stored === "true" || stored === "";
  }

  set selected(next: unknown) {
    const wanted = Boolean(next);
    if (wanted && this.tagName === "option") {
      const select = this.owningSelect();
      if (select !== null && !select.multiple) {
        for (const other of select.options) {
          if (other !== this && other.selected) {
            other.setAttribute("selected", false);
          }
        }
      }
    }
    this.setAttribute("selected", wanted);
  }

  get type(): string {
    const stored = this.attributes.get("type");
    if (typeof stored === "string" && stored !== "") {
      return stored.toLowerCase();
    }
    return this.tagName === "input" ? "text" : "";
  }

  set type(next: string) {
    this.setAttribute("type", next);
  }

  get multiple(): boolean {
    return this.hasAttribute("multiple") && this.attributes.get("multiple") !== false;
  }

  get selectionStart(): number {
    return this.selectionStartValue;
  }

  set selectionStart(next: number) {
    this.selectionStartValue = next;
    backend.select(this, next, this.selectionEndValue);
  }

  get selectionEnd(): number {
    return this.selectionEndValue;
  }

  set selectionEnd(next: number) {
    this.selectionEndValue = next;
    backend.select(this, this.selectionStartValue, next);
  }

  setSelectionRange(start: number, end: number): void {
    this.selectionStartValue = start;
    this.selectionEndValue = end;
    backend.select(this, start, end);
  }

  select(): void {
    this.setSelectionRange(0, this.value.length);
  }

  /** The `option` elements of a select, in order, wherever they sit inside it. */
  get options(): SElement[] {
    const found: SElement[] = [];
    const visit = (node: SNode): void => {
      for (let child = node.firstChildNode; child !== null; child = child.nextSiblingNode) {
        if (child instanceof SElement && child.tagName === "option") {
          found.push(child);
          continue;
        }
        visit(child);
      }
    };
    visit(this);
    return found;
  }

  get selectedOptions(): SElement[] {
    return this.options.filter((option) => option.selected);
  }

  get selectedIndex(): number {
    return this.options.findIndex((option) => option.selected);
  }

  set selectedIndex(index: number) {
    this.options.forEach((option, position) => {
      const wanted = position === index;
      if (option.selected !== wanted) {
        option.setAttribute("selected", wanted);
      }
    });
  }

  private owningSelect(): SElement | null {
    for (let node = this.parentNode; node !== null; node = node.parentNode) {
      if (node instanceof SElement && node.tagName === "select") {
        return node;
      }
    }
    return null;
  }

  get form(): null {
    return null;
  }

  get labels(): SElement[] {
    return [];
  }

  get files(): null {
    return null;
  }

  /** The first descendant matching a simple selector: a tag, class, id, attribute or `:checked`. */
  querySelector(selector: string): SElement | null {
    return this.querySelectorAll(selector)[0] ?? null;
  }

  querySelectorAll(selector: string): SElement[] {
    const matches = simpleMatcher(selector);
    const found: SElement[] = [];
    const visit = (node: SNode): void => {
      for (let child = node.firstChildNode; child !== null; child = child.nextSiblingNode) {
        if (child instanceof SElement) {
          if (matches(child)) {
            found.push(child);
          }
          visit(child);
        }
      }
    };
    visit(this);
    return found;
  }

  /** Whether the element matches a simple selector. */
  matches(selector: string): boolean {
    return simpleMatcher(selector)(this);
  }

  /** The element itself or the nearest ancestor that matches. */
  closest(selector: string): SElement | null {
    const matches = simpleMatcher(selector);
    for (let node: SNode | null = this; node !== null; node = node.parentNode) {
      if (node instanceof SElement && matches(node)) {
        return node;
      }
    }
    return null;
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

  // ---- <canvas> ---------------------------------------------------------
  // A canvas is drawn by the host from what an app sends it, not by a 2D context;
  // the context here is only enough to measure text and to be asked for.

  private canvasWidth = 300;
  private canvasHeight = 150;

  get width(): number {
    return this.canvasWidth;
  }

  set width(value: number) {
    this.canvasWidth = Number(value);
  }

  get height(): number {
    return this.canvasHeight;
  }

  set height(value: number) {
    this.canvasHeight = Number(value);
  }

  getContext(): CanvasStub {
    return new CanvasStub();
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
    const copy = new SElement(this.tagName);
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

/** A 2D context that draws nothing and can measure text, through the host. */
class CanvasStub {
  font = "10px sans-serif";
  fillStyle = "#000";
  strokeStyle = "#000";
  lineWidth = 1;
  textBaseline = "alphabetic";
  globalAlpha = 1;

  measureText(text: string) {
    const size = Number(/(\d+(?:\.\d+)?)px/.exec(this.font)?.[1] ?? 10);
    const measured = (globalThis as any).host.measureText(size, String(text)) as number[];
    return {
      width: measured[0],
      fontBoundingBoxAscent: measured[1],
      fontBoundingBoxDescent: measured[2],
      actualBoundingBoxAscent: measured[1],
      actualBoundingBoxDescent: measured[2],
    };
  }

  save(): void {}
  restore(): void {}
  beginPath(): void {}
  moveTo(): void {}
  lineTo(): void {}
  rect(): void {}
  clip(): void {}
  stroke(): void {}
  fill(): void {}
  fillRect(): void {}
  clearRect(): void {}
  fillText(): void {}
  drawImage(): void {}
  scale(): void {}
  translate(): void {}
  setTransform(): void {}
}

/** Builds a test for the handful of selectors Svelte's runtime and apps use. */
function simpleMatcher(selector: string): (element: SElement) => boolean {
  const alternatives = selector.split(",").map((part) => part.trim());
  const tests = alternatives.map((part) => {
    const tag = /^[a-zA-Z][\w-]*/.exec(part)?.[0];
    const classes = [...part.matchAll(/\.([\w-]+)/g)].map((match) => match[1]);
    const id = /#([\w-]+)/.exec(part)?.[1];
    const attributes = [...part.matchAll(/\[([\w-]+)(?:=["']?([^\]"']*)["']?)?\]/g)];
    const checked = part.includes(":checked");
    return (element: SElement): boolean => {
      if (tag !== undefined && element.tagName !== tag.toLowerCase()) {
        return false;
      }
      if (classes.some((name) => !element.classList.contains(name))) {
        return false;
      }
      if (id !== undefined && element.getAttribute("id") !== id) {
        return false;
      }
      for (const [, name, expected] of attributes) {
        const actual = element.getAttribute(name);
        if (actual === null || (expected !== undefined && String(actual) !== expected)) {
          return false;
        }
      }
      if (checked) {
        const isChecked = element.tagName === "option" ? element.selected : element.checked;
        if (!isChecked) {
          return false;
        }
      }
      return true;
    };
  });
  return (element) => tests.some((test) => test(element));
}

// Svelte writes some names as DOM properties rather than attributes —
// `element.value = v`, `element.disabled = true` — because that is how a browser
// element behaves. Nothing here is a browser element, so each of those names
// gets an accessor that turns the write back into an attribute the host sees.
const PROPERTY_ATTRIBUTES: Record<string, string> = {
  muted: "muted",
  volume: "volume",
  open: "open",
  async: "async",
  autofocus: "autofocus",
  autoplay: "autoplay",
  controls: "controls",
  default: "default",
  disabled: "disabled",
  indeterminate: "indeterminate",
  inert: "inert",
  loop: "loop",
  multiple: "multiple",
  required: "required",
  reversed: "reversed",
  seamless: "seamless",
  webkitdirectory: "webkitdirectory",
  defer: "defer",
  formNoValidate: "formnovalidate",
  isMap: "ismap",
  noModule: "nomodule",
  playsInline: "playsinline",
  readOnly: "readonly",
  srcObject: "srcobject",
  noValidate: "novalidate",
  allowFullscreen: "allowfullscreen",
  disablePictureInPicture: "disablepictureinpicture",
  disableRemotePlayback: "disableremoteplayback",
};

for (const [property, attribute] of Object.entries(PROPERTY_ATTRIBUTES)) {
  Object.defineProperty(SElement.prototype, property, {
    get(this: SElement): unknown {
      return this.getAttribute(attribute);
    },
    set(this: SElement, value: unknown) {
      this.setAttribute(attribute, value);
    },
    configurable: true,
  });
}

// Assigned by parse.ts to keep the parser out of this module's imports.
export let parseInto: (target: SNode, html: string) => void = () => {};

export function setParser(fn: (target: SNode, html: string) => void): void {
  parseInto = fn;
}
