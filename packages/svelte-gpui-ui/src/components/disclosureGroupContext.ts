import { getContext, setContext } from "svelte";

export interface DisclosureGroupContext {
  /** Opening one disclosure closes the one that was open. */
  readonly exclusive: boolean;
  /** The disclosure that opened last, whether or not it is still open. */
  readonly lastOpenedId: string | undefined;
  /** Records that the disclosure opened. */
  opened: (id: string) => void;
}

const contextKey = Symbol("neoworks-disclosure-group");

export function provideDisclosureGroup(context: DisclosureGroupContext): void {
  setContext(contextKey, context);
}

export function useDisclosureGroup(): DisclosureGroupContext | undefined {
  return getContext<DisclosureGroupContext | undefined>(contextKey);
}

/** Whether an open disclosure may stay open: always, unless the group is exclusive and another one opened after it. */
export function staysOpen(exclusive: boolean, lastOpenedId: string | undefined, id: string): boolean {
  if (!exclusive || lastOpenedId === undefined) {
    return true;
  }
  return lastOpenedId === id;
}
