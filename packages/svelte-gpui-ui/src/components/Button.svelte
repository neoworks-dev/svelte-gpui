<!--
  A pill button. `primary` is the one main action on a screen, `accent` is for confirming something good.
  Its content is the children, drawn in the variant's text colour.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button as ButtonPrimitive, Icon, type PointerEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { buttonIconSizes, buttonLook, type ButtonSize, type ButtonVariant } from "./button-look";

  interface Props {
    variant?: ButtonVariant;
    size?: ButtonSize;
    /** SVG markup drawn before the text. */
    icon?: string;
    /** SVG markup drawn after the text. */
    trailingIcon?: string;
    disabled?: boolean;
    /** Fills the width of its parent. */
    full?: boolean;
    /** A circle sized to the height, for an icon-only button. */
    round?: boolean;
    /** Layout only, such as margins. */
    style?: Style;
    onpress?: (event: PointerEvent) => void;
    children?: Snippet;
  }

  let {
    variant = "secondary",
    size = "md",
    icon,
    trailingIcon,
    disabled = false,
    full = false,
    round = false,
    style,
    onpress,
    children,
  }: Props = $props();

  const look = $derived(buttonLook(currentTheme(), { variant, size, full, round }));
</script>

<ButtonPrimitive
  style={{ ...look.style, ...style }}
  hover={look.hover}
  active={look.active}
  focus={look.focus}
  disabledStyle={look.disabledStyle}
  {disabled}
  {onpress}
>
  {#if icon}
    <Icon svg={icon} size={buttonIconSizes[size]} />
  {/if}
  {@render children?.()}
  {#if trailingIcon}
    <Icon svg={trailingIcon} size={buttonIconSizes[size]} />
  {/if}
</ButtonPrimitive>
