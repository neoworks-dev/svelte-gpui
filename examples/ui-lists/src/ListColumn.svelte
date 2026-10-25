<!-- List and ListItem: card and plain, with and without dividers, and every part of a row. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import {
    currentTheme,
    List,
    ListItem,
    SectionHeader,
    StatusBadge,
    Switch,
    typeStyle,
  } from "@neoworks/svelte-gpui-ui";
  import bell from "@phosphor-icons/core/regular/bell.svg";
  import cloud from "@phosphor-icons/core/regular/cloud.svg";
  import folder from "@phosphor-icons/core/regular/folder.svg";
  import gear from "@phosphor-icons/core/regular/gear.svg";
  import lock from "@phosphor-icons/core/regular/lock.svg";
  import user from "@phosphor-icons/core/regular/user.svg";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  let opened = $state("nothing yet");
  let selectedName = $state("Photos");
  let notifications = $state(true);

  const folders = ["Photos", "Documents", "Music"];
</script>

<View style={{ gap: 18 }}>
  <Section title="Settings list, card with dividers">
    <List dividers>
      <ListItem headline="Account" supportingText="Name, email and profile picture" icon={user} chevron onpress={() => (opened = "Account")} />
      <ListItem headline="Security" overline="Sign in" supportingText="Password, devices and recovery" icon={lock} chevron onpress={() => (opened = "Security")} />
      <ListItem headline="Storage" supportingText="Where NeoWorks keeps your files" icon={cloud} trailingText="12.4 GB" onpress={() => (opened = "Storage")} />
      <ListItem headline="Notifications" supportingText="Sounds and banners" icon={bell}>
        {#snippet trailing()}<Switch bind:checked={notifications} label="Notifications" hideLabel inline />{/snippet}
      </ListItem>
      <ListItem headline="Advanced" supportingText="Not available on this device" icon={gear} chevron disabled onpress={() => (opened = "Advanced")} />
    </List>
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>Opened: {opened}</Text>
  </Section>

  <Section title="Selected row">
    <List dividers>
      {#each folders as name (name)}
        <ListItem headline={name} supportingText="Folder" icon={folder} selected={selectedName === name} onpress={() => (selectedName = name)} />
      {/each}
    </List>
  </Section>

  <Section title="Plain list with dividers and badges">
    <List variant="plain" dividers>
      <ListItem headline="Backup finished" supportingText="2026-10-25 18:28" trailingText="3 min">
        {#snippet trailing()}<StatusBadge tone="success" dot>Done</StatusBadge>{/snippet}
      </ListItem>
      <ListItem headline="Sync paused" supportingText="Waiting for a network" trailingText="1 h">
        {#snippet trailing()}<StatusBadge tone="warning" dot>Paused</StatusBadge>{/snippet}
      </ListItem>
      <ListItem headline="Upload failed" supportingText="The file is larger than the limit" trailingText="Yesterday">
        {#snippet trailing()}<StatusBadge tone="danger" dot>Failed</StatusBadge>{/snippet}
      </ListItem>
    </List>
  </Section>

  <Section title="Card without dividers, and three line clamp">
    <List>
      <ListItem headline="One line only" supportingText="This supporting text is long on purpose so that the row has to cut it off after a single line of text." lines={1} />
      <ListItem headline="Up to three lines" overline="Release notes" supportingText="This supporting text is long on purpose so that the row may use up to three lines before it is cut off with an ellipsis, which is what the lines property allows." lines={3} />
    </List>
  </Section>

  <Section title="Section header">
    <SectionHeader title="Recent files" description="Everything you opened this week" style={{ paddingBottom: 0 }}>
      {#snippet action()}<Text style={{ color: theme.colors.accent, fontWeight: 500 }}>See all</Text>{/snippet}
    </SectionHeader>
    <SectionHeader title="Only a title" style={{ paddingBottom: 0 }} />
  </Section>
</View>
