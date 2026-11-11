// The handlers `window.onCloseRequested` registers, and how they decide whether the window closes. The
// native window and the browser's share this, and differ in what closing is.

declare const console: { error(...args: unknown[]): void };

/**
 * Decides whether the window may close. Return `false`, or a promise that resolves to
 * `false`, to keep it open; anything else lets it close.
 */
export type CloseHandler = () => boolean | void | Promise<boolean | void>;

function isThenable(value: unknown): value is PromiseLike<unknown> {
  return typeof (value as { then?: unknown } | null | undefined)?.then === "function";
}

export class CloseHandlers {
  private readonly handlers = new Set<CloseHandler>();
  private asking = false;

  /** Adds a handler. Returns a function that removes it. */
  add(handler: CloseHandler): () => void {
    this.handlers.add(handler);
    return () => {
      this.handlers.delete(handler);
    };
  }

  /**
   * Asks the handlers. Answers whether the window may close now. When a handler answers later, this answers
   * `false` and calls `close` once every handler has said yes.
   */
  ask(close: () => void): boolean {
    if (this.asking) {
      return false;
    }
    const later: PromiseLike<unknown>[] = [];
    for (const handler of [...this.handlers]) {
      let answer;
      try {
        answer = handler();
      } catch (error) {
        console.error("a close handler threw:", error);
        continue;
      }
      if (answer === false) {
        return false;
      }
      if (isThenable(answer)) {
        later.push(answer);
      }
    }
    if (later.length === 0) {
      return true;
    }
    this.asking = true;
    Promise.all(later).then(
      (answers) => {
        this.asking = false;
        if (answers.every((answer) => answer !== false)) {
          close();
        }
      },
      (error) => {
        this.asking = false;
        console.error("a close handler failed:", error);
      },
    );
    return false;
  }
}
