<!-- The heading of a section of a page, with a line saying what it holds and an optional action such as "See all". -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Text, View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import {
    sectionHeaderActionLook,
    sectionHeaderDescriptionLook,
    sectionHeaderLook,
    sectionHeaderTitleLook,
  } from "./section-header-look";

  interface Props {
    title: string;
    /** One line under the title that says what the section holds. */
    description?: string;
    /** A link or button at the end. */
    action?: Snippet;
    /** Layout only, such as margins. */
    style?: Style;
  }

  let { title, description, action, style }: Props = $props();

  const theme = $derived(currentTheme());
</script>

<View style={{ ...sectionHeaderLook(), ...style }}>
  <View style={{ flex: 1, minWidth: 0 }}>
    <Text style={sectionHeaderTitleLook(theme)}>{title}</Text>
    {#if description}
      <Text style={sectionHeaderDescriptionLook(theme)}>{description}</Text>
    {/if}
  </View>
  {#if action}
    <View style={sectionHeaderActionLook(theme)}>{@render action()}</View>
  {/if}
</View>
