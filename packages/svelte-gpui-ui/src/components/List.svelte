<!--
  A column of ListItem rows. `card` puts them in an outlined surface, as on a settings page, and `plain`
  leaves them on the page. For a list of thousands of rows, use the virtualised `List` of the UI primitives.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { provideList } from "./listContext";
  import { listLook, listRowsLook, type ListVariant } from "./list-look";

  interface Props {
    variant?: ListVariant;
    /** Draws a hairline between rows. */
    dividers?: boolean;
    /** Layout only, such as width or margins. */
    style?: Style;
    /** The ListItem rows. */
    children: Snippet;
  }

  let { variant = "card", dividers = false, style, children }: Props = $props();

  const theme = $derived(currentTheme());

  provideList({
    get dividers() {
      return dividers;
    },
  });
</script>

<View style={{ ...listLook(theme, variant, dividers), ...style }}>
  <View style={listRowsLook(dividers)}>
    {@render children()}
  </View>
</View>
