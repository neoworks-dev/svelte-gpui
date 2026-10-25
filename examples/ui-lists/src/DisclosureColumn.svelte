<!-- Disclosure and DisclosureGroup: plain, card, exclusive and standalone. -->
<script lang="ts">
  import { Text, View } from "@neoworks/svelte-gpui/ui";
  import { currentTheme, Disclosure, DisclosureGroup, typeStyle } from "@neoworks/svelte-gpui-ui";
  import lock from "@phosphor-icons/core/regular/lock.svg";
  import cloud from "@phosphor-icons/core/regular/cloud.svg";
  import Section from "./Section.svelte";

  const theme = $derived(currentTheme());

  let standaloneOpen = $state(false);
  let bound = $state(true);
</script>

<View style={{ gap: 18 }}>
  <Section title="Plain group, several open at once">
    <DisclosureGroup>
      <Disclosure title="What is NeoWorks?" open>
        <Text>NeoWorks keeps your files, photos and calendar in one place that only you can read.</Text>
      </Disclosure>
      <Disclosure title="Can I leave with my data?" supportingText="Export is always free">
        <Text>Yes. Every app can export what it holds in an open format, at any time.</Text>
      </Disclosure>
      <Disclosure title="Where is my data stored?">
        <Text>On servers in Germany, encrypted before it leaves your device.</Text>
      </Disclosure>
    </DisclosureGroup>
  </Section>

  <Section title="Card group, exclusive, with icons">
    <DisclosureGroup variant="card" exclusive>
      <Disclosure title="Privacy" supportingText="Who can see what" icon={lock} open>
        <Text>Nobody but you can read your data. Not even NeoWorks.</Text>
      </Disclosure>
      <Disclosure title="Storage" supportingText="Space and backups" icon={cloud}>
        <Text>Your plan includes 200 GB, and backups are kept for thirty days.</Text>
      </Disclosure>
      <Disclosure title="Billing" supportingText="Invoices and payment">
        <Text>Invoices arrive by email on the first of each month.</Text>
      </Disclosure>
    </DisclosureGroup>
  </Section>

  <Section title="Standalone, driven by a button">
    <Disclosure title="Standalone disclosure" bind:open={standaloneOpen}>
      <Text>It has no lines of its own, so it sits on the page.</Text>
    </Disclosure>
    <Disclosure title="Bound and open from the start" bind:open={bound}>
      <Text>Its open state is {String(bound)}.</Text>
    </Disclosure>
    <Text style={{ ...typeStyle("caption"), color: theme.colors.muted }}>First one open: {String(standaloneOpen)}</Text>
  </Section>
</View>
