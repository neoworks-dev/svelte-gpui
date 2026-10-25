<!--
  A stack of Disclosure rows. `plain` draws lines above, between and below them and leaves them on the page,
  `card` puts them in an outlined surface, as on a settings page. With `exclusive`, opening one closes the
  one that was open.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { disclosureGroupLook, disclosureGroupRowsLook, type DisclosureGroupVariant } from "./disclosure-look";
  import { provideDisclosureGroup } from "./disclosureGroupContext";

  interface Props {
    variant?: DisclosureGroupVariant;
    exclusive?: boolean;
    /** Layout only, such as width or margins. */
    style?: Style;
    /** The Disclosure rows. */
    children: Snippet;
  }

  let { variant = "plain", exclusive = false, style, children }: Props = $props();

  const theme = $derived(currentTheme());
  let lastOpenedId: string | undefined = $state();

  provideDisclosureGroup({
    get exclusive() {
      return exclusive;
    },
    get lastOpenedId() {
      return lastOpenedId;
    },
    opened(id) {
      lastOpenedId = id;
    },
  });
</script>

<View style={{ ...disclosureGroupLook(theme, variant), ...style }}>
  <View style={disclosureGroupRowsLook()}>
    {@render children()}
  </View>
</View>
