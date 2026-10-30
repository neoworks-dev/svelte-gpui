// The editor element, which `view/src/lib.rs` draws. An app registers the Rust view under the name
// "editor" with `Elements::register`, and shows a document of its `DocumentHost` by passing its id.

import { element } from "@neoworks/svelte-gpui";
import type { TextPosition } from "./decorations";

/**
 * A syntax colour, or one with a weight and slant, and a font family and size of its own. A line
 * is drawn at the largest size any text on it asks for.
 */
export type SyntaxStyle =
  | string
  | { color: string; bold?: boolean; italic?: boolean; family?: string; size?: number };

/** What the editor's `theme` prop carries (documented in `view/src/theme.rs`). */
export interface EditorTheme {
  background: string;
  foreground: string;
  currentLine: string;
  lineNumber: string;
  currentLineNumber: string;
  selection: string;
  cursor: string;
  /** What decorations are drawn in: diagnostics, git changes, inlay hints and text after a line. */
  error: string;
  warning: string;
  info: string;
  hint: string;
  added: string;
  modified: string;
  deleted: string;
  inlay: string;
  lineEnd: string;
  /** The indentation guides, and the one around the cursor. */
  indentGuide: string;
  indentGuideActive: string;
  /** Behind the other occurrences of the word under the cursor. */
  occurrence: string;
  /** The jump command's labels: their background and text, and the background of what they label. */
  jumpLabel: string;
  jumpLabelText: string;
  jumpMatch: string;
  /** The font the text is drawn in and its size in pixels; the editor's monospace font at 14 by default. */
  fontFamily: string;
  fontSize: number;
  /** Styles by highlight name; a name left out takes its dotted prefix's style, else the foreground. */
  syntax: Record<string, SyntaxStyle>;
}

/** Where the editor asks to go when a definition is in another file. */
export interface GotoEvent {
  path: string;
  line: number;
  character: number;
}

/** Where the editor says its lines and keyed blocks are drawn, in pixels from the top left of the editor. */
export interface PlacesEvent {
  textLeft: number;
  lineHeight: number;
  rows: { row: number; top: number }[];
  blocks: { key: string; top: number; height: number }[];
}

/** A selection as the positions of its two ends, counted from zero; the ends are equal for a bare cursor. */
export interface EditorSelection {
  anchor: TextPosition;
  head: TextPosition;
}

/** What the editor element takes. */
export interface EditorProps {
  /** The id of the document to show; none shows an empty view. */
  document?: number | null;
  /** The colours to paint with; a colour left out keeps the view's default. */
  theme?: Partial<EditorTheme>;
  /** Refuses commands that change the text. */
  readonly?: boolean;
  /** The view's own decoration source, which only this view draws. */
  viewId?: string;
  /** Document-wide decoration sources this view leaves out. */
  hideSources?: string[];
  /** A document drawn read-only left of `document`, as the old side of a diff side by side. */
  companion?: number | null;
  /** What the two sides of a diff side by side are called, as `[old, new]`. */
  sideLabels?: string[];
  /** How wide the minimap is; zero for none. */
  minimap?: number;
  /** Breaks lines too wide for the view instead of scrolling sideways. A side-by-side diff never wraps. */
  wrap?: boolean;
  /** Shows line numbers in a gutter, which is the default; without them the text starts at the left edge. */
  lineNumbers?: boolean;
  /** Views in the same group scroll together. */
  scrollGroup?: string;
  /** Whether the app's keymap waits for the next key of a sequence, which the view then leaves alone. */
  sequencePending?: boolean;
  /** Whether the app has a leader key bound, for which the input layer leaves a bare space. */
  leaderKey?: boolean;
  /** The pane the view is in, which the `editor-mode` event names. */
  pane?: string;
  keymap?: "standard" | "vim";
  ongoto?: (event: GotoEvent) => void;
  onplaces?: (event: PlacesEvent) => void;
}

/** The methods the editor element has. */
export interface EditorViewMethods {
  apply(command: { type: string; [field: string]: unknown }): Promise<{ changed: boolean }>;
  /** Draws the view again, such as after the decorations of its document changed. */
  refresh(): Promise<void>;
  /** Shows what the language server says about the symbol at the cursor. */
  hover(): Promise<void>;
  /** Goes to the definition of the symbol at the cursor. */
  definition(): Promise<void>;
  /** Opens the completion panel at the cursor. */
  complete(): Promise<void>;
  /** Lists where the symbol at the cursor is used. */
  references(): Promise<void>;
  /** Goes to the type of the symbol at the cursor. */
  typeDefinition(): Promise<void>;
  /** Goes to the implementations of the symbol at the cursor. */
  implementation(): Promise<void>;
  /** Asks for a new name for the symbol at the cursor and renames it everywhere. */
  rename(): Promise<void>;
  /** Lists the fixes and refactors the language server offers at the cursor. */
  codeActions(): Promise<void>;
  /** Shows the signature of the call the cursor is in. */
  signature(): Promise<void>;
  /** Formats the document. */
  format(): Promise<void>;
  /** Scrolls so the row under a pointer at this y in the window, over the minimap, is in the middle of the editor. */
  minimapScroll(windowY: number): Promise<void>;
  /** Puts the cursor at a line and UTF-16 column, counted from zero, and scrolls to it. */
  goto(line: number, character: number): Promise<void>;
  /**
   * Handles a right press at a point in window coordinates: the cursor moves there unless a
   * selection covers it. Answers whether text is selected afterwards. A view that has no menu,
   * such as a test's stand-in, leaves it out.
   */
  contextClick?(x: number, y: number): Promise<{ selected: boolean }>;
  /** Every selection, as the positions of its two ends. A view that cannot say, such as a test's stand-in, leaves it out. */
  selections?(): Promise<EditorSelection[]>;
}

/** The editor, as a component. `bind:this` gives its `EditorViewMethods`. */
export const Editor = element<EditorProps, EditorViewMethods>("editor");
