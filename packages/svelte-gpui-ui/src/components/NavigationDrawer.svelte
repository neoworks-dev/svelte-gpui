<!--
  A column of destinations grouped in sections. Docked, it is a permanent column on the page canvas with small rows
  suited to a pointer. With `modal` it is a surface that slides over the page, with a scrim behind it; a press on the
  scrim or Escape closes it by setting `open` to false.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { Presence, Scroll, Text, View, type KeyEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import NavigationDrawerItem from "./NavigationDrawerItem.svelte";
  import { isCurrentItem, type NavSection } from "./navigation";
  import {
    drawerHeadlineLook,
    drawerItemGap,
    drawerLook,
    drawerScrimBackground,
    drawerSectionHeadingLook,
    drawerWidth,
  } from "./navigation-look";

  interface Props {
    sections: NavSection[];
    /** The `value` of the active destination. Sub-pages of an item keep it active. */
    current: string;
    /** A modal drawer slides over the page. A docked one is a permanent column. */
    modal?: boolean;
    /** Whether a modal drawer is showing. */
    open?: boolean;
    /** Name of the product or account at the top of the drawer. */
    title?: string;
    header?: Snippet;
    footer?: Snippet;
    onselect?: (value: string) => void;
    /** Layout only. */
    style?: Style;
  }

  let { sections, current, modal = false, open = $bindable(true), title, header, footer, onselect, style }: Props = $props();

  const theme = $derived(currentTheme());
  const docked = $derived(!modal);
  let panel: ReturnType<typeof View> | undefined = $state();

  $effect(() => {
    if (modal && open) {
      panel?.focus();
    }
  });

  function close(): void {
    open = false;
  }

  function handleKey(event: KeyEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  }

  function select(value: string): void {
    onselect?.(value);
  }
</script>

{#snippet drawer()}
  <View bind:this={panel} focusable={modal} style={{ ...drawerLook(theme, docked), ...style }} onkey={handleKey}>
    {#if title}
      <Text style={drawerHeadlineLook(theme)}>{title}</Text>
    {/if}
    {#if header}
      <View style={{ paddingX: 4 }}>{@render header()}</View>
    {/if}
    <Scroll style={{ flex: 1, minHeight: 0 }}>
      <View style={{ gap: 16 }}>
        {#each sections as section, sectionIndex (sectionIndex)}
          <View>
            {#if section.heading}
              <Text style={drawerSectionHeadingLook(theme)}>{section.heading}</Text>
            {/if}
            <View style={{ gap: drawerItemGap(docked) }}>
              {#each section.items as item (item.value)}
                <NavigationDrawerItem {item} active={isCurrentItem(current, item.value)} {docked} onselect={select} />
              {/each}
            </View>
          </View>
        {/each}
      </View>
    </Scroll>
    {#if footer}
      <View style={{ paddingX: 4 }}>{@render footer()}</View>
    {/if}
  </View>
{/snippet}

{#if !modal}
  {@render drawer()}
{:else if open}
  <Presence in={[fade, { duration: 220 }]} out={[fade, { duration: 160 }]} style={{ position: "absolute", inset: 0, direction: "row" }}>
    <View style={{ position: "absolute", inset: 0, background: drawerScrimBackground(theme) }} onpress={close} />
    <Presence in={[fly, { x: -drawerWidth, duration: 380 }]} out={[fly, { x: -drawerWidth, duration: 200 }]} style={{ height: "full" }}>
      {@render drawer()}
    </Presence>
  </Presence>
{/if}
