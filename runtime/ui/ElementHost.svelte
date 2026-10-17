<!--
  The node an app element is drawn in. `element(name)` (element.ts) wraps this in a component of its own, so
  an app never uses it directly. Props reach the Rust view as JSON, one attribute each, and the props that
  are functions and start with `on` become listeners for the events the view emits.
-->
<script lang="ts">
  import { callMethod } from "../host/bridge";
  import { expect } from "../io/channel";
  import { encode, type Style } from "./style";
  import { itself, listener, type KeyEvent, type Layout } from "./events";

  interface Props {
    /** The name the element is registered under in Rust. */
    __element: string;
    /** Keeps the Rust view alive across a hot reload: the new run gets the view with the same name and key. */
    key?: string;
    style?: Style;
    onkey?: (event: KeyEvent) => void;
    onfocus?: () => void;
    onblur?: () => void;
    onlayout?: (layout: Layout) => void;
    onresize?: (layout: Layout) => void;
    [prop: string]: unknown;
  }

  let { __element: name, key, style, onkey, onfocus, onblur, onlayout, onresize, ...rest }: Props = $props();

  let node: any = $state();

  /** Events the view emits are listened for under `app:` and their name, so none of the window's own collide. */
  const attributes = $derived.by(() => {
    const found: Record<string, unknown> = {
      name,
      key,
      look: encode(style),
      onkey: listener(onkey, itself<KeyEvent>),
      onfocus: listener(onfocus, () => undefined),
      onblur: listener(onblur, () => undefined),
      onlayout: listener(onlayout, itself<Layout>),
      onresize: listener(onresize, itself<Layout>),
    };
    for (const [prop, value] of Object.entries(rest)) {
      if (prop.startsWith("on") && typeof value === "function") {
        found[`onapp:${prop.slice(2)}`] = (event: { detail: unknown }) => value(event.detail);
      }
    }
    return found;
  });

  /** The props that are on the node now, so one the app stops passing can be taken off. */
  let sent = new Set<string>();

  $effect(() => {
    if (node === undefined) {
      return;
    }
    const now = new Set<string>();
    for (const [prop, value] of Object.entries(rest)) {
      const json = typeof value === "function" ? undefined : JSON.stringify(value);
      if (json === undefined) {
        continue;
      }
      node.setAttribute(`prop:${prop}`, json);
      now.add(prop);
    }
    for (const prop of sent) {
      if (!now.has(prop)) {
        node.removeAttribute(`prop:${prop}`);
      }
    }
    sent = now;
  });

  /** Gives the Rust view keyboard focus, when it takes any. */
  export function focus(): void {
    node?.focus();
  }

  /** Gives focus up if the view has it. */
  export function blur(): void {
    node?.blur();
  }

  /** Calls a method of the Rust view. Resolves with its answer, and rejects with its error. */
  export function __call(method: string, args: unknown[]): Promise<unknown> {
    if (node === undefined) {
      return Promise.reject(new Error(`cannot call ${method}: the element is not mounted`));
    }
    const { call, reply } = expect();
    callMethod(node, call, method, args);
    return reply.then((answer) => answer.json);
  }
</script>

<g-element bind:this={node} {...attributes}></g-element>
