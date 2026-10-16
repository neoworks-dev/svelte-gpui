<!--
  The network, through the web's own `fetch` and `WebSocket`. The host runs requests
  and sockets on threads, so the window never waits on them.
-->
<script lang="ts">
  import { Button, Input, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import { button, buttonActive, buttonHover, caption, colors, field, fieldFocus, mono, primaryButton, primaryHover } from "../theme";

  let url = $state("https://api.github.com/zen");
  let busy = $state(false);
  let status = $state("");
  let body = $state("");
  let failed = $state(false);

  async function load(): Promise<void> {
    busy = true;
    failed = false;
    const started = performance.now();
    try {
      const response = await fetch(url, { headers: { accept: "application/json, text/plain, */*" } });
      const text = await response.text();
      const took = Math.round(performance.now() - started);
      status = `${response.status} ${response.statusText} · ${took} ms · ${text.length} characters`;
      body = text.length > 2000 ? `${text.slice(0, 2000)}…` : text;
    } catch (error) {
      failed = true;
      status = (error as Error).message;
      body = "";
    } finally {
      busy = false;
    }
  }

  interface Line {
    id: number;
    out: boolean;
    text: string;
  }

  let address = $state("wss://echo.websocket.org");
  let socket = $state<WebSocket | null>(null);
  let phase = $state("closed");
  let message = $state("Hello from svelte-gpui");
  let lines = $state<Line[]>([]);
  let nextLine = 0;

  function log(out: boolean, text: string): void {
    lines = [...lines, { id: nextLine++, out, text }].slice(-100);
  }

  function connect(): void {
    const opened = new WebSocket(address);
    socket = opened;
    phase = "connecting";
    opened.onopen = () => (phase = "open");
    opened.onmessage = (event) => log(false, typeof event.data === "string" ? event.data : `${event.data.byteLength} bytes`);
    opened.onerror = () => log(false, "error");
    opened.onclose = (event) => {
      phase = "closed";
      log(false, `closed (${event.code}${event.reason ? `, ${event.reason}` : ""})`);
      if (socket === opened) {
        socket = null;
      }
    };
  }

  function send(): void {
    if (socket === null || phase !== "open" || message === "") {
      return;
    }
    socket.send(message);
    log(true, message);
  }

  $effect(() => () => socket?.close());

  const tone = $derived(phase === "open" ? colors.green : phase === "connecting" ? colors.amber : colors.faint);
</script>

<View style={{ gap: 16 }}>
  <Card title="fetch" note="The web's fetch: headers, bodies, status, redirects and AbortController.">
    <View style={{ direction: "row", gap: 8 }}>
      <Input bind:value={url} style={{ ...field, flex: 1 }} focus={fieldFocus} onsubmit={load} />
      <Button style={primaryButton} hover={primaryHover} disabled={busy} onpress={load}><Text>{busy ? "Loading…" : "Fetch"}</Text></Button>
    </View>
    {#if status}
      <Text style={{ ...mono, color: failed ? colors.red : colors.green }}>{status}</Text>
    {/if}
    {#if body}
      <Scroll style={{ maxHeight: 200, padding: 10, radius: 6, background: colors.window, borderWidth: 1, borderColor: colors.border }}>
        <Text pre style={mono}>{body}</Text>
      </Scroll>
    {/if}
  </Card>

  <Card title="WebSocket" note="The web's WebSocket, with text and binary messages; wss:// included.">
    <View style={{ direction: "row", gap: 8, align: "center" }}>
      <Input bind:value={address} style={{ ...field, flex: 1 }} focus={fieldFocus} />
      {#if socket === null}
        <Button style={primaryButton} hover={primaryHover} onpress={connect}><Text>Connect</Text></Button>
      {:else}
        <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => socket?.close(1000, "done")}><Text>Disconnect</Text></Button>
      {/if}
      <View style={{ size: 8, radius: 4, background: tone }} />
      <Text style={{ ...caption, width: 72 }}>{phase}</Text>
    </View>
    <View style={{ direction: "row", gap: 8 }}>
      <Input bind:value={message} style={{ ...field, flex: 1 }} focus={fieldFocus} disabled={phase !== "open"} onsubmit={send} />
      <Button style={button} hover={buttonHover} active={buttonActive} disabled={phase !== "open"} onpress={send}><Text>Send</Text></Button>
    </View>
    <Scroll style={{ height: 180, padding: 8, gap: 4, radius: 6, background: colors.window, borderWidth: 1, borderColor: colors.border }}>
      {#each lines as line (line.id)}
        <Text style={{ ...mono, color: line.out ? colors.accentText : colors.text }}>{line.out ? "→" : "←"} {line.text}</Text>
      {:else}
        <Text style={caption}>Messages appear here.</Text>
      {/each}
    </Scroll>
  </Card>
</View>
