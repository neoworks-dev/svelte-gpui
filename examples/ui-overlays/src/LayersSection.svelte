<!-- Dialogs, modal sheets and snackbars, each opened by its button. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import warningIcon from "@phosphor-icons/core/regular/warning.svg";
  import { BottomSheet, Button, Dialog, ListRow, SideSheet, Snackbar, TextField, currentTheme, typeStyle } from "@neoworks/svelte-gpui-ui";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());
  let basicOpen = $state(false);
  let lockedOpen = $state(false);
  let fullscreenOpen = $state(false);
  let bottomOpen = $state(false);
  let endOpen = $state(false);
  let startOpen = $state(false);
  let snackbarOpen = $state(false);
  let undoOpen = $state(false);
  let eventTitle = $state("");
  let outcome = $state("nothing yet");
</script>

<Section title="Dialog, sheets and snackbar">
  <View style={{ gap: 10, align: "start" }}>
    <View style={{ direction: "row", gap: 8, wrap: true }}>
      <Button onpress={() => (basicOpen = true)}>Basic dialog</Button>
      <Button onpress={() => (lockedOpen = true)}>Locked dialog</Button>
      <Button onpress={() => (fullscreenOpen = true)}>Fullscreen dialog</Button>
    </View>
    <View style={{ direction: "row", gap: 8, wrap: true }}>
      <Button onpress={() => (bottomOpen = true)}>Bottom sheet</Button>
      <Button onpress={() => (endOpen = true)}>Side sheet end</Button>
      <Button onpress={() => (startOpen = true)}>Side sheet start</Button>
    </View>
    <View style={{ direction: "row", gap: 8, wrap: true }}>
      <Button onpress={() => (snackbarOpen = true)}>Snackbar</Button>
      <Button onpress={() => (undoOpen = true)}>Snackbar with undo</Button>
    </View>
    <Text style={{ ...typeStyle("bodySmall"), color: theme.colors.muted }}>Outcome: {outcome}</Text>
  </View>
</Section>

<Dialog bind:open={basicOpen} title="Delete this album?" icon={warningIcon} onclose={() => (outcome = "basic closed")}>
  The 48 photos in it stay in your library. This cannot be undone.
  {#snippet actions()}
    <Button variant="ghost" onpress={() => (basicOpen = false)}>Cancel</Button>
    <Button variant="primary" onpress={() => ((basicOpen = false), (outcome = "album deleted"))}>Delete album</Button>
  {/snippet}
</Dialog>

<Dialog bind:open={lockedOpen} title="Choose how to continue" dismissible={false}>
  Escape and a press outside do nothing while a choice must be made.
  {#snippet actions()}
    <Button variant="primary" onpress={() => (lockedOpen = false)}>Continue</Button>
  {/snippet}
</Dialog>

<Dialog bind:open={fullscreenOpen} title="New event" variant="fullscreen">
  <View style={{ gap: 16, maxWidth: 480 }}>
    <TextField bind:value={eventTitle} label="Event title" />
    <ListRow title="Calendar" subtitle="Personal" chevron />
  </View>
  {#snippet actions()}
    <Button variant="primary" onpress={() => (fullscreenOpen = false)}>Save</Button>
  {/snippet}
</Dialog>

<BottomSheet bind:open={bottomOpen} title="Share album">
  <View style={{ gap: 8 }}>
    <ListRow title="Copy link" chevron />
    <ListRow title="Invite people" subtitle="Anyone with the link can view" chevron />
  </View>
</BottomSheet>

<SideSheet bind:open={endOpen} title="Event details">
  <Text style={{ ...typeStyle("body"), color: theme.colors.muted }}>Friday 10 October, 18:30 at the lake house. Bring something to grill.</Text>
  {#snippet actions()}
    <Button variant="ghost" onpress={() => (endOpen = false)}>Close</Button>
    <Button variant="primary">Edit</Button>
  {/snippet}
</SideSheet>

<SideSheet bind:open={startOpen} side="start" title="Filters">
  <Text style={{ ...typeStyle("body"), color: theme.colors.muted }}>Anything that narrows the list goes here.</Text>
</SideSheet>

<Snackbar bind:open={snackbarOpen} message="Photo moved to the bin" duration={20000} dismissible />
<Snackbar bind:open={undoOpen} message="Album archived" actionLabel="Undo" onaction={() => (outcome = "undo pressed")} duration={20000} />
