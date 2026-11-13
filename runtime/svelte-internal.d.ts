// The part of Svelte's compiled-output runtime that `ui/primitive.ts` calls, which Svelte does not type.
declare module "svelte/internal/client" {
  export function from_html(content: string, flags: number): () => unknown;
  export function child(node: unknown): unknown;
  export function append(anchor: unknown, element: unknown): void;
  export function render_effect(fn: () => void | (() => void)): unknown;
}
