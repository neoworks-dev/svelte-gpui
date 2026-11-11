// Turns an element the app's Rust crate registered (`Elements`, docs/rust.md) into a component. The component
// draws the Rust view in its box; its props reach the view as JSON, its `on…` props hear the view's events,
// and the methods of the view are called on the instance `bind:this` gives.

import type { Component } from "svelte";
import ElementHost from "./ElementHost.svelte";
import { elementOf, type ElementMethods, type ElementProps } from "./element-of";

export type { ElementMethods, ElementProps } from "./element-of";

/**
 * The component for the element registered under `name` in Rust. `Props` are the props the view takes, as
 * JSON, and the listeners for the events it emits (`onsave` hears `emit("save", …)`); `Methods` are the
 * methods it answers, by the signature the Rust side gives them.
 *
 * ```svelte
 * const Editor = element<{ path: string; onsave: (text: string) => void }, { insert(text: string): void }>("editor");
 * <Editor style={{ flex: 1 }} {path} onsave={save} bind:this={editor} />
 * ```
 */
export function element<Props extends object = {}, Methods extends object = {}>(
  name: string,
): Component<ElementProps & Props, ElementMethods<Methods>> {
  return elementOf<Props, Methods>(ElementHost, name);
}
