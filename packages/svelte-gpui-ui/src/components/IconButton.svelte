<!--
  A round button with only an icon. With `selected` set it is a toggle. `label` is what the tooltip says.
-->
<script lang="ts">
  import { Button as ButtonPrimitive, Icon, Tooltip, type PointerEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import { currentTheme } from "../theme.svelte";
  import { wrapperAlignment } from "./look";
  import { iconButtonIconSizes, iconButtonLook, type IconButtonSize, type IconButtonVariant } from "./icon-button-look";

  interface Props {
    /** SVG markup. */
    icon: string;
    /** The tooltip's text. */
    label: string;
    /** `standard` for toolbars and lists, `filled` for the strongest emphasis, `tonal` and `outlined` in between. */
    variant?: IconButtonVariant;
    size?: IconButtonSize;
    /** Makes the button a toggle: `true` or `false` is its state, `undefined` means a plain button. */
    selected?: boolean;
    /** Drawn while selected. Without it the same icon stays. */
    selectedIcon?: string;
    disabled?: boolean;
    /** Layout only, such as margins. */
    style?: Style;
    onpress?: (event: PointerEvent) => void;
  }

  let { icon, label, variant = "standard", size = "md", selected, selectedIcon, disabled = false, style, onpress }: Props = $props();

  const look = $derived(iconButtonLook(currentTheme(), { variant, size, selected }));
  const shownIcon = $derived.by(() => {
    if (selected === true && selectedIcon !== undefined) {
      return selectedIcon;
    }
    return icon;
  });
</script>

<Tooltip text={label} {disabled} anchorStyle={wrapperAlignment(style)}>
  <ButtonPrimitive
    style={{ ...look.style, ...style }}
    hover={look.hover}
    active={look.active}
    focus={look.focus}
    disabledStyle={look.disabledStyle}
    {disabled}
    {onpress}
  >
    <Icon svg={shownIcon} size={iconButtonIconSizes[size]} />
  </ButtonPrimitive>
</Tooltip>
