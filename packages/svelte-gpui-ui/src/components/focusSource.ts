/**
 * Tells keyboard focus from focus a press gave, so the focus ring only shows for the keyboard, as `:focus-visible` does.
 * The host reports the press's focus after the pointer went down, so a press arms it and the next focus consumes it.
 */
export interface FocusSource {
  pointerDown(): void;
  /** True when this focus came from the keyboard. */
  focused(): boolean;
  blurred(): void;
}

export function createFocusSource(): FocusSource {
  let pressArmed = false;
  return {
    pointerDown() {
      pressArmed = true;
    },
    focused() {
      const fromPress = pressArmed;
      pressArmed = false;
      return !fromPress;
    },
    blurred() {
      pressArmed = false;
    },
  };
}
