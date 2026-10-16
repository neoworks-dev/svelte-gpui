// The little of `Event` and `EventTarget` that `AbortSignal` and `WebSocket` need. The
// DOM's own `Event` is only defined once the window is installed, after the web
// globals, so these stand alone.

/** An event: what happened, and the object it happened to. */
export class NetEvent {
  target: unknown = null;
  currentTarget: unknown = null;
  defaultPrevented = false;
  readonly timeStamp = Date.now();

  constructor(
    readonly type: string,
    readonly cancelable = false,
  ) {}

  preventDefault(): void {
    if (this.cancelable) {
      this.defaultPrevented = true;
    }
  }

  stopPropagation(): void {}

  stopImmediatePropagation(): void {}
}

type Listener = ((event: any) => void) | { handleEvent(event: any): void };

interface Registration {
  listener: Listener;
  once: boolean;
}

/** Holds listeners by event type, and calls them in the order they were added. */
export class NetEventTarget {
  private readonly registrations = new Map<string, Registration[]>();

  addEventListener(type: string, listener: Listener | null, options?: boolean | { once?: boolean }): void {
    if (listener === null || listener === undefined) {
      return;
    }
    const registrations = this.registrations.get(type) ?? [];
    if (registrations.some((registration) => registration.listener === listener)) {
      return;
    }
    registrations.push({ listener, once: typeof options === "object" && options?.once === true });
    this.registrations.set(type, registrations);
  }

  removeEventListener(type: string, listener: Listener | null): void {
    const registrations = this.registrations.get(type);
    if (registrations === undefined) {
      return;
    }
    const index = registrations.findIndex((registration) => registration.listener === listener);
    if (index !== -1) {
      registrations.splice(index, 1);
    }
  }

  /** Calls the listeners for the event's type. A listener that throws is reported, not rethrown. */
  dispatchEvent(event: NetEvent): boolean {
    event.target = this;
    event.currentTarget = this;
    for (const registration of [...(this.registrations.get(event.type) ?? [])]) {
      if (registration.once) {
        this.removeEventListener(event.type, registration.listener);
      }
      try {
        const { listener } = registration;
        if (typeof listener === "function") {
          listener.call(this, event);
        } else {
          listener.handleEvent(event);
        }
      } catch (error) {
        (globalThis as any).console?.error("uncaught in event listener:", error);
      }
    }
    event.currentTarget = null;
    return !event.defaultPrevented;
  }
}
