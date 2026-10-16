<!--
  Calls into the Rust program in src/main.rs: `invoke` runs a command on a thread and
  resolves with its answer, `listen` hears the events Rust emits.
-->
<script lang="ts">
  import { invoke, invokeBytes, listen } from "@neoworks/svelte-gpui";
  import { Button, Input, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
  import Card from "./Card.svelte";
  import type { Process, Tick } from "./commands";
  import { button, colors, field, mono } from "./theme";

  let name = $state("GPUI");
  let greeting = $state("");

  let path = $state("/proc/self/exe");
  let hash = $state("");

  let processes = $state<Process[]>([]);

  let bytesResult = $state("");

  let waiting = $state("");

  let problem = $state("");

  let ticks = $state(0);
  let latest = $state<Tick | null>(null);
  let stopListening = $state<(() => void) | null>(null);

  /** Shows a failure the way an app would: the message, and the code if the command gave one. */
  function describe(error: unknown): string {
    const { message, code } = error as Error & { code?: string };
    return code === undefined ? message : `${code}: ${message}`;
  }

  async function greet(): Promise<void> {
    try {
      greeting = await invoke("greet", { name });
    } catch (error) {
      greeting = describe(error);
    }
  }

  async function hashFile(): Promise<void> {
    hash = "hashing...";
    try {
      const { sha256, size, millis } = await invoke("hash_file", { path });
      hash = `${sha256}\n${size.toLocaleString()} bytes in ${millis} ms`;
    } catch (error) {
      hash = describe(error);
    }
  }

  async function listProcesses(): Promise<void> {
    processes = await invoke("list_processes", { limit: 8 });
  }

  async function reverse(): Promise<void> {
    const sent = new TextEncoder().encode("Hello from the UI");
    const { value, bytes } = await invokeBytes<{ length: number; checksum: number }>("reverse_bytes", null, sent);
    bytesResult = `${new TextDecoder().decode(bytes)}\nRust saw ${value.length} bytes, checksum ${value.checksum}`;
  }

  async function wait(): Promise<void> {
    waiting = "waiting...";
    waiting = await invoke("wait", { millis: 800 });
  }

  async function callMissing(): Promise<void> {
    try {
      await invoke("does_not_exist");
    } catch (error) {
      problem = describe(error);
    }
  }

  async function callFailing(): Promise<void> {
    try {
      await invoke("fail", null);
    } catch (error) {
      problem = describe(error);
    }
  }

  function toggleTicks(): void {
    if (stopListening !== null) {
      stopListening();
      stopListening = null;
      return;
    }
    stopListening = listen("tick", (tick) => {
      ticks += 1;
      latest = tick;
    });
  }

  $effect(() => {
    void listProcesses();
    return () => stopListening?.();
  });
</script>

<Scroll style={{ flex: 1, background: colors.window }}>
  <View style={{ padding: 20, gap: 14, color: colors.text, fontSize: 13 }}>
    <Text style={{ fontSize: 22, fontWeight: "bold" }}>Rust commands</Text>

    <Card title="A command" note="invoke('greet', { name }) runs a Rust closure on a thread.">
      <View style={{ direction: "row", gap: 8 }}>
        <Input bind:value={name} placeholder="Name" style={field} onsubmit={greet} />
        <Button style={button} hover={{ background: colors.hover }} onpress={greet}><Text>Greet</Text></Button>
      </View>
      {#if greeting}<Text style={{ color: colors.green }}>{greeting}</Text>{/if}
    </Card>

    <Card title="Real work" note="Rust reads the file in chunks and hashes it; the window stays responsive.">
      <View style={{ direction: "row", gap: 8 }}>
        <Input bind:value={path} placeholder="Path" style={field} onsubmit={hashFile} />
        <Button style={button} hover={{ background: colors.hover }} onpress={hashFile}><Text>SHA-256</Text></Button>
      </View>
      {#if hash}<Text pre style={mono}>{hash}</Text>{/if}
    </Card>

    <Card title="Processes" note="Read from /proc by Rust: the eight using the most memory.">
      <View style={{ gap: 4 }}>
        {#each processes as process (process.pid)}
          <View style={{ direction: "row", gap: 8 }}>
            <Text style={{ ...mono, width: 64 }}>{process.pid}</Text>
            <Text style={{ flex: 1, minWidth: 0, truncate: true }}>{process.name}</Text>
            <Text style={mono}>{(process.memoryKb / 1024).toFixed(0)} MB</Text>
          </View>
        {/each}
      </View>
      <Button style={{ ...button, alignSelf: "start" }} hover={{ background: colors.hover }} onpress={listProcesses}>
        <Text>Refresh</Text>
      </Button>
    </Card>

    <Card title="Bytes" note="invokeBytes sends bytes to Rust and gets bytes back, with no JSON in between.">
      <Button style={{ ...button, alignSelf: "start" }} hover={{ background: colors.hover }} onpress={reverse}>
        <Text>Reverse "Hello from the UI"</Text>
      </Button>
      {#if bytesResult}<Text pre style={mono}>{bytesResult}</Text>{/if}
    </Card>

    <Card title="An async command" note="command_async awaits another thread before it answers.">
      <Button style={{ ...button, alignSelf: "start" }} hover={{ background: colors.hover }} onpress={wait}>
        <Text>Wait 800 ms</Text>
      </Button>
      {#if waiting}<Text style={mono}>{waiting}</Text>{/if}
    </Card>

    <Card title="Errors" note="A failed command rejects with an Error; its code says which kind.">
      <View style={{ direction: "row", gap: 8 }}>
        <Button style={button} hover={{ background: colors.hover }} onpress={callMissing}><Text>Unknown command</Text></Button>
        <Button style={button} hover={{ background: colors.hover }} onpress={callFailing}><Text>Failing command</Text></Button>
      </View>
      {#if problem}<Text style={{ color: colors.red }}>{problem}</Text>{/if}
    </Card>

    <Card title="Events from Rust" note="A Rust thread emits 'tick' once a second, only while something listens.">
      <Button
        style={{ ...button, alignSelf: "start", background: stopListening === null ? colors.raised : colors.accent }}
        hover={{ background: colors.hover }}
        onpress={toggleTicks}
      >
        <Text>{stopListening === null ? "Listen" : "Stop listening"}</Text>
      </Button>
      <Text style={mono}>
        {#if latest}
          {ticks} events. Load {latest.load.join(" ")}, memory in use {latest.memoryUsedMb} MB
        {:else}
          Nothing heard yet.
        {/if}
      </Text>
    </Card>
  </View>
</Scroll>
