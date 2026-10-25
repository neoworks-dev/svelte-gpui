<!-- A thin line between things. A vertical one stretches to the height of its row. -->
<script lang="ts">
  import { View, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { dividerLook, type DividerInset, type DividerOrientation } from "./divider-look";

  interface Props {
    orientation?: DividerOrientation;
    /** `start` leaves 64 pixels free so the line begins at the text of a row with a leading icon, `both` indents both ends. */
    inset?: DividerInset;
    /** A darker line for dividing larger areas. */
    strong?: boolean;
    /** Layout only, such as margins. */
    style?: Style;
  }

  let { orientation = "horizontal", inset = "none", strong = false, style }: Props = $props();

  const look = $derived(dividerLook(currentTheme(), { orientation, inset, strong }));
</script>

<View style={{ ...look, ...style }} />
