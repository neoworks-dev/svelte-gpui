// Keyboard movement through the rows of a menu or a select list.

import { nextFocusIndex } from "./rovingFocus";

export const typeaheadResetMilliseconds = 600;

/** The row that Arrow, Home or End moves to, or `undefined` for any other key. Nothing active yet makes Up start from the end. */
export function movedIndex(key: string, activeIndex: number, disabledStates: boolean[]): number | undefined {
  if (disabledStates.length === 0) {
    return undefined;
  }
  let from = activeIndex;
  if (activeIndex < 0 && key === "ArrowUp") {
    from = disabledStates.length;
  }
  return nextFocusIndex(key, from, disabledStates, "vertical");
}

/** The first row that can be chosen, or -1. */
export function firstEnabledIndex(disabledStates: boolean[]): number {
  return disabledStates.findIndex((disabled) => !disabled);
}

/** The last row that can be chosen, or -1. */
export function lastEnabledIndex(disabledStates: boolean[]): number {
  return disabledStates.map((disabled) => !disabled).lastIndexOf(true);
}

/** The text typed so far: `character` is added unless the last key was longer ago than the reset time. */
export function typedTextAfter(previousText: string, previousAt: number, character: string, now: number): string {
  if (now - previousAt > typeaheadResetMilliseconds) {
    return character;
  }
  return previousText + character;
}

/** The next row that can be chosen after `activeIndex` whose label starts with `typedText`, wrapping around. */
export function typeaheadIndex(typedText: string, activeIndex: number, labels: string[], disabledStates: boolean[]): number | undefined {
  const needle = typedText.toLowerCase();
  const count = labels.length;
  for (let step = 1; step <= count; step++) {
    const candidate = (((activeIndex + step) % count) + count) % count;
    if (!disabledStates[candidate] && labels[candidate].toLowerCase().startsWith(needle)) {
      return candidate;
    }
  }
  return undefined;
}

/** Whether a key press is a printable character for typeahead rather than a shortcut. */
export function isTypeaheadKey(key: string, ctrl: boolean, alt: boolean, meta: boolean): boolean {
  return key.length === 1 && key !== " " && !ctrl && !alt && !meta;
}
