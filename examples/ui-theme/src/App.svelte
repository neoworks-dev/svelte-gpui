<!--
  The NeoWorks theme on one page: colour roles, the colour scales, the type scale, radii and
  shadows, drawn from `currentTheme()`. The button switches between light and dark.
-->
<script lang="ts">
  import { Button, Text, View } from "@neoworks/svelte-gpui/ui";
  import {
    currentTheme,
    themeMode,
    toggleThemeMode,
    typeStyle,
    type ColorRole,
    type RadiusRole,
    type ScaleName,
    type ShadowRole,
    type TypeRole,
  } from "@neoworks/svelte-gpui-ui";
  import Heading from "./Heading.svelte";
  import RoleSwatch from "./RoleSwatch.svelte";
  import ScaleRow from "./ScaleRow.svelte";

  const theme = $derived(currentTheme());
  const colorRoles = $derived(Object.keys(theme.colors) as ColorRole[]);
  const scaleNames = $derived(Object.keys(theme.scales) as ScaleName[]);
  const typeRoles = $derived(Object.keys(theme.typeScale) as TypeRole[]);
  const radiusRoles = $derived(Object.keys(theme.radii) as RadiusRole[]);
  const shadowRoles = $derived(Object.keys(theme.shadows) as ShadowRole[]);
  const windowWidth = 1280;

  function toggleLabel(): string {
    if (themeMode() === "light") {
      return "Switch to dark";
    }
    return "Switch to light";
  }
</script>

<View style={{ size: "full", background: theme.colors.canvas, padding: 28, gap: 24 }}>
  <View style={{ direction: "row", align: "center", justify: "between" }}>
    <Text style={{ ...typeStyle("title"), color: theme.colors.text }}>NeoWorks theme, {themeMode()}</Text>
    <Button
      onpress={toggleThemeMode}
      style={{
        paddingX: 14,
        paddingY: 8,
        radius: theme.radii.control,
        background: theme.colors.fill,
        cursor: "pointer",
      }}
      hover={{ background: theme.colors.fillHover }}
    >
      <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.onFill }}>{toggleLabel()}</Text>
    </Button>
  </View>

  <View style={{ direction: "row", gap: 40, flex: 1 }}>
    <View style={{ flex: 1, gap: 24 }}>
      <View style={{ gap: 10 }}>
        <Heading title="Colour roles" />
        <View style={{ direction: "row", wrap: true }}>
          {#each colorRoles as role (role)}
            <RoleSwatch {role} />
          {/each}
        </View>
      </View>
      <View style={{ direction: "row", gap: 40 }}>
        <View style={{ gap: 10 }}>
          <Heading title="Radii" />
          <View style={{ direction: "row", gap: 16 }}>
            {#each radiusRoles as role (role)}
              <View style={{ gap: 6 }}>
                <View
                  style={{
                    size: 72,
                    radius: theme.radii[role],
                    background: theme.colors.surface,
                    borderWidth: 1,
                    borderColor: theme.colors.lineStrong,
                  }}
                />
                <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>{role} {theme.radii[role]}</Text>
              </View>
            {/each}
          </View>
        </View>
        <View style={{ gap: 10 }}>
          <Heading title="Shadows" />
          <View style={{ direction: "row", gap: 28, padding: 12 }}>
            {#each shadowRoles as role (role)}
              <View style={{ gap: 10 }}>
                <View
                  style={{
                    width: 96,
                    height: 72,
                    radius: theme.radii.card,
                    background: theme.colors.surface,
                    shadow: theme.shadows[role],
                  }}
                />
                <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>{role}</Text>
              </View>
            {/each}
          </View>
        </View>
      </View>
    </View>

    <View style={{ flex: 1, gap: 24 }}>
      <View style={{ gap: 6 }}>
        <Heading title="Type scale" />
        {#each typeRoles as role (role)}
          <Text style={{ ...typeStyle(role, windowWidth), color: theme.colors.text, nowrap: true }}>{role}</Text>
        {/each}
      </View>
      <View style={{ gap: 8 }}>
        <Heading title="Scales, steps 1 to 12" />
        {#each scaleNames as name (name)}
          <ScaleRow {name} />
        {/each}
      </View>
    </View>
  </View>
</View>
