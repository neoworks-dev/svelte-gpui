<!--
  Pixels to paint into. The host keeps a buffer at the window's scale and rasterises
  each call as it arrives, so drawing costs the same however much is already there.
  Coordinates are in the canvas's own pixels, from its top left, as pointer events
  report them. Resizing keeps what was drawn, from the top left.

    <Canvas bind:this={pad} ondown={(e) => pad.begin(e.x, e.y, { color, width: 4 })}
            onmove={(e) => pad.lineTo(e.x, e.y)} onup={() => pad.end()} />

  With `vector`, the host keeps the shapes drawn since the last `clear` and paints them
  as paths on every frame, uploading no pixels. That suits a drawing that is cleared and
  drawn again as it moves, such as an arrow, and not one that piles up stroke on stroke.
-->
<script lang="ts">
  import { draw } from "../host/bridge";
  import { encode, type Color, type Pen, type Style } from "./style";
  import { itself, listener, type Layout, type PointerEvent, type WheelEvent } from "./events";

  interface Props {
    style?: Style;
    /** Keeps shapes and paints them as paths instead of pixels. Set it when the canvas is made. */
    vector?: boolean;
    ondown?: (event: PointerEvent) => void;
    onmove?: (event: PointerEvent) => void;
    onup?: (event: PointerEvent) => void;
    onhover?: (hovered: boolean) => void;
    onwheel?: (event: WheelEvent) => void;
    onlayout?: (layout: Layout) => void;
  }

  let props: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    vector: props.vector ? "" : undefined,
    ondown: listener(props.ondown, itself<PointerEvent>),
    onmove: listener(props.onmove, itself<PointerEvent>),
    onup: listener(props.onup, itself<PointerEvent>),
    onhover: listener(props.onhover, (event: { hovered: boolean }) => event.hovered),
    onwheel: listener(props.onwheel, itself<WheelEvent>),
    onlayout: listener(props.onlayout, itself<Layout>),
  });

  let element: any;

  /** Clears to transparent, or fills everything with a colour. */
  export function clear(color?: Color): void {
    draw(element, color === undefined ? ["clear"] : ["clear", color]);
  }

  /** Starts a stroke at a point, drawing a dot there. */
  export function begin(x: number, y: number, pen: Pen = {}): void {
    draw(element, ["begin", x, y, pen.color ?? "#ffffff", pen.width ?? 2]);
  }

  /** Continues the stroke to a point. */
  export function lineTo(x: number, y: number): void {
    draw(element, ["to", x, y]);
  }

  /** Ends the stroke. */
  export function end(): void {
    draw(element, ["end"]);
  }

  export function line(x0: number, y0: number, x1: number, y1: number, pen: Pen = {}): void {
    draw(element, ["line", x0, y0, x1, y1, pen.color ?? "#ffffff", pen.width ?? 1]);
  }

  export function fillRect(x: number, y: number, width: number, height: number, color: Color): void {
    draw(element, ["rect", x, y, width, height, color]);
  }

  export function fillCircle(x: number, y: number, radius: number, color: Color): void {
    draw(element, ["circle", x, y, radius, color]);
  }
</script>

<g-canvas bind:this={element} {...attributes}></g-canvas>
