export type Orientation = "horizontal" | "vertical" | "both";

const forwardKeys: Record<Orientation, string[]> = {
  horizontal: ["ArrowRight"],
  vertical: ["ArrowDown"],
  both: ["ArrowRight", "ArrowDown"],
};

const backwardKeys: Record<Orientation, string[]> = {
  horizontal: ["ArrowLeft"],
  vertical: ["ArrowUp"],
  both: ["ArrowLeft", "ArrowUp"],
};

/** The index focus moves to after `key`, skipping disabled entries and wrapping at the ends. Undefined when the key does not navigate. */
export function nextFocusIndex(
  key: string,
  currentIndex: number,
  disabledStates: boolean[],
  orientation: Orientation,
): number | undefined {
  if (key === "Home") {
    return firstEnabledIndex(disabledStates, 1, -1);
  }
  if (key === "End") {
    return firstEnabledIndex(disabledStates, -1, disabledStates.length);
  }
  if (forwardKeys[orientation].includes(key)) {
    return firstEnabledIndex(disabledStates, 1, currentIndex);
  }
  if (backwardKeys[orientation].includes(key)) {
    return firstEnabledIndex(disabledStates, -1, currentIndex);
  }
  return undefined;
}

function firstEnabledIndex(disabledStates: boolean[], direction: 1 | -1, startIndex: number): number | undefined {
  const count = disabledStates.length;
  for (let step = 1; step <= count; step++) {
    const candidate = (((startIndex + direction * step) % count) + count) % count;
    if (!disabledStates[candidate]) {
      return candidate;
    }
  }
  return undefined;
}
