import { getContext, setContext } from "svelte";

export interface ListContext {
  /** Rows draw a line above themselves. */
  readonly dividers: boolean;
}

const contextKey = Symbol("neoworks-list");

export function provideList(context: ListContext): void {
  setContext(contextKey, context);
}

export function useList(): ListContext | undefined {
  return getContext<ListContext | undefined>(contextKey);
}
