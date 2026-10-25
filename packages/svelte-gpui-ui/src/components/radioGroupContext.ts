import { getContext, setContext } from "svelte";
import { nextFocusIndex } from "./rovingFocus";

export interface RadioRegistration {
  value: string;
  isDisabled: () => boolean;
  focus: () => void;
}

export interface RadioGroupContext {
  readonly value: string | undefined;
  readonly disabled: boolean;
  readonly error: boolean;
  /** Options sit side by side, each at the width of its content. */
  readonly inline: boolean;
  select: (value: string) => void;
  register: (registration: RadioRegistration) => () => void;
  /** Moves focus and the choice to the neighbour that `key` points to. Returns whether the key navigated. */
  navigate: (fromValue: string, key: string) => boolean;
}

const contextKey = Symbol("neoworks-radio-group");

export function provideRadioGroup(context: RadioGroupContext): void {
  setContext(contextKey, context);
}

export function useRadioGroup(): RadioGroupContext | undefined {
  return getContext<RadioGroupContext | undefined>(contextKey);
}

/** The value of the radio that `key` leads to from `fromValue`, among the registered radios in order. */
export function neighbourValue(
  registrations: Pick<RadioRegistration, "value" | "isDisabled">[],
  fromValue: string,
  key: string,
): string | undefined {
  const currentIndex = registrations.findIndex((registration) => registration.value === fromValue);
  const disabledStates = registrations.map((registration) => registration.isDisabled());
  const targetIndex = nextFocusIndex(key, currentIndex, disabledStates, "both");
  if (targetIndex === undefined) {
    return undefined;
  }
  return registrations[targetIndex].value;
}
