<!--
  A text field. The host owns the text while it is being edited, so typing, input
  methods, selection and the clipboard work without the app; each change arrives
  through `oninput` and `bind:value`, and setting `value` replaces the text.
-->
<script lang="ts">
  import { encode, type Style } from "./style";
  import { itself, listener, type KeyEvent } from "./events";

  interface Props {
    value?: string;
    placeholder?: string;
    /** Several lines; Enter breaks the line instead of submitting. */
    multiline?: boolean;
    /** How many lines a multiline field shows at least. */
    rows?: number;
    /** Shows dots instead of the text. */
    secure?: boolean;
    /** Takes only digits, signs, a point and exponents. */
    numeric?: boolean;
    maxLength?: number;
    disabled?: boolean;
    readonly?: boolean;
    style?: Style;
    hover?: Style;
    focus?: Style;
    /** Every change to the text. */
    oninput?: (value: string) => void;
    /** Enter in a single-line field. */
    onsubmit?: (value: string) => void;
    /** The text changed since the field took focus, and focus left or Enter was pressed. */
    oncommit?: (value: string) => void;
    /** The cursor or the selection moved, with the text offsets (UTF-16, as for `select()`) of its two ends. */
    onselect?: (selection: { start: number; end: number }) => void;
    onkey?: (event: KeyEvent) => void;
    onfocus?: () => void;
    onblur?: () => void;
  }

  let { value = $bindable(""), ...props }: Props = $props();

  const attributes = $derived({
    look: encode(props.style),
    hover: encode(props.hover),
    focus: encode(props.focus),
    placeholder: props.placeholder,
    multiline: props.multiline ? "" : undefined,
    rows: props.rows,
    secure: props.secure ? "" : undefined,
    numeric: props.numeric ? "" : undefined,
    maxlength: props.maxLength,
    disabled: props.disabled ? "" : undefined,
    readonly: props.readonly ? "" : undefined,
    onsubmit: listener(props.onsubmit, (event: { value: string }) => event.value),
    oncommit: listener(props.oncommit, (event: { value: string }) => event.value),
    onselect: listener(props.onselect, (event: { start: number; end: number }) => event),
    onkey: listener(props.onkey, itself<KeyEvent>),
    onfocus: listener(props.onfocus, () => undefined),
    onblur: listener(props.onblur, () => undefined),
  });

  function edit(event: { value: string }): void {
    value = event.value;
    props.oninput?.(event.value);
  }

  let element: any;

  export function focus(): void {
    element?.focus();
  }

  /** Gives focus up if it has it: back to what had it before, when that is still there. */
  export function blur(): void {
    element?.blur();
  }

  /**
   * Where the cursor is, right now: the selection's two ends in UTF-16 offsets, as for `select()`,
   * which are equal when nothing is selected, and whether the cursor is on the first and on the last
   * row of the text as drawn. A line that wraps counts as several rows. Null until the field is drawn.
   */
  export function getSelection(): { start: number; end: number; firstRow: boolean; lastRow: boolean } | null {
    return element?.fieldState() ?? null;
  }

  /** Selects part of the text, in UTF-16 offsets; all of it by default. */
  export function select(start = 0, end = value.length): void {
    element?.setSelectionRange(start, end);
  }
</script>

<g-input bind:this={element} {...attributes} value={value} onedit={edit}></g-input>
