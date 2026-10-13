// Order matters: the DOM globals have to exist before Svelte's client runtime
// is evaluated, and this is the module every entry point imports first.
import { pump } from "./dom/globals";

import { flushSync, mount } from "svelte";
import { SElement } from "./dom/nodes";
import { adoptRoot, flush, install } from "./host/bridge";
import { dispatch } from "./host/events";

declare const host: { apply(batch: string): void };

/**
 * Mounts a component into the host's root, and exposes the two entry points the
 * host calls back into: `__event` for input, and `__pump` for the clock.
 */
export function start(Component: unknown, props: Record<string, unknown> = {}): void {
  install();
  const root = new SElement("#root");
  adoptRoot(root);

  const settle = (): void => {
    flushSync();
    flush();
  };

  mount(Component as any, { target: root as any, props });
  settle();

  const globals = globalThis as any;
  globals.__settle = settle;
  globals.__event = (id: number, type: string, detail: string): void => {
    dispatch(id, type, JSON.parse(detail));
    settle();
  };
  globals.__pump = (clock: number): void => {
    if (pump(clock)) {
      settle();
    }
  };
}
