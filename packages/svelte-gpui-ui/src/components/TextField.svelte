<!--
  A text field with a label inside it. The label moves up and shrinks once the field has focus or
  text. `variant="filled"` suits a page, `"outlined"` a surface that is already filled.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Input, Text, View, type Shadow, type Style } from "@neoworks/svelte-gpui/ui";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import { ring } from "./ring";

  type Variant = "outlined" | "filled";
  type FieldType = "text" | "password" | "number" | "email" | "search" | "tel" | "url";

  interface Props {
    value?: string;
    /** Names the field. It sits inside the field and moves up once there is something to read. */
    label: string;
    variant?: Variant;
    /** `password` hides the text and `number` takes only digits; the rest are plain text. */
    type?: FieldType;
    /** Example input, shown while the field is focused and empty. */
    placeholder?: string;
    /** Help under the field. Shown in the danger colour while `error` is on, so write it as the fix. */
    supportingText?: string;
    error?: boolean;
    /** SVG markup of an icon at the start of the field. */
    icon?: string;
    /** SVG markup of a decorative icon at the end of the field, such as a warning. */
    trailingIcon?: string;
    /** Interactive content at the end, such as an icon button that clears the field. */
    trailing?: Snippet;
    /** Fixed text before the value, such as a currency sign. */
    prefix?: string;
    /** Fixed text after the value, such as a unit or a domain. */
    suffix?: string;
    /** Several lines, for longer text. */
    multiline?: boolean;
    rows?: number;
    /** Shows a counter under the field and stops input at this length. */
    maxlength?: number;
    disabled?: boolean;
    readonly?: boolean;
    /** Layout of the whole field, such as its width. */
    style?: Style;
    oninput?: (value: string) => void;
    onsubmit?: (value: string) => void;
  }

  let {
    value = $bindable(""),
    label,
    variant = "filled",
    type = "text",
    placeholder,
    supportingText,
    error = false,
    icon,
    trailingIcon,
    trailing,
    prefix,
    suffix,
    multiline = false,
    rows = 3,
    maxlength,
    disabled = false,
    readonly = false,
    style,
    oninput,
    onsubmit,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const fieldHeight = 52;
  let focused = $state(false);

  const floated = $derived(focused || value.length > 0);
  const showsPrefix = $derived(prefix !== undefined && floated);
  const showsSuffix = $derived(suffix !== undefined && floated);
  const hasSupportingRow = $derived(supportingText !== undefined || maxlength !== undefined);

  function containerRadius(): number {
    if (multiline) {
      return theme.radii.card;
    }
    return 999;
  }

  function restingBackground(): string {
    if (variant === "filled") {
      return theme.colors.sunken;
    }
    return "transparent";
  }

  function restingBorder(): string {
    if (variant === "filled") {
      return "transparent";
    }
    return theme.colors.line;
  }

  function containerBorder(): string {
    if (focused || error) {
      return "transparent";
    }
    return restingBorder();
  }

  function fieldRing(): Shadow | null {
    if (error) {
      return ring(theme.colors.danger, 2);
    }
    if (focused) {
      return ring(theme.colors.text, 2);
    }
    return null;
  }

  function containerHover(): Style {
    if (disabled || focused || error) {
      return {};
    }
    if (variant === "filled") {
      return { background: theme.colors.pressed };
    }
    return { borderColor: theme.colors.lineStrong };
  }

  function containerOpacity(): number {
    if (disabled) {
      return 0.4;
    }
    return 1;
  }

  function labelColor(): string {
    if (error) {
      return theme.colors.danger;
    }
    if (focused) {
      return theme.colors.text;
    }
    return theme.colors.muted;
  }

  function labelStyle(): Style {
    if (floated) {
      return { ...typeStyle("caption"), position: "absolute", left: 20, top: 6, color: labelColor() };
    }
    return { ...typeStyle("body"), position: "absolute", left: 20, top: 14, color: labelColor() };
  }

  function inputPaddingLeft(): number {
    if (showsPrefix) {
      return 4;
    }
    return 20;
  }

  function inputPaddingRight(): number {
    if (showsSuffix) {
      return 4;
    }
    return 20;
  }

  function shownPlaceholder(): string | undefined {
    if (focused) {
      return placeholder;
    }
    return undefined;
  }

  function trailingIconColor(): string {
    if (error) {
      return theme.colors.danger;
    }
    return theme.colors.muted;
  }

  function supportingColor(): string {
    if (error) {
      return theme.colors.danger;
    }
    return theme.colors.muted;
  }

  function handleInput(text: string): void {
    oninput?.(text);
  }
</script>

<View style={{ width: "full", ...style }}>
  <View
    style={{
      direction: "row",
      align: "start",
      width: "full",
      minHeight: fieldHeight,
      radius: containerRadius(),
      borderWidth: 1,
      borderColor: containerBorder(),
      background: restingBackground(),
      shadow: fieldRing(),
      opacity: containerOpacity(),
      transition: 200,
    }}
    hover={containerHover()}
  >
    {#if icon}
      <View style={{ height: fieldHeight, paddingLeft: 20, align: "center", justify: "center" }}>
        <Icon svg={icon} size={20} color={theme.colors.muted} />
      </View>
    {/if}

    <View style={{ direction: "row", align: "end", flex: 1, minWidth: 0 }}>
      <Text style={labelStyle()}>{label}</Text>
      {#if showsPrefix}
        <Text style={{ ...typeStyle("body"), paddingBottom: 6, paddingLeft: 20, color: theme.colors.muted }}>{prefix}</Text>
      {/if}
      <Input
        bind:value
        placeholder={shownPlaceholder()}
        {multiline}
        {rows}
        secure={type === "password"}
        numeric={type === "number"}
        maxLength={maxlength}
        {disabled}
        {readonly}
        style={{
          ...typeStyle("body"),
          flex: 1,
          minWidth: 0,
          paddingTop: 22,
          paddingBottom: 6,
          paddingLeft: inputPaddingLeft(),
          paddingRight: inputPaddingRight(),
          background: "transparent",
          color: theme.colors.text,
        }}
        oninput={handleInput}
        {onsubmit}
        onfocus={() => (focused = true)}
        onblur={() => (focused = false)}
      />
      {#if showsSuffix}
        <Text style={{ ...typeStyle("body"), paddingBottom: 6, paddingRight: 20, color: theme.colors.muted }}>{suffix}</Text>
      {/if}
    </View>

    {#if trailing}
      <View style={{ height: fieldHeight, paddingRight: 8, align: "center", justify: "center" }}>
        {@render trailing()}
      </View>
    {:else if trailingIcon}
      <View style={{ height: fieldHeight, paddingRight: 20, align: "center", justify: "center" }}>
        <Icon svg={trailingIcon} size={20} color={trailingIconColor()} />
      </View>
    {/if}
  </View>

  {#if hasSupportingRow}
    <View style={{ direction: "row", justify: "between", gap: 16, marginTop: 6, paddingX: 20 }}>
      <Text style={{ ...typeStyle("caption"), color: supportingColor() }}>{supportingText}</Text>
      {#if maxlength !== undefined}
        <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>{value.length}/{maxlength}</Text>
      {/if}
    </View>
  {/if}
</View>
