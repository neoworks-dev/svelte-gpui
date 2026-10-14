// The only HTML this ever sees is what Svelte's compiler emits for a template:
// our own lowercase tags, comment anchors written as `<!---->`, and the
// whitespace between them. That is a small enough language to parse directly.

import {
  SComment,
  SElement,
  SNode,
  SText,
  setBuilding,
  setParser,
} from "./nodes";

function parseInto(target: SNode, html: string): void {
  // A template is a mould, not a tree that is ever shown, so parsing must not
  // build nodes — only the clones taken from it do.
  const previous = setBuilding(false);
  try {
    fill(target, html);
  } finally {
    setBuilding(previous);
  }
}

function fill(target: SNode, html: string): void {
  const stack: SNode[] = [target];
  let at = 0;

  while (at < html.length) {
    const parent = stack[stack.length - 1];
    const next = html.indexOf("<", at);

    if (next === -1) {
      appendText(parent, html.slice(at));
      return;
    }
    if (next > at) {
      appendText(parent, html.slice(at, next));
    }

    if (html.startsWith("<!--", next)) {
      const close = html.indexOf("-->", next);
      const end = close === -1 ? html.length : close;
      parent.appendChild(new SComment(html.slice(next + 4, end)));
      at = end + 3;
      continue;
    }

    if (html.startsWith("</", next)) {
      at = html.indexOf(">", next) + 1;
      if (stack.length > 1) {
        stack.pop();
      }
      continue;
    }

    at = openTag(stack, html, next);
  }
}

function openTag(stack: SNode[], html: string, start: number): number {
  const close = html.indexOf(">", start);
  const end = close === -1 ? html.length : close;
  const selfClosing = html[end - 1] === "/";
  const inner = html.slice(start + 1, selfClosing ? end - 1 : end);

  const nameEnd = firstSpace(inner);
  const element = new SElement(inner.slice(0, nameEnd));
  for (const [name, value] of readAttributes(inner.slice(nameEnd))) {
    element.setAttribute(name, value);
  }

  stack[stack.length - 1].appendChild(element);
  if (!selfClosing) {
    stack.push(element);
  }
  return end + 1;
}

function readAttributes(source: string): Array<[string, string]> {
  const out: Array<[string, string]> = [];
  let at = 0;

  while (at < source.length) {
    while (at < source.length && isSpace(source[at])) {
      at++;
    }
    const nameStart = at;
    while (at < source.length && !isSpace(source[at]) && source[at] !== "=") {
      at++;
    }
    if (at === nameStart) {
      return out;
    }
    const name = source.slice(nameStart, at);

    if (source[at] !== "=") {
      out.push([name, ""]);
      continue;
    }
    at++;
    at = readValue(source, at, name, out);
  }
  return out;
}

function readValue(
  source: string,
  at: number,
  name: string,
  out: Array<[string, string]>,
): number {
  const quote = source[at];
  if (quote !== '"' && quote !== "'") {
    const start = at;
    while (at < source.length && !isSpace(source[at])) {
      at++;
    }
    out.push([name, unescape(source.slice(start, at))]);
    return at;
  }
  const closing = source.indexOf(quote, at + 1);
  const end = closing === -1 ? source.length : closing;
  out.push([name, unescape(source.slice(at + 1, end))]);
  return end + 1;
}

function appendText(parent: SNode, text: string): void {
  if (text.length === 0) {
    return;
  }
  parent.appendChild(new SText(unescape(text)));
}

const NAMED_ENTITIES: Record<string, string> = {
  lt: "<",
  gt: ">",
  quot: '"',
  apos: "'",
  amp: "&",
  nbsp: "\u00a0",
  copy: "\u00a9",
  middot: "\u00b7",
  hellip: "\u2026",
  ndash: "\u2013",
  mdash: "\u2014",
  laquo: "\u00ab",
  raquo: "\u00bb",
};

/** Decodes the character references Svelte leaves in a template: named, decimal and hex. */
function unescape(value: string): string {
  if (!value.includes("&")) {
    return value;
  }
  return value.replace(/&(#x[0-9a-fA-F]+|#[0-9]+|[a-zA-Z]+);/g, (whole, body: string) => {
    if (body.startsWith("#x") || body.startsWith("#X")) {
      return String.fromCodePoint(parseInt(body.slice(2), 16));
    }
    if (body.startsWith("#")) {
      return String.fromCodePoint(parseInt(body.slice(1), 10));
    }
    return NAMED_ENTITIES[body] ?? whole;
  });
}

function firstSpace(source: string): number {
  for (let at = 0; at < source.length; at++) {
    if (isSpace(source[at])) {
      return at;
    }
  }
  return source.length;
}

function isSpace(char: string): boolean {
  return char === " " || char === "\n" || char === "\t" || char === "\r";
}

setParser(parseInto);
