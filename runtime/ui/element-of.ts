// What turns an element host component into the component `element(name)` returns, for both
// targets: the native host draws the Rust view, and the browser's stand-in says there is none.

import type { Component } from "svelte";
import type { KeyEvent, Layout } from "./events";
import type { Style } from "./style";

/** The props every element takes. */
export interface ElementProps {
  /** Lays the element out, like a `View`'s. The Rust view is drawn inside the box this gives it. */
  style?: Style;
  /** Keeps the Rust view across a hot reload: the new run gets the view with the same name and key back. */
  key?: string;
  /** A key the view did not handle, on its way up to the elements around it and to `window`. */
  onkey?: (event: KeyEvent) => void;
  onfocus?: () => void;
  onblur?: () => void;
  onlayout?: (layout: Layout) => void;
  onresize?: (layout: Layout) => void;
}

/** What an instance offers: the view's methods, each answering with a promise, and `focus` and `blur`. */
export type ElementMethods<Methods extends object> = {
  [Name in keyof Methods]: Methods[Name] extends (...args: infer Args) => infer Result
    ? (...args: Args) => Promise<Awaited<Result>>
    : never;
} & {
  focus(): void;
  blur(): void;
};

/** What an element host component exports. */
interface Host {
  focus(): void;
  blur(): void;
  __call(method: string, args: unknown[]): Promise<unknown>;
}

/** The component for the element registered under `name`, drawn by `host`. */
export function elementOf<Props extends object, Methods extends object>(
  host: unknown,
  name: string,
): Component<ElementProps & Props, ElementMethods<Methods>> {
  const component = (anchor: unknown, props: Record<string | symbol, unknown>): unknown => {
    const instance = (host as any)(anchor, withElementName(props, name)) as Host;
    return withMethods(instance);
  };
  return component as unknown as Component<ElementProps & Props, ElementMethods<Methods>>;
}

/** The props the component was given, and the name of the element they are for. */
function withElementName(props: Record<string | symbol, unknown>, name: string): Record<string | symbol, unknown> {
  return new Proxy(props, {
    get: (target, key, receiver) => (key === "__element" ? name : Reflect.get(target, key, receiver)),
    has: (target, key) => key === "__element" || Reflect.has(target, key),
    ownKeys: (target) => [...Reflect.ownKeys(target), "__element"],
    getOwnPropertyDescriptor: (target, key) => {
      if (key === "__element") {
        return { configurable: true, enumerable: true, value: name };
      }
      return Reflect.getOwnPropertyDescriptor(target, key);
    },
  });
}

/** The instance `bind:this` gets: `focus` and `blur`, and any other name as a call to the Rust view's method. */
function withMethods(host: Host): unknown {
  return new Proxy(host, {
    get(target, property, receiver) {
      if (typeof property === "symbol" || property in target || property === "then") {
        return Reflect.get(target, property, receiver);
      }
      return (...args: unknown[]) => target.__call(property, args);
    },
  });
}
