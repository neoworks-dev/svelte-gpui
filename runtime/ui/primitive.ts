// What a primitive is made of. Every primitive is one host element whose attributes come from its props and
// whose listeners are its `on…` props, so it needs one effect and nothing else. A compiled `.svelte` file for
// the same thing makes a derived, a spread effect, a snippet block and a `bind:this` effect, and an app
// builds hundreds of primitives per screen. This is what Svelte's compiler would emit for the component if it
// knew that the attributes are a fixed list, written once for all of them.

import * as $ from "svelte/internal/client";
import type { Component } from "svelte";
import type { EventHandler, SElement } from "../dom/nodes";

/** What a listener hands its prop. */
export type Argument = "event" | "hovered" | "none";

export interface ListenerSpec<Props> {
  /** The host's event name. */
  event: string;
  /** The prop holding the callback. */
  prop: keyof Props & string;
  /** What the callback gets: the host's event (the default), whether the pointer is over it, or nothing. */
  argument?: Argument;
  /** A prop that, when it is true, takes the listener away. */
  unless?: keyof Props & string;
}

export interface PrimitiveSpec<Props> {
  /** The host tag, such as `g-view`. */
  tag: string;
  /** Sets every attribute the props give, with `put`. It runs in the primitive's effect, so what it reads is tracked. */
  attributes(props: Props, element: SElement): void;
  listeners: ListenerSpec<Props>[];
}

/** Sets an attribute, or takes it off when `value` is undefined. */
export function put(element: SElement, name: string, value: string | undefined): void {
  if (value !== undefined) {
    element.setAttribute(name, value);
  } else if (element.hasAttribute(name)) {
    element.removeAttribute(name);
  }
}

/** A flag as the host reads it: present with no value, or absent. */
export function flag(on: unknown): string | undefined {
  return on ? "" : undefined;
}

type Attached = (EventHandler | undefined)[];

function reader(argument: Argument | undefined): (event: any) => unknown {
  if (argument === "hovered") {
    return (event) => event.hovered;
  }
  if (argument === "none") {
    return () => undefined;
  }
  return (event) => event;
}

/** The function that calls the prop's callback as it is now: built once, it reads the callback on each event. */
function trampoline<Props>(props: Props, listener: ListenerSpec<Props>): EventHandler {
  const read = reader(listener.argument);
  const prop = listener.prop;
  return (event) => (props as any)[prop]?.(read(event));
}

function syncListeners<Props>(
  element: SElement,
  props: Props,
  listeners: ListenerSpec<Props>[],
  attached: Attached,
): void {
  for (let index = 0; index < listeners.length; index++) {
    const listener = listeners[index];
    const callback = (props as any)[listener.prop];
    const wanted = callback !== undefined && !(listener.unless !== undefined && (props as any)[listener.unless]);
    const current = attached[index];
    if (wanted && current === undefined) {
      const handler = trampoline(props, listener);
      attached[index] = handler;
      element.addEventListener(listener.event, handler);
    } else if (!wanted && current !== undefined) {
      attached[index] = undefined;
      element.removeEventListener(listener.event, current);
    }
  }
}

/** The instance `bind:this` gets from a primitive that can take focus. */
export interface Focusable {
  /** Gives it keyboard focus. It must be `focusable`. */
  focus(): void;
  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  blur(): void;
}

/**
 * Makes the component for a host tag. `exports` builds what `bind:this` gives from the element.
 * A primitive's children are rendered once, when it is made: a `children` that appears later is not drawn.
 */
export function definePrimitive<Props extends object, Exports extends object>(
  spec: PrimitiveSpec<Props>,
  exports: (element: SElement) => Exports,
): Component<Props, Exports> {
  const bare = $.from_html(`<${spec.tag}></${spec.tag}>`, 2);
  const withChildren = $.from_html(`<${spec.tag}><!></${spec.tag}>`, 2);
  const { attributes, listeners } = spec;

  const primitive = (anchor: unknown, props: Props): Exports => {
    const children = (props as { children?: (anchor: unknown) => void }).children;
    const element = (children === undefined ? bare() : withChildren()) as SElement;
    const attached: Attached = [];
    $.render_effect(() => {
      attributes(props, element);
      syncListeners(element, props, listeners, attached);
    });
    if (children !== undefined) {
      children($.child(element));
    }
    $.append(anchor, element);
    return exports(element);
  };
  return primitive as unknown as Component<Props, Exports>;
}

export function focusable(element: SElement): Focusable {
  return {
    focus: () => element.focus(),
    blur: () => element.blur(),
  };
}

export function nothing(): object {
  return {};
}
