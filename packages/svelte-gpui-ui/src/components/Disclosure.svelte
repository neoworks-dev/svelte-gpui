<!--
  A title that opens and closes the content under it, such as a question and its answer. Alone it stands on the
  page; inside a DisclosureGroup it takes the group's lines and, when the group is exclusive, closes when
  another one opens.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Icon, Text, View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import caretDown from "@phosphor-icons/core/regular/caret-down.svg";
  import caretUp from "@phosphor-icons/core/regular/caret-up.svg";
  import { currentTheme } from "../theme.svelte";
  import {
    disclosureChevronSize,
    disclosureContentLook,
    disclosureFocusRingLook,
    disclosureHeaderLook,
    disclosureIconCircleLook,
    disclosureIconSize,
    disclosureLook,
    disclosureSupportingTextLook,
    disclosureTitleLook,
  } from "./disclosure-look";
  import { staysOpen, useDisclosureGroup } from "./disclosureGroupContext";
  import { createFocusSource } from "./focusSource";
  import { isActivationKey } from "./navigation";

  interface Props {
    /** The line that is always visible and opens the section, such as a question. */
    title: string;
    /** A short line under the title that says what is inside. */
    supportingText?: string;
    /** Whether the content is showing. */
    open?: boolean;
    /** SVG markup of a leading icon in a soft circle. */
    icon?: string;
    /** The content that opens and closes. Put the answer or the details here. */
    children: Snippet;
  }

  let { title, supportingText, open = $bindable(false), icon, children }: Props = $props();

  const theme = $derived(currentTheme());
  const group = useDisclosureGroup();
  const id = $props.id();
  const header = $derived(disclosureHeaderLook(theme));
  const caret = $derived.by(() => {
    if (open) {
      return caretUp;
    }
    return caretDown;
  });
  let focused = $state(false);
  const focusSource = createFocusSource();

  function toggle(): void {
    open = !open;
  }

  function toggleOnActivationKey(event: KeyEvent): void {
    if (!isActivationKey(event.key)) {
      return;
    }
    event.preventDefault();
    toggle();
  }

  function handleFocus(): void {
    focused = focusSource.focused();
  }

  function handleBlur(): void {
    focused = false;
    focusSource.blurred();
  }

  $effect(() => {
    if (open) {
      group?.opened(id);
    }
  });

  $effect(() => {
    if (open && group !== undefined && !staysOpen(group.exclusive, group.lastOpenedId, id)) {
      open = false;
    }
  });
</script>

<View style={disclosureLook(theme, group !== undefined)}>
  <View
    style={header.style}
    hover={header.hover}
    active={header.active}
    focusable
    onpress={toggle}
    ondown={focusSource.pointerDown}
    onkey={toggleOnActivationKey}
    onfocus={handleFocus}
    onblur={handleBlur}
  >
    {#if icon}
      <View style={disclosureIconCircleLook(theme)}>
        <Icon svg={icon} size={disclosureIconSize} />
      </View>
    {/if}
    <View style={{ flex: 1, minWidth: 0 }}>
      <Text style={disclosureTitleLook()}>{title}</Text>
      {#if supportingText}
        <Text style={disclosureSupportingTextLook(theme)}>{supportingText}</Text>
      {/if}
    </View>
    <Icon svg={caret} size={disclosureChevronSize} color={theme.colors.muted} style={{ shrink: 0 }} />
    {#if focused}
      <View style={disclosureFocusRingLook(theme)} />
    {/if}
  </View>
  {#if open}
    <View style={disclosureContentLook(theme)}>
      {@render children()}
    </View>
  {/if}
</View>
