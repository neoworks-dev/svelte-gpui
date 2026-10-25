// Turns the NeoWorks design system's tokens.css into the TypeScript module the theme is
// built from. The CSS is the source of truth; the colours, which it writes in oklch() and
// color-mix(), come out as the hex and rgba() strings svelte-gpui's style parser reads.

import { gamutOverflow, mixOklch, parseOklch, toCssColor, transparentColor, type Oklch } from "./color";

interface Block {
  selectors: string[];
  declarations: [string, string][];
}

type Scope = Map<string, string>;

interface ScopeDefinition {
  name: string;
  selectors: string[];
}

/** The CSS scopes that become themes. A block applies to a scope when it names one of its selectors. */
const scopeDefinitions: ScopeDefinition[] = [
  { name: "light", selectors: [":root"] },
  { name: "dark", selectors: [":root", ".scope-dark"] },
  { name: "lightInverse", selectors: [":root", ".scope-inverse"] },
  { name: "darkInverse", selectors: [":root", ".scope-dark", ".scope-inverse", ".scope-dark .scope-inverse"] },
];

const rootFontSize = 16;
const gamutToleranceInBytes = 0.5;

// ── Reading the stylesheet ────────────────────────────────────

export function parseBlocks(stylesheet: string): Block[] {
  const withoutComments = stylesheet.replace(/\/\*[\s\S]*?\*\//g, "");
  const blocks: Block[] = [];
  for (const match of withoutComments.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const selectors = match[1].split(",").map((selector) => selector.trim().replace(/\s+/g, " "));
    blocks.push({ selectors, declarations: parseDeclarations(match[2]) });
  }
  return blocks;
}

function parseDeclarations(body: string): [string, string][] {
  const declarations: [string, string][] = [];
  for (const declaration of body.split(";")) {
    const colon = declaration.indexOf(":");
    if (colon !== -1) {
      declarations.push([declaration.slice(0, colon).trim(), declaration.slice(colon + 1).trim()]);
    }
  }
  return declarations;
}

function buildScope(blocks: Block[], selectors: string[]): Scope {
  const scope: Scope = new Map();
  for (const block of blocks) {
    if (block.selectors.some((selector) => selectors.includes(selector))) {
      block.declarations.forEach(([property, value]) => scope.set(property, value));
    }
  }
  return scope;
}

function themeDeclarations(blocks: Block[], selector: string): [string, string][] {
  return blocks.filter((block) => block.selectors[0] === selector).flatMap((block) => block.declarations);
}

function isSeparator(character: string, separator: "," | " "): boolean {
  if (separator === ",") {
    return character === ",";
  }
  return /\s/.test(character);
}

/** Splits at the separator wherever it is not inside parentheses. Whitespace separators collapse. */
export function splitOutsideParentheses(text: string, separator: "," | " "): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = "";
  for (const character of text) {
    if (character === "(") {
      depth++;
    }
    if (character === ")") {
      depth--;
    }
    if (depth === 0 && isSeparator(character, separator)) {
      parts.push(current);
      current = "";
      continue;
    }
    current += character;
  }
  parts.push(current);
  return parts.map((part) => part.trim()).filter((part) => part.length > 0);
}

// ── Colours ───────────────────────────────────────────────────

interface Mixed {
  color: string;
  share: number | null;
}

function parseMixOperand(operand: string): Mixed {
  const match = /^(.*?)(?:\s+(\d+(?:\.\d+)?)%)?$/s.exec(operand);
  if (match === null || match[2] === undefined) {
    return { color: operand, share: null };
  }
  return { color: match[1], share: Number(match[2]) / 100 };
}

function resolveMixShares(first: Mixed, second: Mixed): number {
  if (first.share !== null) {
    return first.share;
  }
  if (second.share !== null) {
    return 1 - second.share;
  }
  return 0.5;
}

function evaluateMix(arguments_: string, scope: Scope): Oklch {
  const [space, firstOperand, secondOperand] = splitOutsideParentheses(arguments_, ",");
  if (space !== "in oklch" || secondOperand === undefined) {
    throw new Error(`unsupported color-mix(${arguments_})`);
  }
  const first = parseMixOperand(firstOperand);
  const second = parseMixOperand(secondOperand);
  const firstShare = resolveMixShares(first, second);
  return mixOklch(evaluateColor(first.color, scope), firstShare, evaluateColor(second.color, scope));
}

/** Resolves `var()`, `oklch()`, `color-mix(in oklch, …)` and `transparent` as the browser would in this scope. */
export function evaluateColor(value: string, scope: Scope): Oklch {
  const text = value.trim();
  if (text === "transparent") {
    return transparentColor;
  }
  if (text.startsWith("var(")) {
    const name = text.slice(4, -1).trim();
    const target = scope.get(name);
    if (target === undefined) {
      throw new Error(`${name} is not defined`);
    }
    return evaluateColor(target, scope);
  }
  if (text.startsWith("color-mix(")) {
    return evaluateMix(text.slice(10, -1), scope);
  }
  return parseOklch(text);
}

class ColorWriter {
  readonly gamutWarnings: string[] = [];

  write(color: Oklch, source: string): string {
    if (gamutOverflow(color) > gamutToleranceInBytes) {
      this.gamutWarnings.push(`${source} lies outside sRGB and was clipped`);
    }
    return toCssColor(color);
  }
}

function camelCase(name: string): string {
  return name.replace(/-([a-z])/g, (_match, letter: string) => letter.toUpperCase());
}

function themeColors(inlineDeclarations: [string, string][], scope: Scope, writer: ColorWriter) {
  const colors: Record<string, string> = {};
  for (const [property, value] of inlineDeclarations) {
    if (property.startsWith("--color-") && value.startsWith("var(")) {
      const target = value.slice(4, -1).trim();
      colors[camelCase(property.slice(8))] = writer.write(evaluateColor(value, scope), `${property} (${target})`);
    }
  }
  return colors;
}

function scaleNamed(scales: Record<string, Record<number, string>>, name: string): Record<number, string> {
  if (scales[name] === undefined) {
    scales[name] = {};
  }
  return scales[name];
}

function themeScales(scope: Scope, writer: ColorWriter) {
  const scales: Record<string, Record<number, string>> = {};
  for (const [property, value] of scope) {
    const match = /^--([a-z]+)-(\d{1,2})$/.exec(property);
    if (match !== null && Number(match[2]) >= 1 && Number(match[2]) <= 12) {
      const scale = scaleNamed(scales, match[1]);
      scale[Number(match[2])] = writer.write(evaluateColor(value, scope), property);
    }
  }
  return scales;
}

// ── Lengths, type, radii, shadows, easing ─────────────────────

function toPixels(length: string): number {
  const match = /^(-?[\d.]+)(rem|px)?$/.exec(length.trim());
  if (match === null) {
    throw new Error(`not a length: ${length}`);
  }
  if (match[2] === "rem") {
    return Number(match[1]) * rootFontSize;
  }
  return Number(match[1]);
}

export interface FontSizeRange {
  minimum: number;
  viewportPercent: number;
  maximum: number;
}

/** `clamp(min, Nvw, max)` keeps its three parts; a fixed size has the same minimum and maximum. */
export function parseFontSize(value: string): FontSizeRange {
  if (!value.startsWith("clamp(")) {
    const size = toPixels(value);
    return { minimum: size, viewportPercent: 0, maximum: size };
  }
  const [minimum, preferred, maximum] = splitOutsideParentheses(value.slice(6, -1), ",");
  return { minimum: toPixels(minimum), viewportPercent: Number(preferred.replace("vw", "")), maximum: toPixels(maximum) };
}

interface TypeRoleTokens {
  fontSize: FontSizeRange;
  lineHeight: number;
  letterSpacing: number;
  fontWeight: number;
}

function typeRoleNames(declarations: [string, string][]): string[] {
  const names = new Set<string>();
  for (const [property] of declarations) {
    const match = /^--text-([a-z-]+?)(?:--[a-z-]+)?$/.exec(property);
    if (match !== null) {
      names.add(match[1]);
    }
  }
  return [...names];
}

function requiredValue(values: Map<string, string>, property: string): string {
  const value = values.get(property);
  if (value === undefined) {
    throw new Error(`${property} is not defined`);
  }
  return value;
}

function readNumber(values: Map<string, string>, property: string, fallback: number): number {
  const value = values.get(property);
  if (value === undefined) {
    return fallback;
  }
  return Number(value.replace("em", ""));
}

function typeScale(declarations: [string, string][]): Record<string, TypeRoleTokens> {
  const values = new Map(declarations);
  const scale: Record<string, TypeRoleTokens> = {};
  for (const name of typeRoleNames(declarations)) {
    scale[camelCase(name)] = {
      fontSize: parseFontSize(requiredValue(values, `--text-${name}`)),
      lineHeight: Number(requiredValue(values, `--text-${name}--line-height`)),
      // CSS leaves these at normal (0 and 400) when a role does not set them.
      letterSpacing: readNumber(values, `--text-${name}--letter-spacing`, 0),
      fontWeight: readNumber(values, `--text-${name}--font-weight`, 400),
    };
  }
  return scale;
}

function lengthTokens(declarations: [string, string][], prefix: string): Record<string, number> {
  const tokens: Record<string, number> = {};
  for (const [property, value] of declarations) {
    if (property.startsWith(prefix) && value !== "initial") {
      tokens[camelCase(property.slice(prefix.length))] = toPixels(value);
    }
  }
  return tokens;
}

function easingTokens(declarations: [string, string][]): Record<string, number[]> {
  const tokens: Record<string, number[]> = {};
  for (const [property, value] of declarations) {
    if (property.startsWith("--ease-")) {
      const curve = splitOutsideParentheses(value.slice(value.indexOf("(") + 1, -1), ",");
      tokens[camelCase(property.slice(7))] = curve.map(Number);
    }
  }
  return tokens;
}

interface ShadowTokens {
  x: number;
  y: number;
  blur: number;
  spread: number;
  color: string;
}

function parseShadow(layer: string, scope: Scope, writer: ColorWriter): ShadowTokens {
  const parts = splitOutsideParentheses(layer, " ");
  const color = parts[parts.length - 1];
  const [x, y, blur, spread = 0] = parts.slice(0, -1).map(toPixels);
  if (blur === undefined || parts.includes("inset")) {
    throw new Error(`unsupported shadow: ${layer}`);
  }
  const resolved = writer.write(evaluateColor(color, scope), `shadow colour ${color}`);
  return { x, y, blur, spread, color: resolved };
}

function themeShadows(declarations: [string, string][], scope: Scope, writer: ColorWriter) {
  const shadows: Record<string, ShadowTokens[]> = {};
  for (const [property, value] of declarations) {
    if (property.startsWith("--shadow-") && value !== "initial") {
      const layers = splitOutsideParentheses(value, ",");
      shadows[camelCase(property.slice(9))] = layers.map((layer) => parseShadow(layer, scope, writer));
    }
  }
  return shadows;
}

// ── Output ────────────────────────────────────────────────────

const identifierPattern = /^[A-Za-z_$][\w$]*$/;
const indentation = "  ";

function formatKey(key: string): string {
  if (identifierPattern.test(key) || /^\d+$/.test(key)) {
    return key;
  }
  return JSON.stringify(key);
}

function isFlat(value: object): boolean {
  return Object.values(value).every((entry) => typeof entry !== "object");
}

function formatObject(value: Record<string, unknown>, depth: number): string {
  const entries = Object.entries(value);
  if (isFlat(value) && entries.length <= 5) {
    return `{ ${entries.map(([key, entry]) => `${formatKey(key)}: ${format(entry, depth)}`).join(", ")} }`;
  }
  const inner = indentation.repeat(depth + 1);
  const lines = entries.map(([key, entry]) => `${inner}${formatKey(key)}: ${format(entry, depth + 1)},`);
  return `{\n${lines.join("\n")}\n${indentation.repeat(depth)}}`;
}

function formatArray(value: unknown[], depth: number): string {
  if (value.every((entry) => typeof entry !== "object")) {
    return `[${value.map((entry) => format(entry, depth)).join(", ")}]`;
  }
  const inner = indentation.repeat(depth + 1);
  const lines = value.map((entry) => `${inner}${format(entry, depth + 1)},`);
  return `[\n${lines.join("\n")}\n${indentation.repeat(depth)}]`;
}

function format(value: unknown, depth: number): string {
  if (Array.isArray(value)) {
    return formatArray(value, depth);
  }
  if (typeof value === "object" && value !== null) {
    return formatObject(value as Record<string, unknown>, depth);
  }
  return JSON.stringify(value);
}

export interface GeneratedTokens {
  module: string;
  gamutWarnings: string[];
}

function generateThemes(blocks: Block[], writer: ColorWriter) {
  const inlineDeclarations = themeDeclarations(blocks, "@theme inline");
  const baseDeclarations = themeDeclarations(blocks, "@theme");
  const themes: Record<string, unknown> = {};
  for (const definition of scopeDefinitions) {
    const scope = buildScope(blocks, definition.selectors);
    themes[definition.name] = {
      colorScheme: scope.get("color-scheme"),
      colors: themeColors(inlineDeclarations, scope, writer),
      scales: themeScales(scope, writer),
      shadows: themeShadows(baseDeclarations, scope, writer),
    };
  }
  return themes;
}

export function generateTokens(stylesheet: string): GeneratedTokens {
  const blocks = parseBlocks(stylesheet);
  const baseDeclarations = themeDeclarations(blocks, "@theme");
  const writer = new ColorWriter();
  const tokens = {
    typeScale: typeScale(baseDeclarations),
    radii: lengthTokens(baseDeclarations, "--radius-"),
    easing: easingTokens(baseDeclarations),
    themes: generateThemes(blocks, writer),
  };
  const module = [
    "// Generated by scripts/generate.mjs from the NeoWorks design system's tokens.css. Do not edit.",
    "",
    `export const tokens = ${format(tokens, 0)};`,
    "",
  ].join("\n");
  return { module, gamutWarnings: [...new Set(writer.gamutWarnings)] };
}
