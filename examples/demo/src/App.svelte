<!--
  The demo: one window, a section per thing svelte-gpui does. Every element on
  screen is a primitive from @neoworks/svelte-gpui/ui, drawn by GPUI.
-->
<script lang="ts">
  import type { Component } from "svelte";
  import { Button, Icon, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
  import FrameCounter from "./components/FrameCounter.svelte";
  import { colors, mono } from "./theme";

  import sparkle from "@phosphor-icons/core/regular/sparkle.svg";
  import layout from "@phosphor-icons/core/regular/layout.svg";
  import textAa from "@phosphor-icons/core/regular/text-aa.svg";
  import sliders from "@phosphor-icons/core/regular/sliders-horizontal.svg";
  import rows from "@phosphor-icons/core/regular/rows.svg";
  import cursor from "@phosphor-icons/core/regular/cursor-click.svg";
  import image from "@phosphor-icons/core/regular/image.svg";
  import wave from "@phosphor-icons/core/regular/wave-sine.svg";
  import globe from "@phosphor-icons/core/regular/globe.svg";
  import hardDrives from "@phosphor-icons/core/regular/hard-drives.svg";
  import logo from "@phosphor-icons/core/fill/cube-transparent-fill.svg";

  import Basics from "./sections/Basics.svelte";
  import Layout from "./sections/Layout.svelte";
  import Typography from "./sections/Typography.svelte";
  import Controls from "./sections/Controls.svelte";
  import Lists from "./sections/Lists.svelte";
  import Pointer from "./sections/Pointer.svelte";
  import Media from "./sections/Media.svelte";
  import Motion from "./sections/Motion.svelte";
  import Network from "./sections/Network.svelte";
  import Files from "./sections/Files.svelte";

  interface Section {
    title: string;
    icon: string;
    blurb: string;
    component: Component;
  }

  const sections: Section[] = [
    { title: "Basics", icon: sparkle, component: Basics, blurb: "Runes, blocks and components, as on the web." },
    { title: "Layout", icon: layout, component: Layout, blurb: "Flexbox on every primitive, styled with plain objects." },
    { title: "Text", icon: textAa, component: Typography, blurb: "Paragraphs with styled runs, wrapping and truncation." },
    { title: "Controls", icon: sliders, component: Controls, blurb: "Native text fields, and controls built from primitives." },
    { title: "Lists", icon: rows, component: Lists, blurb: "A hundred thousand rows, built only near the window." },
    { title: "Pointer", icon: cursor, component: Pointer, blurb: "Presses, drags with capture, hover and the wheel." },
    { title: "Media", icon: image, component: Media, blurb: "SVG icons and images, local and remote." },
    { title: "Motion", icon: wave, component: Motion, blurb: "Eased styles, Svelte transitions, tweens and springs." },
    { title: "Network", icon: globe, component: Network, blurb: "fetch and WebSocket, run by the host off the main thread." },
    { title: "Files", icon: hardDrives, component: Files, blurb: "Reading, writing and watching files, within a declared limit." },
  ];

  let current = $state(0);
  const section = $derived(sections[current]);

  function shortcut(event: KeyboardEvent): void {
    if (!event.ctrlKey) {
      return;
    }
    const number = Number(event.key);
    if (number >= 1 && number <= sections.length) {
      current = number - 1;
    } else if (event.key === "ArrowDown") {
      current = (current + 1) % sections.length;
    } else if (event.key === "ArrowUp") {
      current = (current + sections.length - 1) % sections.length;
    }
  }
</script>

<svelte:window onkeydown={shortcut} />

<View style={{ flex: 1, direction: "row", background: colors.window }}>
  <View style={{ width: 216, shrink: 0, padding: 12, gap: 2, background: colors.sidebar, borderRightWidth: 1, borderColor: colors.border }}>
    <View style={{ direction: "row", align: "center", gap: 8, paddingX: 8, paddingTop: 6, paddingBottom: 16 }}>
      <Icon svg={logo} size={20} color={colors.accent} />
      <Text style={{ fontWeight: "bold", fontSize: 15 }}>svelte-gpui</Text>
    </View>
    {#each sections as entry, index (entry.title)}
      {@const active = index === current}
      <Button
        style={{
          gap: 10,
          paddingX: 10,
          paddingY: 7,
          radius: 6,
          background: active ? colors.raised : "transparent",
          color: active ? colors.text : colors.dim,
          transition: 120,
          cursor: "default",
        }}
        hover={active ? undefined : { background: "#ffffff08", color: colors.text }}
        onpress={() => (current = index)}
      >
        <Icon svg={entry.icon} size={16} color={active ? colors.accentText : undefined} />
        <Text style={{ flex: 1 }}>{entry.title}</Text>
        {#if index < 9}<Text style={{ ...mono, fontSize: 11, color: colors.faint }}>^{index + 1}</Text>{/if}
      </Button>
    {/each}
    <View style={{ flex: 1 }} />
    <FrameCounter />
    <View style={{ padding: 8, gap: 4 }}>
      <Text style={{ fontSize: 11, color: colors.faint }}>Svelte 5 in QuickJS, drawn by GPUI.</Text>
      <Text style={{ fontSize: 11, color: colors.faint }}>No DOM, no CSS, no webview.</Text>
    </View>
  </View>

  <View style={{ flex: 1, minWidth: 0 }}>
    <View style={{ paddingX: 28, paddingTop: 22, paddingBottom: 16, gap: 4, borderBottomWidth: 1, borderColor: colors.border }}>
      <Text style={{ fontSize: 22, fontWeight: "bold" }}>{section.title}</Text>
      <Text style={{ color: colors.dim }}>{section.blurb}</Text>
    </View>
    <Scroll style={{ flex: 1 }}>
      <View style={{ padding: 28 }}>
        {#key current}
          <section.component />
        {/key}
      </View>
    </Scroll>
  </View>
</View>
