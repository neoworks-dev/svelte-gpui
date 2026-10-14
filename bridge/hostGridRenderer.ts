// The nvim grid, drawn by the host. Grove's renderer paints a canvas with 2D calls;
// this one has the same interface and keeps the same decisions (runs of cells that
// share a highlight, the cursor's colours), but sends each changed row to the host
// as coloured runs, and the host draws them. It stands in for Grove's
// `canvasRenderer.ts` when the app is built.

import type { DirtyState, GridState } from "@grove/src/renderer/src/lib/nvim/types";
import type { CellMetrics, FontSpec } from "@grove/src/renderer/src/lib/nvim/metrics";
import type { GridRenderer } from "@grove/src/renderer/src/lib/nvim/renderer";
import { buildRuns } from "@grove/src/renderer/src/lib/nvim/renderer";
import { highlightAttrs, resolveColors } from "@grove/src/renderer/src/lib/nvim/highlights";
import { gridOperation } from "../runtime/host/bridge";
import type { SElement } from "../runtime/dom/nodes";

// What a grid instruction says; the host's `grid.rs` reads the same names.
const CONFIGURE = "configure";
const CLEAR = "clear";
const ROW = "row";
const CURSOR = "cursor";
const COPY = "copy";
const DROP = "drop";

const BOLD = 1;
const ITALIC = 2;
const UNDERLINE = 4;
const UNDERCURL = 8;
const STRIKETHROUGH = 16;

function toNumber(css: string): number {
  return parseInt(css.slice(1), 16);
}

export class CanvasGridRenderer implements GridRenderer {
  private canvas: SElement | null = null;
  private font: FontSpec = { family: "monospace", sizePx: 13 };
  private metrics: CellMetrics = { cellWidth: 8, cellHeight: 18, baseline: 14 };
  private cols = 0;
  private rows = 0;

  attach(canvas: HTMLCanvasElement): void {
    this.canvas = canvas as unknown as SElement;
  }

  get gridCols(): number {
    return this.cols;
  }

  get gridRows(): number {
    return this.rows;
  }

  carryFrom(previous: HTMLCanvasElement): void {
    if (this.canvas === null) {
      return;
    }
    gridOperation(this.canvas, COPY, (previous as unknown as SElement).hostId);
  }

  setFont(font: FontSpec, metrics: CellMetrics): void {
    this.font = font;
    this.metrics = metrics;
  }

  resize(cols: number, rows: number, _dpr: number, pxWidth: number, pxHeight: number): void {
    if (this.canvas === null) {
      return;
    }
    this.cols = cols;
    this.rows = rows;
    // The element is as big as the pane; the host lays the cells out inside it.
    this.canvas.style.setProperty("width", `${pxWidth}px`);
    this.canvas.style.setProperty("height", `${pxHeight}px`);
    gridOperation(
      this.canvas,
      CONFIGURE,
      cols,
      rows,
      this.metrics.cellWidth,
      this.metrics.cellHeight,
      this.metrics.baseline,
      this.font.sizePx,
      this.font.family,
    );
  }

  render(state: GridState, dirty: DirtyState): void {
    const canvas = this.canvas;
    if (canvas === null) {
      return;
    }
    if (dirty.all) {
      gridOperation(canvas, CLEAR, toNumber(resolveColors(state, 0).bg));
    }
    const rows = dirty.all ? [...Array(state.rows).keys()] : [...dirty.rows];
    for (const row of rows) {
      this.sendRow(canvas, state, row);
    }
    this.sendCursor(canvas, state);
  }

  dispose(): void {
    if (this.canvas !== null) {
      gridOperation(this.canvas, DROP);
    }
    this.canvas = null;
  }

  /** One row as runs of [column, width, text, foreground, background, special, flags]. */
  private sendRow(canvas: SElement, state: GridState, row: number): void {
    const line = state.lines[row];
    if (line === undefined) {
      return;
    }
    const runs = buildRuns(line).map((run) => {
      const colors = resolveColors(state, run.hlId);
      const attrs = highlightAttrs(state, run.hlId);
      let flags = 0;
      if (attrs.bold === true) flags |= BOLD;
      if (attrs.italic === true) flags |= ITALIC;
      if (attrs.underline === true) flags |= UNDERLINE;
      if (attrs.undercurl === true) flags |= UNDERCURL;
      if (attrs.strikethrough === true) flags |= STRIKETHROUGH;
      return [run.col, run.width, run.text, toNumber(colors.fg), toNumber(colors.bg), toNumber(colors.sp), flags];
    });
    gridOperation(canvas, ROW, row, runs);
  }

  private sendCursor(canvas: SElement, state: GridState): void {
    const { row, col, visible } = state.cursor;
    const line = state.lines[row];
    if (!visible || line === undefined || col >= line.length) {
      gridOperation(canvas, CURSOR, -1, -1, 0, 0, 0, 0, "");
      return;
    }
    const mode = state.modes[state.modeIdx];
    const shape = mode?.cursorShape ?? "block";
    const percentage = (mode?.cellPercentage ?? 100) / 100;
    const cell = line[col];
    const colors = cursorColors(state, mode?.attrId, cell.hlId);
    const attrs = highlightAttrs(state, cell.hlId);
    let flags = 0;
    if (attrs.bold === true) flags |= BOLD;
    if (attrs.italic === true) flags |= ITALIC;
    const shapeCode = shape === "vertical" ? 1 : shape === "horizontal" ? 2 : 0;
    gridOperation(
      canvas,
      CURSOR,
      row,
      col,
      shapeCode,
      percentage,
      toNumber(colors.body),
      toNumber(colors.text),
      cell.text,
      flags,
    );
  }
}

/** What to paint the cursor and the character under it in; Grove's rule, unchanged. */
export function cursorColors(
  state: GridState,
  attrId: number | undefined,
  cellHlId: number,
): { body: string; text: string } {
  const cellColors = resolveColors(state, cellHlId);
  const reverseVideo = { body: cellColors.fg, text: cellColors.bg };
  if (attrId === undefined || attrId === 0) return reverseVideo;
  const attrs = highlightAttrs(state, attrId);
  if (attrs.background === undefined && attrs.reverse !== true) return reverseVideo;
  const colors = resolveColors(state, attrId);
  return { body: colors.bg, text: colors.fg };
}

export function buildEdges(count: number, total: number): number[] {
  const edges: number[] = [];
  for (let index = 0; index <= count; index += 1) {
    edges.push(Math.round((index * total) / count));
  }
  return edges;
}
