// `element(name)` in a browser: the component the native host would draw with a Rust view is a notice
// in the element's box, and its methods reject. See `./ui/ElementHost.svelte`.

import type { Component } from "svelte";
import { elementOf, type ElementMethods, type ElementProps } from "../ui/element-of";
import ElementHost from "./ui/ElementHost.svelte";

export function element<Props extends object = {}, Methods extends object = {}>(
  name: string,
): Component<ElementProps & Props, ElementMethods<Methods>> {
  return elementOf<Props, Methods>(ElementHost, name);
}
