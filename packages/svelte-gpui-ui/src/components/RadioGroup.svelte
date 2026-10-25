<!--
  Names a question and holds the Radio options that answer it. `value` is the chosen option's value.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import {
    neighbourValue,
    provideRadioGroup,
    type RadioRegistration,
  } from "./radioGroupContext";

  type Orientation = "vertical" | "horizontal";

  interface Props {
    /** The value of the chosen Radio, or undefined while nothing is chosen. */
    value?: string;
    /** Names the question the options answer. */
    label: string;
    /** Help text, or the reason for the error when `error` is on. */
    supportingText?: string;
    error?: boolean;
    /** Disables every option in the group. */
    disabled?: boolean;
    orientation?: Orientation;
    /** Layout of the whole group, such as its width. */
    style?: Style;
    onchange?: (value: string) => void;
    children: Snippet;
  }

  let {
    value = $bindable(),
    label,
    supportingText,
    error = false,
    disabled = false,
    orientation = "vertical",
    style,
    onchange,
    children,
  }: Props = $props();

  const theme = $derived(currentTheme());
  const registrations: RadioRegistration[] = [];

  function listStyle() {
    if (orientation === "horizontal") {
      return { direction: "row" as const, wrap: true, columnGap: 16, marginTop: 4 };
    }
    return { direction: "column" as const, marginTop: 4 };
  }

  function supportingColor(): string {
    if (error) {
      return theme.colors.danger;
    }
    return theme.colors.muted;
  }

  function select(selectedValue: string): void {
    value = selectedValue;
    onchange?.(selectedValue);
  }

  function register(registration: RadioRegistration): () => void {
    registrations.push(registration);
    return () => {
      registrations.splice(registrations.indexOf(registration), 1);
    };
  }

  function navigate(fromValue: string, key: string): boolean {
    const target = neighbourValue(registrations, fromValue, key);
    if (target === undefined) {
      return false;
    }
    registrations.find((registration) => registration.value === target)?.focus();
    select(target);
    return true;
  }

  provideRadioGroup({
    get value() {
      return value;
    },
    get disabled() {
      return disabled;
    },
    get error() {
      return error;
    },
    get inline() {
      return orientation === "horizontal";
    },
    select,
    register,
    navigate,
  });
</script>

<View style={{ minWidth: 0, ...style }}>
  <Text style={{ ...typeStyle("body"), fontWeight: 500, color: theme.colors.text }}>{label}</Text>
  <View style={listStyle()}>
    {@render children()}
  </View>
  {#if supportingText}
    <Text style={{ ...typeStyle("bodySmall"), marginTop: 4, color: supportingColor() }}>{supportingText}</Text>
  {/if}
</View>
