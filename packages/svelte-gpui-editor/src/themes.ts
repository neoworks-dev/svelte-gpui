// Themes for the editor's `theme` prop that go beyond colouring code.

import type { EditorTheme, SyntaxStyle } from "./editor";

/** The colours a prose theme is drawn in, which an app takes from its own theme. */
export interface ProseColors {
  /** Body text. */
  text: string;
  /** Headings and strong emphasis. */
  heading: string;
  /** Markdown's own characters, such as `#`, `*` and the backticks, and link addresses. */
  muted: string;
  /** The text of links. */
  link: string;
  /** Inline code and code blocks. */
  code: string;
}

/** The sans-serif and the monospace font the host bundles. Geist has no italic face. */
const PROSE_FONT = "Geist";
const CODE_FONT = "JetBrainsMono Nerd Font Mono";

/**
 * The highlights of languages a code block is written in. A block's text is `embedded`, but what
 * its language highlights inside it wins, so those names need the code font too.
 */
const CODE_HIGHLIGHTS = [
  "attribute",
  "boolean",
  "comment",
  "constant",
  "constructor",
  "embedded",
  "enum",
  "function",
  "keyword",
  "label",
  "module",
  "number",
  "operator",
  "preproc",
  "property",
  "punctuation.bracket",
  "string",
  "tag",
  "type",
  "variable",
];

/**
 * Markdown the way a notes app shows it: the text in a sans-serif, headings larger by level,
 * emphasis in italics and bold, code in monospace, links in colour and Markdown's own characters
 * dimmed. Spread it over an app's editor theme, which keeps its window colours.
 */
export function proseTheme(colors: ProseColors): Partial<EditorTheme> {
  const code: SyntaxStyle = { color: colors.code, family: CODE_FONT };
  const syntax: Record<string, SyntaxStyle> = {};
  for (const name of CODE_HIGHLIGHTS) {
    syntax[name] = code;
  }
  Object.assign(syntax, {
    "title.1": { color: colors.heading, bold: true, size: 26 },
    "title.2": { color: colors.heading, bold: true, size: 21 },
    "title.3": { color: colors.heading, bold: true, size: 18 },
    title: { color: colors.heading, bold: true },
    emphasis: { color: colors.text, italic: true },
    "emphasis.strong": { color: colors.heading, bold: true },
    "text.literal": code,
    link_text: colors.link,
    link_uri: colors.muted,
    // Markdown's own characters share these names with the punctuation of code, whose columns
    // only line up in the code font.
    punctuation: { color: colors.muted, family: CODE_FONT },
    "string.escape": colors.muted,
  });
  return { foreground: colors.text, fontFamily: PROSE_FONT, fontSize: 15, syntax };
}
