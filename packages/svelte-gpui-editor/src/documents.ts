// The documents the Rust host keeps, which the editor element shows by id. The commands and
// events are the ones `view/src/documents.rs` registers; an app adds them to its host with
// `SharedDocuments::register`.

import { invoke, listen } from "@neoworks/svelte-gpui";
import type { Decoration } from "./decorations";

/** How a document is indented. */
export type IndentStyle = { kind: "tabs" } | { kind: "spaces"; width: number };

/** What the editor's settings tell the host to do. */
export interface EditorOptions {
  /** The indentation of files whose own lines do not say. */
  defaultIndent: IndentStyle;
  smoothScroll: boolean;
  indentGuides: boolean;
  occurrenceHighlights: boolean;
}

/** A document as the host describes it. */
export interface DocumentSummary {
  id: number;
  path: string | null;
  title: string;
  dirty: boolean;
  /** How the file is indented; a host that does not say leaves it out. */
  indent?: IndentStyle;
}

/** What the host tells the app about its documents. */
export type DocumentEvent =
  | { type: "opened"; summary: DocumentSummary }
  | { type: "closed"; id: number }
  | { type: "dirty"; id: number; dirty: boolean }
  | { type: "saved"; id: number }
  | { type: "edited"; id: number }
  | { type: "syntax"; id: number }
  | { type: "error"; id: number; message: string }
  | { type: "selections"; id: number; row: number; column: number; count: number; selected: boolean }
  | { type: "mode"; pane: string; mode: string; pending: string; commandLine: string | null }
  | { type: "indent"; id: number; indent: IndentStyle }
  | { type: "close-request"; id: number; discard: boolean }
  | { type: "open-request"; id: number; path: string }
  | { type: "change-request"; id: number; forward: boolean }
  | { type: "notice"; message: string };

/** The documents the Rust host keeps. A test supplies a fake. */
export interface DocumentHost {
  /** Opens a file, or finds the document that already has it. Rejects with the reason it failed. */
  open(path: string): Promise<DocumentSummary>;
  /** Closes a document, throwing away unsaved changes. */
  close(id: number): Promise<void>;
  /** Writes a document to its file. Rejects with the reason it failed. */
  save(id: number): Promise<void>;
  /** Replaces what one source has put on top of a document's text. An empty list removes the source's. */
  setDecorations(id: number, source: string, decorations: Decoration[]): Promise<void>;
  /** Tells the host what the editor's settings say. A host that has nothing to configure leaves it out. */
  configure?(options: EditorOptions): Promise<void>;
  /** Makes a document of text that belongs to no file; `name`'s extension says what language it is. */
  openText(name: string, text: string): Promise<DocumentSummary>;
  /** The text of a document. */
  text(id: number): Promise<string>;
  /** Replaces the text of a document made by `openText`. */
  setText(id: number, text: string): Promise<void>;
  /** Reads a document's file from disk again, dropping its edits. */
  reload(id: number): Promise<void>;
  /**
   * Takes what another program wrote to a document's file, keeping its cursors and history.
   * Leaves a document with unsaved changes alone. Resolves to whether the text changed.
   */
  sync(id: number): Promise<boolean>;
  /** Hears of everything that happens to documents. Returns the function that stops it. */
  onEvent(handler: (event: DocumentEvent) => void): () => void;
}

/** Documents kept by the native host, reached through svelte-gpui's `invoke` and `listen`. */
export function gpuiDocumentHost(): DocumentHost {
  return {
    open: (path) => invoke<DocumentSummary>("document_open", { path }),
    close: async (id) => {
      await invoke("document_close", { id });
    },
    save: async (id) => {
      await invoke("document_save", { id });
    },
    setDecorations: async (id, source, decorations) => {
      await invoke("document_set_decorations", { id, source, decorations });
    },
    configure: async (options) => {
      await invoke("editor_configure", options);
    },
    openText: (name, text) => invoke<DocumentSummary>("document_open_text", { name, text }),
    text: (id) => invoke<string>("document_text", { id }),
    setText: async (id, text) => {
      await invoke("document_set_text", { id, text });
    },
    reload: async (id) => {
      await invoke("document_reload", { id });
    },
    sync: (id) => invoke<boolean>("document_sync", { id }),
    onEvent: listenToDocuments,
  };
}

/** Passes the host's `document-*` events on as `DocumentEvent`s, and returns the function that stops. */
function listenToDocuments(handler: (event: DocumentEvent) => void): () => void {
  const stops = [
    listen<DocumentSummary>("document-opened", (summary) => handler({ type: "opened", summary })),
    listen<{ id: number }>("document-closed", (event) => handler({ type: "closed", id: event.id })),
    listen<{ id: number; dirty: boolean }>("document-dirty", (event) => handler({ type: "dirty", ...event })),
    listen<{ id: number }>("document-saved", (event) => handler({ type: "saved", id: event.id })),
    listen<{ id: number }>("document-edited", (event) => handler({ type: "edited", id: event.id })),
    listen<{ id: number }>("document-syntax", (event) => handler({ type: "syntax", id: event.id })),
    listen<{ id: number; message: string }>("document-error", (event) => handler({ type: "error", ...event })),
    listen<{ message: string }>("editor-notice", (event) => handler({ type: "notice", message: event.message })),
    listen<{ id: number; row: number; column: number; count: number; selected: boolean }>(
      "document-selections",
      (event) => handler({ type: "selections", ...event }),
    ),
    listen<{ id: number; indent: IndentStyle }>("document-indent", (event) => handler({ type: "indent", ...event })),
    listen<{ pane: string; mode: string; pending: string; commandLine: string | null }>("editor-mode", (event) =>
      handler({ type: "mode", ...event }),
    ),
    listen<{ id: number; request: "close" | "open" | "change"; discard?: boolean; path?: string; forward?: boolean }>(
      "document-request",
      (event) => {
        if (event.request === "close") {
          handler({ type: "close-request", id: event.id, discard: event.discard === true });
          return;
        }
        if (event.request === "change") {
          handler({ type: "change-request", id: event.id, forward: event.forward === true });
          return;
        }
        handler({ type: "open-request", id: event.id, path: event.path ?? "" });
      },
    ),
  ];
  return () => {
    for (const stop of stops) {
      stop();
    }
  };
}
