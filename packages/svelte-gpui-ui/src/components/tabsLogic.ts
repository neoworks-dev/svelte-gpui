import { nextFocusIndex } from "./rovingFocus";
import { isActivationKey } from "./navigation";
import type { TabItem } from "./types";

export type TabKeyAction =
  | { kind: "move"; index: number }
  | { kind: "select" }
  | { kind: "none" };

/** What a key does on the tab at `index`: the arrow keys, Home and End move to an enabled tab and choose it, Enter and Space choose this one. */
export function tabKeyAction(key: string, tabs: TabItem[], index: number): TabKeyAction {
  if (isActivationKey(key)) {
    return { kind: "select" };
  }
  const disabledStates = tabs.map((tab) => tab.disabled === true);
  const target = nextFocusIndex(key, index, disabledStates, "horizontal");
  if (target === undefined) {
    return { kind: "none" };
  }
  return { kind: "move", index: target };
}
