import type { Shadow } from "@neoworks/svelte-gpui/ui";

/** A solid ring just outside an element. The host draws no shadow with a blur of 0, so the ring has a blur of 1. */
export function ring(color: string, width: number): Shadow {
  return { x: 0, y: 0, blur: 1, spread: width, color };
}
