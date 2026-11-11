<!--
  Pixels to paint into, with a 2D context. Coordinates are in the canvas's own pixels, from its top
  left, as pointer events report them. Resizing keeps what was drawn, from the top left.

    <Canvas bind:this={pad} ondown={(e) => pad.begin(e.x, e.y, { color, width: 4 })}
            onmove={(e) => pad.lineTo(e.x, e.y)} onup={() => pad.end()} />

  With `vector`, the shapes drawn since the last `clear` are kept and painted again whenever the
  canvas is resized, instead of keeping the pixels.
-->
<script lang="ts">
  import type { Color, Pen } from "../../ui/style";
  import type { CanvasProps } from "../../ui/props";
  import { cssText, toCss } from "./css";
  import { interaction } from "./interact.svelte";
  import { watchLayout } from "./layout";

  let props: CanvasProps = $props();

  const ix = interaction(() => props);
  const css = $derived(cssText(toCss(props.style)));

  type Shape =
    | { kind: "clear"; color?: Color }
    | { kind: "dot"; x: number; y: number; pen: Required<Pen> }
    | { kind: "segment"; x0: number; y0: number; x1: number; y1: number; pen: Required<Pen> }
    | { kind: "rect"; x: number; y: number; width: number; height: number; color: Color }
    | { kind: "circle"; x: number; y: number; radius: number; color: Color };

  let canvas: HTMLCanvasElement | undefined;
  let context: CanvasRenderingContext2D | null = null;
  let width = 0;
  let height = 0;
  let ratio = 1;
  /** The shapes since the last clear, for a vector canvas. */
  let shapes: Shape[] = [];
  /** Where the stroke in progress is. */
  let last: { x: number; y: number; pen: Required<Pen> } | null = null;

  function paint(shape: Shape): void {
    const c = context;
    if (c === null) {
      return;
    }
    switch (shape.kind) {
      case "clear":
        c.clearRect(0, 0, width, height);
        if (shape.color !== undefined) {
          c.fillStyle = shape.color;
          c.fillRect(0, 0, width, height);
        }
        break;
      case "dot":
        c.fillStyle = shape.pen.color;
        c.beginPath();
        c.arc(shape.x, shape.y, shape.pen.width / 2, 0, Math.PI * 2);
        c.fill();
        break;
      case "segment":
        c.strokeStyle = shape.pen.color;
        c.lineWidth = shape.pen.width;
        c.lineCap = "round";
        c.beginPath();
        c.moveTo(shape.x0, shape.y0);
        c.lineTo(shape.x1, shape.y1);
        c.stroke();
        break;
      case "rect":
        c.fillStyle = shape.color;
        c.fillRect(shape.x, shape.y, shape.width, shape.height);
        break;
      case "circle":
        c.fillStyle = shape.color;
        c.beginPath();
        c.arc(shape.x, shape.y, shape.radius, 0, Math.PI * 2);
        c.fill();
        break;
    }
  }

  function draw(shape: Shape): void {
    if (props.vector) {
      if (shape.kind === "clear") {
        shapes = [];
      }
      shapes.push(shape);
    }
    paint(shape);
  }

  /** Gives the bitmap the canvas's size at the screen's density, keeping what was drawn from the top left. */
  function resize(box: { width: number; height: number }): void {
    const node = canvas;
    if (node === undefined) {
      return;
    }
    const kept = !props.vector && node.width > 0 && node.height > 0 ? document.createElement("canvas") : null;
    if (kept !== null) {
      kept.width = node.width;
      kept.height = node.height;
      kept.getContext("2d")!.drawImage(node, 0, 0);
    }
    width = box.width;
    height = box.height;
    ratio = window.devicePixelRatio || 1;
    node.width = Math.max(Math.round(width * ratio), 1);
    node.height = Math.max(Math.round(height * ratio), 1);
    context = node.getContext("2d");
    if (kept !== null) {
      context?.drawImage(kept, 0, 0);
    }
    context?.setTransform(ratio, 0, 0, ratio, 0, 0);
    if (props.vector) {
      for (const shape of shapes) {
        paint(shape);
      }
    }
  }

  /** Clears to transparent, or fills everything with a colour. */
  export function clear(color?: Color): void {
    draw({ kind: "clear", color });
  }

  /** Starts a stroke at a point, drawing a dot there. */
  export function begin(x: number, y: number, pen: Pen = {}): void {
    const full = { color: pen.color ?? "#ffffff", width: pen.width ?? 2 };
    last = { x, y, pen: full };
    draw({ kind: "dot", x, y, pen: full });
  }

  /** Continues the stroke to a point. */
  export function lineTo(x: number, y: number): void {
    if (last === null) {
      return;
    }
    draw({ kind: "segment", x0: last.x, y0: last.y, x1: x, y1: y, pen: last.pen });
    last = { ...last, x, y };
  }

  /** Ends the stroke. */
  export function end(): void {
    last = null;
  }

  export function line(x0: number, y0: number, x1: number, y1: number, pen: Pen = {}): void {
    draw({ kind: "segment", x0, y0, x1, y1, pen: { color: pen.color ?? "#ffffff", width: pen.width ?? 1 } });
  }

  export function fillRect(x: number, y: number, rectWidth: number, rectHeight: number, color: Color): void {
    draw({ kind: "rect", x, y, width: rectWidth, height: rectHeight, color });
  }

  export function fillCircle(x: number, y: number, radius: number, color: Color): void {
    draw({ kind: "circle", x, y, radius, color });
  }
</script>

<div
  class="g-canvas"
  class:g-pointer={ix.grabbing}
  style={css}
  {@attach ix.attach}
  {@attach (node) => {
    // Sized at once, so what the app draws as it mounts is not lost, and again whenever the box changes.
    canvas = node.querySelector("canvas") ?? undefined;
    resize(node.getBoundingClientRect());
    return watchLayout(node, resize, true);
  }}
>
  <canvas></canvas>
</div>
