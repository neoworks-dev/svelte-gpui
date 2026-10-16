// Whether the demo has something the user would lose by closing the window, and the
// question it asks before letting them. Shared state is a module exporting `$state`.

export const unsaved = $state({ dirty: false });

/** Set while the window waits for the user's answer to "close without saving?". */
export const closing = $state({ asking: false });

let answer: ((discard: boolean) => void) | undefined;

/** Asks the user, and resolves to whether the window may close. */
export function askToDiscard(): Promise<boolean> {
  closing.asking = true;
  return new Promise((resolve) => {
    answer = resolve;
  });
}

/** The user's answer from the dialog: `true` discards the note and closes. */
export function answerDiscard(discard: boolean): void {
  closing.asking = false;
  answer?.(discard);
  answer = undefined;
}
