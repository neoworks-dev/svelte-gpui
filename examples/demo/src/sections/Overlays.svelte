<!--
  Things that float next to what opened them: a popover, tooltips, a menu, a context menu
  and a dropdown. They are drawn above everything, placed after layout against the element
  they belong to, and flip or shift to stay inside the window.
-->
<script lang="ts">
  import { Button, ContextMenu, Icon, Input, Menu, Popover, Select, Text, Tooltip, View, type MenuItem } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import Segmented from "../components/Segmented.svelte";
  import { button, buttonActive, buttonHover, caption, colors, field, fieldFocus, mono } from "../theme";

  import copy from "@phosphor-icons/core/regular/copy.svg";
  import trash from "@phosphor-icons/core/regular/trash.svg";
  import pencil from "@phosphor-icons/core/regular/pencil-simple.svg";
  import share from "@phosphor-icons/core/regular/share-network.svg";
  import dots from "@phosphor-icons/core/regular/dots-three.svg";
  import floppy from "@phosphor-icons/core/regular/floppy-disk.svg";
  import gear from "@phosphor-icons/core/regular/gear.svg";
  import info from "@phosphor-icons/core/regular/info.svg";

  type Side = "top" | "bottom" | "left" | "right";
  type Align = "start" | "center" | "end";

  let side = $state<Side>("bottom");
  let align = $state<Align>("start");
  let popoverOpen = $state(false);
  let note = $state("");

  let menuOpen = $state(false);
  let last = $state("Nothing chosen yet");
  const say = (what: string) => () => (last = what);

  const actions: MenuItem[] = [
    { label: "Rename", icon: pencil, shortcut: "F2", onselect: say("Rename") },
    { label: "Duplicate", icon: copy, shortcut: "Ctrl+D", onselect: say("Duplicate") },
    { label: "Share", icon: share, disabled: true },
    { separator: true },
    { label: "Delete", icon: trash, danger: true, shortcut: "Del", onselect: say("Delete") },
  ];

  let size = $state("Medium");
  let language = $state<number | undefined>(undefined);
  const languages = [
    { value: 1, label: "Rust" },
    { value: 2, label: "TypeScript" },
    { value: 3, label: "Svelte" },
    { value: 4, label: "Zig", disabled: true },
    { value: 5, label: "Go" },
    { value: 6, label: "Python" },
    { value: 7, label: "Elixir" },
    { value: 8, label: "OCaml" },
    { value: 9, label: "Haskell" },
    { value: 10, label: "Lua" },
  ];

  let region = $state("Right-press in the box");
  const regionItems = (): MenuItem[] => [
    { label: "Copy", icon: copy, shortcut: "Ctrl+C", onselect: () => (region = "Copy") },
    { label: "Paste", shortcut: "Ctrl+V", onselect: () => (region = "Paste") },
    { separator: true },
    { label: "Wrap lines", checked: true, onselect: () => (region = "Wrap lines") },
    { label: "Delete", icon: trash, danger: true, onselect: () => (region = "Delete") },
  ];
</script>

<View style={{ gap: 16 }}>
  <Card title="Popover" note="Anchored to the button's box. Pick a side and alignment, then move the window or scroll so it nears an edge: it flips to the other side, and slides back inside.">
    <View style={{ direction: "row", gap: 16, align: "center", wrap: true }}>
      <View style={{ gap: 4 }}>
        <Text style={caption}>Side</Text>
        <Segmented options={["top", "bottom", "left", "right"]} bind:value={side} />
      </View>
      <View style={{ gap: 4 }}>
        <Text style={caption}>Alignment</Text>
        <Segmented options={["start", "center", "end"]} bind:value={align} />
      </View>
    </View>
    <View style={{ height: 56, direction: "row", justify: "center", align: "center", background: colors.window, radius: 8 }}>
      <Popover bind:open={popoverOpen} {side} {align} anchorStyle={{ alignSelf: "center" }}>
        {#snippet trigger({ toggle })}
          <Button style={button} hover={buttonHover} active={buttonActive} onpress={toggle}>
            <Icon svg={info} size={16} />
            <Text>{popoverOpen ? "Close popover" : "Open popover"}</Text>
          </Button>
        {/snippet}
        {#snippet children({ close })}
          <View style={{ width: 240, padding: 8, gap: 8 }}>
            <Text style={{ fontWeight: "semibold" }}>A popover</Text>
            <Text style={{ color: colors.dim }}>Press outside or Escape to close it. Focus moved into this panel, and goes back to the button when it closes.</Text>
            <Input bind:value={note} placeholder="Type here" style={field} focus={fieldFocus} onsubmit={() => close("select")} />
            <Button style={{ ...button, justify: "center" }} hover={buttonHover} onpress={() => close("select")}>
              <Text>Done</Text>
            </Button>
          </View>
        {/snippet}
      </Popover>
    </View>
  </Card>

  <Card title="Tooltips" note="Rest the pointer on a button for half a second. Once one has shown, moving to the next is immediate.">
    <View style={{ direction: "row", gap: 8 }}>
      {#each [{ icon: floppy, label: "Save", text: "Save the document", side: "bottom" }, { icon: gear, label: "Settings", text: "Open the settings", side: "top" }, { icon: share, label: "Share", text: "Share a link with anyone who has the address", side: "right" }] as entry (entry.text)}
        <Tooltip text={entry.text} side={entry.side as Side}>
          <Button style={button} hover={buttonHover} active={buttonActive}>
            <Icon svg={entry.icon} size={16} />
            <Text>{entry.label}</Text>
          </Button>
        </Tooltip>
      {/each}
    </View>
  </Card>

  <Card title="Menus" note="Keys: Down and Up, Home and End, Enter, a letter to jump, Escape to close. Focus returns to the button.">
    <View style={{ direction: "row", gap: 16, align: "center" }}>
      <Popover bind:open={menuOpen} side="bottom" align="start" offset={4} focusPanel={false}>
        {#snippet trigger({ toggle })}
          <Button style={button} hover={buttonHover} active={buttonActive} onpress={toggle}>
            <Icon svg={dots} size={18} />
            <Text>Actions</Text>
          </Button>
        {/snippet}
        {#snippet children({ close })}
          <Menu items={actions} onclose={() => close("select")} />
        {/snippet}
      </Popover>
      <Text style={{ ...mono, color: colors.dim }}>{last}</Text>
    </View>
    <ContextMenu
      items={regionItems}
      style={{ height: 96, align: "center", justify: "center", gap: 4, background: colors.window, radius: 8, borderWidth: 1, borderColor: colors.border }}
    >
      <Text>{region}</Text>
      <Text style={caption}>A context menu opens at the pointer, flipped when it would not fit</Text>
    </ContextMenu>
  </Card>

  <Card title="Select" note="A dropdown as wide as its button. Down, Enter or Space open it from the keyboard; a long list scrolls.">
    <View style={{ direction: "row", gap: 16, align: "center", wrap: true }}>
      <Select bind:value={size} options={["Small", "Medium", "Large"]} style={{ width: 160 }} />
      <Select bind:value={language} options={languages} placeholder="Language" maxHeight={180} style={{ width: 180 }} />
      <Text style={{ ...mono, color: colors.dim }}>{size}, {language === undefined ? "no language" : languages.find((entry) => entry.value === language)?.label}</Text>
    </View>
  </Card>
</View>
