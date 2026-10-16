<!--
  A dropdown: a button showing the chosen option, and a list of options under it. The list opens
  on press, or with Down, Enter or Space while the button has focus, and has the same keys as a
  `Menu`. It is as wide as the button and scrolls when it has more than `maxHeight` of options.

    <Select bind:value={size} options={["Small", "Medium", "Large"]} />
    <Select bind:value={id} options={[{ value: 1, label: "One" }, { value: 2, label: "Two", disabled: true }]} />
-->
<script lang="ts" module>
  import { panel } from "./overlay-look";
  import type { Style } from "./style";

  const panelStyleDefault: Style = { ...panel, padding: 0 };
</script>

<script lang="ts" generics="T">
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import Menu from "./Menu.svelte";
  import Popover from "./Popover.svelte";
  import Text from "./Text.svelte";
  import { caretUpDown } from "./icons";
  import { colors } from "./overlay-look";
  import type { MenuItem } from "./menu";
  import type { KeyEvent } from "./events";

  interface Option {
    value: T;
    label: string;
    disabled?: boolean;
  }

  interface Props {
    /** The chosen option's value. */
    value?: T;
    /** Plain values, or `{ value, label, disabled }`. */
    options: (T | Option)[];
    placeholder?: string;
    disabled?: boolean;
    /** The tallest the list gets, in pixels, before it scrolls. The default is 260. */
    maxHeight?: number;
    onchange?: (value: T) => void;
    /** The button's style. */
    style?: Style;
    hover?: Style;
    focus?: Style;
    /** The list's panel style. */
    panelStyle?: Style;
  }

  let {
    value = $bindable(),
    options,
    placeholder = "Select",
    disabled = false,
    maxHeight = 260,
    onchange,
    style,
    hover,
    focus,
    panelStyle,
  }: Props = $props();

  const isOption = (candidate: T | Option): candidate is Option =>
    typeof candidate === "object" && candidate !== null && "value" in candidate && "label" in candidate;

  const normalized: Option[] = $derived(
    options.map((option) => (isOption(option) ? option : { value: option as T, label: String(option) })),
  );
  const chosen = $derived(normalized.findIndex((option) => option.value === value));
  const items: MenuItem[] = $derived(
    normalized.map((option, index) => ({
      label: option.label,
      disabled: option.disabled,
      checked: index === chosen,
      value: option.value,
    })),
  );

  let open = $state(false);

  function choose(item: MenuItem): void {
    value = item.value as T;
    onchange?.(value);
  }

  function key(event: KeyEvent, show: () => void): void {
    if (!open && event.key === "ArrowDown") {
      event.preventDefault();
      show();
    }
  }
</script>

<Popover bind:open side="bottom" align="start" offset={4} matchWidth focusPanel={false} style={{ ...panelStyleDefault, ...panelStyle }}>
  {#snippet trigger({ toggle, show })}
    <Button
      {disabled}
      style={{
        direction: "row",
        align: "center",
        gap: 8,
        paddingX: 10,
        paddingY: 6,
        radius: 6,
        minWidth: 140,
        background: colors.panel,
        borderWidth: 1,
        borderColor: open ? colors.accent : colors.border,
        color: colors.text,
        transition: 120,
        cursor: "default",
        ...style,
      }}
      hover={{ borderColor: colors.accent, ...hover }}
      focus={{ borderColor: colors.accent, ...focus }}
      onpress={toggle}
      onkey={(event) => key(event, show)}
    >
      <Text style={{ flex: 1, minWidth: 0, truncate: true, color: chosen < 0 ? colors.dim : undefined }}>
        {chosen < 0 ? placeholder : normalized[chosen].label}
      </Text>
      <Icon svg={caretUpDown} size={14} color={colors.dim} />
    </Button>
  {/snippet}
  {#snippet children({ close })}
    <Menu {items} style={{ minWidth: 0 }} initial={chosen >= 0 ? chosen : undefined} {maxHeight} onselect={choose} onclose={() => close("select")} />
  {/snippet}
</Popover>
