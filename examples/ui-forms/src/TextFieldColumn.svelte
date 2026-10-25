<!-- Text fields in each variant and state. -->
<script lang="ts">
  import { Icon, View } from "@neoworks/svelte-gpui/ui";
  import { TextField } from "@neoworks/svelte-gpui-ui";
  import warningCircle from "@phosphor-icons/core/regular/warning-circle.svg";
  import user from "@phosphor-icons/core/regular/user.svg";
  import eye from "@phosphor-icons/core/regular/eye.svg";
  import Section from "./Section.svelte";

  let focusDemo = $state("");
  let name = $state("Ada Lovelace");
  let invalidEmail = $state("ada@");
  let amount = $state("42");
  let site = $state("neoworks");
  let password = $state("secret");
  let bio = $state("Wrote the first program.");
  let handle = $state("ada");
  let outlinedEmpty = $state("");
  let outlinedFilled = $state("Outlined text");
</script>

<View style={{ gap: 18 }}>
  <Section title="Filled">
    <TextField bind:value={focusDemo} label="Empty, first field takes focus" placeholder="Example input" />
    <TextField bind:value={name} label="Full name" supportingText="As on your passport." />
    <TextField bind:value={invalidEmail} label="Email" error supportingText="Add the domain, such as ada@example.com." trailingIcon={warningCircle} />
    <TextField value="Not editable" label="Disabled" disabled />
    <TextField value="Read only text" label="Read only" readonly />
  </Section>
  <Section title="Icon, prefix, suffix, trailing">
    <TextField bind:value={handle} label="Username" icon={user} maxlength={20} supportingText="Letters and digits." />
    <TextField bind:value={amount} label="Amount" prefix="EUR" type="number" />
    <TextField bind:value={site} label="Website" suffix=".neoworks.dev" />
    <TextField bind:value={password} label="Password" type="password">
      {#snippet trailing()}
        <View style={{ size: 40, radius: 20, align: "center", justify: "center" }} hover={{ background: "#00000014" }}>
          <Icon svg={eye} size={20} />
        </View>
      {/snippet}
    </TextField>
  </Section>
  <Section title="Multiline and outlined">
    <TextField bind:value={bio} label="About you" multiline rows={3} />
    <TextField bind:value={outlinedEmpty} label="Outlined, empty" variant="outlined" />
    <TextField bind:value={outlinedFilled} label="Outlined, filled" variant="outlined" />
  </Section>
</View>
