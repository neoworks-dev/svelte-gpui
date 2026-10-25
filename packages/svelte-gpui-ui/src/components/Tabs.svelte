<!--
  A row of tabs that switch between views. `primary` tabs switch the main sections of a screen, with the icon above the
  label; `secondary` tabs divide the content inside a section. The arrow keys, Home and End move between tabs and choose them.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import Tab from "./Tab.svelte";
  import { tabListLook, type TabsVariant } from "./tabs-look";
  import { tabKeyAction } from "./tabsLogic";
  import type { TabItem } from "./types";

  interface Props {
    tabs: TabItem[];
    /** The id of the active tab. */
    value?: string;
    variant?: TabsVariant;
    /** Spreads the tabs across the full width instead of sizing them to their text. */
    fullWidth?: boolean;
    /** Content of the active tab, drawn under the list. */
    panel?: Snippet<[string]>;
    onchange?: (id: string) => void;
  }

  let { tabs, value = $bindable(""), variant = "primary", fullWidth = false, panel, onchange }: Props = $props();

  let tabElements: ReturnType<typeof Tab>[] = $state([]);

  const options = $derived({ variant, fullWidth });

  function select(tab: TabItem): void {
    value = tab.id;
    onchange?.(tab.id);
  }

  function handleKey(event: KeyEvent, index: number): void {
    const action = tabKeyAction(event.key, tabs, index);
    if (action.kind === "none") {
      return;
    }
    event.preventDefault();
    if (action.kind === "select") {
      select(tabs[index]);
      return;
    }
    tabElements[action.index]?.focus();
    select(tabs[action.index]);
  }
</script>

<View>
  <View style={tabListLook(currentTheme(), fullWidth)}>
    {#each tabs as tab, index (tab.id)}
      <Tab
        bind:this={tabElements[index]}
        {tab}
        active={value === tab.id}
        {options}
        onselect={() => select(tab)}
        onkey={(event) => handleKey(event, index)}
      />
    {/each}
  </View>
  {#if panel}
    {@render panel(value)}
  {/if}
</View>
