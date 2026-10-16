<!--
  Text fields are native: the host edits the text, so input methods, selection and
  the clipboard work. Everything else here is a Svelte component made of primitives.
-->
<script lang="ts">
  import { Button, Input, Text, View, type KeyEvent } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import Checkbox from "../components/Checkbox.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Slider from "../components/Slider.svelte";
  import Switch from "../components/Switch.svelte";
  import { button, buttonActive, buttonHover, caption, colors, field, fieldFocus, mono, primaryButton, primaryHover, row } from "../theme";

  let name = $state("");
  let password = $state("");
  let amount = $state("42");
  let notes = $state("Several lines.\nEnter breaks the line here.");
  let submitted = $state<string[]>([]);

  let notify = $state(true);
  let compact = $state(false);
  let agree = $state(false);
  let volume = $state(60);
  let size = $state<"S" | "M" | "L">("M");

  let keys = $state<string[]>([]);

  function logKey(event: KeyEvent): void {
    const parts = [event.ctrl && "Ctrl", event.alt && "Alt", event.shift && "Shift", event.meta && "Meta", event.key];
    keys = [parts.filter(Boolean).join("+"), ...keys].slice(0, 6);
    event.preventDefault();
  }

  const strength = $derived(Math.min(4, Math.floor(password.length / 3)));
</script>

<View style={{ gap: 16 }}>
  <View style={{ direction: "row", gap: 16, align: "start" }}>
    <Card title="Text fields" style={{ flex: 1 }}>
      <View style={{ gap: 4 }}>
        <Text style={caption}>Name, submitted with Enter</Text>
        <Input
          bind:value={name}
          placeholder="Ada Lovelace"
          style={field}
          focus={fieldFocus}
          onsubmit={(value) => (submitted = [value, ...submitted].slice(0, 3))}
        />
      </View>
      <View style={{ gap: 4 }}>
        <Text style={caption}>Password</Text>
        <Input bind:value={password} secure placeholder="At least twelve characters" style={field} focus={fieldFocus} />
        <View style={{ direction: "row", gap: 4 }}>
          {#each [0, 1, 2, 3] as step (step)}
            <View
              style={{
                flex: 1,
                height: 3,
                radius: 2,
                background: step < strength ? [colors.red, colors.amber, colors.blue, colors.green][strength - 1] : colors.border,
                transition: 200,
              }}
            />
          {/each}
        </View>
      </View>
      <View style={{ gap: 4 }}>
        <Text style={caption}>A number, filtered as it is typed</Text>
        <Input bind:value={amount} numeric maxLength={8} style={{ ...field, width: 120 }} focus={fieldFocus} />
      </View>
      <View style={{ gap: 4 }}>
        <Text style={caption}>Multiline</Text>
        <Input bind:value={notes} multiline rows={3} style={field} focus={fieldFocus} />
      </View>
      {#if submitted.length > 0}
        <Text style={{ ...mono, color: colors.green }}>submitted: {submitted.join(", ")}</Text>
      {/if}
    </Card>

    <Card title="Made of primitives" note="Checkbox, switch, slider and segmented control are ordinary Svelte components." style={{ flex: 1 }}>
      <Switch bind:on={notify} label="Notifications" />
      <Switch bind:on={compact} label="Compact rows" />
      <Checkbox bind:checked={agree} label="I have read the terms" />
      <View style={{ gap: 4 }}>
        <Text style={caption}>Volume</Text>
        <Slider bind:value={volume} />
      </View>
      <View style={{ gap: 4 }}>
        <Text style={caption}>Size</Text>
        <Segmented options={["S", "M", "L"]} bind:value={size} />
      </View>
      <View style={row}>
        <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => ((notify = true), (compact = false), (agree = false), (volume = 60), (size = "M"))}>
          <Text>Reset</Text>
        </Button>
        <Button style={primaryButton} hover={primaryHover} disabled={!agree} disabledStyle={{ opacity: 0.4 }}>
          <Text style={{ color: "#ffffff" }}>Continue</Text>
        </Button>
      </View>
      <Text style={{ ...mono, color: colors.faint }}>
        notify: {notify}, compact: {compact}, agree: {agree}, volume: {volume}, size: {size}
      </Text>
    </Card>
  </View>

  <Card title="Keys" note="Press this box, then type: it takes focus and hears every key, Ctrl and arrows included.">
    <View
      focusable
      style={{ padding: 12, gap: 6, radius: 8, background: colors.window, borderWidth: 1, borderColor: colors.border, minHeight: 72 }}
      focus={{ borderColor: colors.accent }}
      onkey={logKey}
    >
      {#if keys.length === 0}
        <Text style={caption}>Nothing typed yet.</Text>
      {:else}
        <View style={{ direction: "row", gap: 6, wrap: true }}>
          {#each keys as key, index (index + key)}
            <View style={{ paddingX: 8, paddingY: 3, radius: 5, background: index === 0 ? colors.accentSoft : colors.raised, borderWidth: 1, borderColor: colors.strongBorder }}>
              <Text style={{ ...mono, color: index === 0 ? colors.accentText : colors.dim }}>{key}</Text>
            </View>
          {/each}
        </View>
      {/if}
    </View>
  </Card>
</View>
