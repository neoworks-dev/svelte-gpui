<!--
  A text field: an `input`, or a `textarea` when `multiline`. The browser owns the text while it is
  being edited, so typing, input methods, selection and the clipboard work without the app; each
  change arrives through `oninput` and `bind:value`, and setting `value` replaces the text.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { InputProps } from "../../ui/props";
  import { cssText, layered, toCss } from "./css";
  import { interaction } from "./interact.svelte";
  import { clampSelection, cursorRows, numericOnly } from "./input-text";

  let { value = $bindable(""), ...props }: InputProps = $props();

  const ix = interaction(() => props);
  let element: HTMLInputElement | HTMLTextAreaElement | undefined = $state();

  const css = $derived(cssText(toCss(layered(props.style, [ix.focused && props.focus, ix.hovered && props.hover]))));

  // The page's own text goes into the field when the app sets `value`, unless it is what the field has already.
  $effect(() => {
    const text = value ?? "";
    if (element !== undefined && element.value !== text) {
      element.value = text;
    }
  });

  // A multiline field is as tall as its text, and at least `rows` lines, unless the style gives it a height.
  $effect(() => {
    void [value, props.rows, css];
    const node = element;
    if (node instanceof HTMLTextAreaElement && props.style?.height === undefined) {
      node.style.height = "auto";
      node.style.height = `${node.scrollHeight}px`;
    }
  });

  function typed(event: Event): void {
    const node = event.currentTarget as HTMLInputElement | HTMLTextAreaElement;
    if (props.numeric) {
      const clean = numericOnly(node.value, node.selectionStart ?? node.value.length);
      if (clean.text !== node.value) {
        node.value = clean.text;
        node.setSelectionRange(clean.caret, clean.caret);
      }
    }
    value = node.value;
    props.oninput?.(node.value);
  }

  /** What the field does with a key once the app's `onkey` has had it, which is set up after the interaction's. */
  function field(node: HTMLElement): () => void {
    return untrack(() => {
      let atFocus = value ?? "";
      const commit = (): void => {
        const text = (node as HTMLInputElement).value;
        if (text !== atFocus) {
          atFocus = text;
          props.oncommit?.(text);
        }
      };
      const keydown = (event: KeyboardEvent): void => {
        if (event.key === "Enter" && !event.isComposing && !event.defaultPrevented && !props.multiline) {
          props.onsubmit?.((node as HTMLInputElement).value);
          commit();
        }
      };
      const focus = (): void => {
        atFocus = (node as HTMLInputElement).value;
      };
      let reported = "";
      const selection = (): void => {
        if (document.activeElement !== node || props.onselect === undefined) {
          return;
        }
        const { selectionStart: start, selectionEnd: end } = node as HTMLInputElement;
        const key = `${start}:${end}`;
        if (start !== null && end !== null && key !== reported) {
          reported = key;
          props.onselect({ start, end });
        }
      };
      node.addEventListener("keydown", keydown);
      node.addEventListener("focus", focus);
      node.addEventListener("blur", commit);
      document.addEventListener("selectionchange", selection);
      return () => {
        node.removeEventListener("keydown", keydown);
        node.removeEventListener("focus", focus);
        node.removeEventListener("blur", commit);
        document.removeEventListener("selectionchange", selection);
      };
    });
  }

  export function focus(): void {
    ix.focus(element);
  }

  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  export function blur(): void {
    ix.blur(element);
  }

  /** The row each character of a textarea is on, by laying its text out again in a copy of it. */
  function rowTops(node: HTMLTextAreaElement): { top: (offset: number) => number; dispose: () => void } {
    const style = getComputedStyle(node);
    const mirror = document.createElement("div");
    for (const property of [
      "fontFamily", "fontSize", "fontWeight", "fontStyle", "letterSpacing", "wordSpacing", "textTransform",
      "textIndent", "lineHeight", "tabSize", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft",
    ] as const) {
      mirror.style[property] = style[property];
    }
    Object.assign(mirror.style, {
      position: "absolute",
      visibility: "hidden",
      top: "0",
      left: "-9999px",
      boxSizing: "border-box",
      width: `${node.clientWidth}px`,
      whiteSpace: "pre-wrap",
      overflowWrap: "break-word",
    });
    document.body.append(mirror);
    const text = node.value;
    const top = (offset: number): number => {
      mirror.textContent = text.slice(0, offset);
      const marker = document.createElement("span");
      marker.textContent = "​";
      mirror.append(marker);
      return marker.offsetTop;
    };
    return { top, dispose: () => mirror.remove() };
  }

  /**
   * Where the cursor is, right now: the selection's two ends in UTF-16 offsets, as for `select()`, which are equal
   * when nothing is selected, and whether the cursor is on the first and on the last row of the text as drawn. A line
   * that wraps counts as several rows. Null until the field is drawn.
   */
  export function getSelection(): { start: number; end: number; firstRow: boolean; lastRow: boolean } | null {
    const node = element;
    if (node === undefined) {
      return null;
    }
    const start = node.selectionStart ?? 0;
    const end = node.selectionEnd ?? 0;
    if (!(node instanceof HTMLTextAreaElement)) {
      return { start, end, firstRow: true, lastRow: true };
    }
    const rows = rowTops(node);
    try {
      return { start, end, ...cursorRows(rows.top, start, end, node.value.length) };
    } finally {
      rows.dispose();
    }
  }

  /** Selects part of the text, in UTF-16 offsets; all of it by default. */
  export function select(start = 0, end = (value ?? "").length): void {
    const range = clampSelection(start, end, element?.value.length ?? 0);
    element?.setSelectionRange(range.start, range.end);
  }
</script>

{#if props.multiline}
  <textarea
    bind:this={element}
    class="g-input"
    style={css}
    rows={props.rows ?? 3}
    placeholder={props.placeholder}
    disabled={props.disabled}
    readonly={props.readonly}
    maxlength={props.maxLength}
    autocomplete="off"
    spellcheck="false"
    oninput={typed}
    {@attach ix.attach}
    {@attach field}
  ></textarea>
{:else}
  <input
    bind:this={element}
    class="g-input"
    style={css}
    type={props.secure ? "password" : "text"}
    inputmode={props.numeric ? "decimal" : undefined}
    placeholder={props.placeholder}
    disabled={props.disabled}
    readonly={props.readonly}
    maxlength={props.maxLength}
    autocomplete="off"
    spellcheck="false"
    oninput={typed}
    {@attach ix.attach}
    {@attach field}
  />
{/if}
