<!--
  A chat of three thousand messages whose heights nobody knows before they are drawn: paragraphs of
  random length, some with a heading, a list or a code block. The list opens at the end, and the
  buttons change it while it is there or scrolled up:

  - "Reopen" mounts the list again, to see it open at the end and how long it takes to settle.
  - "Add message" appends one; "Grow last" makes the last message taller, as streaming text does.
  - "Grow above" makes a row that is built above the window taller, as an image loading would.

  The line under the buttons says where the list is, how many times its rows changed size since
  it was opened and how many rows are built, and the host's log gets one line when it has been
  quiet for a while.
-->
<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { Button, List, Text, View } from "@neoworks/svelte-gpui/ui";
  import Mark from "./Mark.svelte";

  /** A stretch of a paragraph, plain, bold or code, as markdown inlines are. */
  interface Run {
    text: string;
    kind: "plain" | "bold" | "code";
  }

  interface Message {
    id: number;
    author: string;
    heading: string | null;
    paragraphs: Run[][];
    bullets: string[];
    code: string | null;
  }

  const people = ["Ada", "Grace", "Linus", "Margaret", "Edsger"];
  const phrases = [
    "the frame is built once", "layout comes out of flexbox", "scrolling keeps the rows in view",
    "a row is measured as it is drawn", "that sounds right", "I will look at it tomorrow", "can we ship it",
    "the host draws what the app builds", "it works on my machine", "good morning", "no DOM, no CSS",
    "see the attached", "the end stays where it is", "text wraps at the width it is given",
  ];

  /** A repeatable number from 0 up to but not including `limit`, for an id. */
  function roll(id: number, salt: number, limit: number): number {
    let value = Math.imul(id + 1, 2654435761) ^ Math.imul(salt + 1, 40503);
    value = Math.imul(value ^ (value >>> 16), 2246822507) >>> 0;
    value = Math.imul(value ^ (value >>> 13), 3266489909) >>> 0;
    return ((value ^ (value >>> 16)) >>> 0) % limit;
  }

  /** Some sentences of the phrases above, as many as `count` says. */
  function sentences(id: number, salt: number, count: number): string {
    const parts = Array.from({ length: count }, (_, at) => phrases[roll(id, salt + at, phrases.length)]);
    const text = parts.join(", ");
    return text[0].toUpperCase() + text.slice(1) + ".";
  }

  /** A paragraph cut into runs of a few words, some of them bold or code. */
  function runs(id: number, salt: number, count: number): Run[] {
    const words = sentences(id, salt, count).split(" ");
    const out: Run[] = [];
    for (let at = 0, run = 0; at < words.length; run++) {
      const length = 2 + roll(id, salt + run, 4);
      const pick = roll(id, salt + 500 + run, 10);
      out.push({ text: words.slice(at, at + length).join(" ") + " ", kind: pick < 6 ? "plain" : pick < 8 ? "bold" : "code" });
      at += length;
    }
    return out;
  }

  /** Message `id`, made the same every time it is asked for. */
  function message(id: number): Message {
    const paragraphCount = 1 + (roll(id, 1, 10) < 6 ? 0 : roll(id, 2, 3));
    return {
      id,
      author: people[roll(id, 4, people.length)],
      heading: roll(id, 5, 5) === 0 ? sentences(id, 50, 2) : null,
      paragraphs: Array.from({ length: paragraphCount }, (_, at) => runs(id, 100 + at * 20, 1 + roll(id, 6 + at, 22))),
      bullets: roll(id, 7, 4) === 0 ? Array.from({ length: 2 + roll(id, 8, 5) }, (_, at) => sentences(id, 200 + at * 5, 1 + roll(id, 9 + at, 4))) : [],
      code: roll(id, 12, 8) === 0 ? Array.from({ length: 2 + roll(id, 13, 6) }, (_, at) => `let value_${at} = frame(${roll(id, 14 + at, 999)});`).join("\n") : null,
    };
  }

  const COUNT = 3000;
  let messages = $state.raw<number[]>(Array.from({ length: COUNT }, (_, id) => id));
  let newest = COUNT - 1;
  /** Messages made taller by a button, with how many lines they were given. */
  let extra = $state.raw<Record<number, number>>({});
  /** The messages whose rows exist now, so a button can pick one above the window. */
  const built = new SvelteSet<number>();
  let generation = $state(0);
  let position = $state({ y: 0, maxY: 0 });

  // How the list settled: row size changes and scrolls since it was opened, and when the last one was.
  let openedAt = Date.now();
  let lastEventAt = openedAt;
  let firstAtEndAt = -1;
  let resizes = $state(0);
  let scrolls = $state(0);
  let reported = true;
  /** What the app had sent the host when the list was opened, to count only what opening took. */
  const sent: { batches: number; scrollRequests: number } = (globalThis as any).__sent;
  let sentAtOpen = { ...sent };

  /** Notes that something happened to the list, for the settling figures. */
  function touched(): void {
    lastEventAt = Date.now();
    reported = false;
  }

  function reopen(): void {
    sentAtOpen = { ...sent };
    openedAt = Date.now();
    lastEventAt = openedAt;
    firstAtEndAt = -1;
    resizes = 0;
    scrolls = 0;
    reported = false;
    generation++;
  }

  $effect(() => {
    const timer = setInterval(() => {
      if (!reported && Date.now() - lastEventAt > 600) {
        reported = true;
        console.log(
          `settled: ${lastEventAt - openedAt} ms after opening, ${resizes} row size changes, ${scrolls} scroll events, ` +
            `${sent.scrollRequests - sentAtOpen.scrollRequests} scroll requests sent, ${sent.batches - sentAtOpen.batches} batches sent, ` +
            `first at the end after ${firstAtEndAt < 0 ? "never" : firstAtEndAt - openedAt} ms, now y=${Math.round(position.y)} of ${Math.round(position.maxY)}`,
        );
      }
    }, 100);
    return () => clearInterval(timer);
  });

  function scrolled(next: { y: number; maxY: number }): void {
    position = { y: next.y, maxY: next.maxY };
    scrolls++;
    touched();
    if (firstAtEndAt < 0 && next.maxY > 0 && next.y >= next.maxY - 1) {
      firstAtEndAt = Date.now();
    }
  }

  function addMessage(): void {
    messages = [...messages, ++newest];
  }

  /** The message the last button made taller, shown beside the figures. */
  let grown = $state<number | null>(null);

  function grow(id: number): void {
    grown = id;
    extra = { ...extra, [id]: (extra[id] ?? 0) + 6 };
  }

  function growAbove(): void {
    const oldest = Math.min(...built);
    grow(oldest + 2);
  }

  const atEnd = $derived(position.maxY > 0 && position.y >= position.maxY - 4);
  const button = { paddingX: 10, paddingY: 6, radius: 6, background: "#1f1f24" };
</script>

<View style={{ flex: 1, padding: 12, gap: 10, background: "#0c0c0e" }}>
  <View style={{ direction: "row", gap: 8 }}>
    <Button style={button} onpress={reopen}><Text>Reopen</Text></Button>
    <Button style={button} onpress={addMessage}><Text>Add message</Text></Button>
    <Button style={button} onpress={() => grow(newest)}><Text>Grow last</Text></Button>
    <Button style={button} onpress={growAbove}><Text>Grow above</Text></Button>
  </View>
  <Text style={{ fontFamily: "mono", fontSize: 12, color: "#a1a1aa" }}>
    y {Math.round(position.y)} of {Math.round(position.maxY)}, {atEnd ? "at the end" : "not at the end"}, {resizes} row size changes, {scrolls} scroll events, {messages.length} messages, {built.size} rows built{grown === null ? "" : `, grew #${grown}`}
  </Text>
  {#key generation}
    <List
      items={messages}
      key={(id) => id}
      stickToEnd
      onscroll={scrolled}
      style={{ flex: 1, radius: 8, background: "#16161a", borderWidth: 1, borderColor: "#26262c" }}
    >
      {#snippet row(id)}
        {@const item = message(id)}
        <View
          style={{ paddingX: 12, paddingY: 8, gap: 4, borderWidth: 1, borderColor: "#1f1f24" }}
          onresize={() => {
            resizes++;
            touched();
          }}
        >
          <Mark id={item.id} registry={built} />
          <View style={{ direction: "row", gap: 8, align: "baseline" }}>
            <Text style={{ fontWeight: "semibold", color: "#c4b5fd" }}>{item.author}</Text>
            <Text style={{ fontFamily: "mono", fontSize: 11, color: "#6b6b74" }}>#{item.id}</Text>
          </View>
          {#if item.heading}
            <Text style={{ fontWeight: "bold", fontSize: 16 }}>{item.heading}</Text>
          {/if}
          {#each item.paragraphs as paragraph}
            <Text style={{ color: "#d4d4d8" }}>
              {#each paragraph as run}
                {#if run.kind === "bold"}
                  <Text style={{ fontWeight: "bold", color: "#f4f4f5" }}>{run.text}</Text>
                {:else if run.kind === "code"}
                  <Text style={{ fontFamily: "mono", fontSize: 12, color: "#86efac" }}>{run.text}</Text>
                {:else}
                  <Text>{run.text}</Text>
                {/if}
              {/each}
            </Text>
          {/each}
          {#each item.bullets as bullet}
            <Text style={{ color: "#a1a1aa" }}>- {bullet}</Text>
          {/each}
          {#if item.code}
            <Text style={{ fontFamily: "mono", fontSize: 12, color: "#4ade80", background: "#0c0c0e", padding: 6, radius: 4 }}>{item.code}</Text>
          {/if}
          {#each Array.from({ length: extra[item.id] ?? 0 }, (_, at) => at) as line}
            <Text style={{ color: "#fbbf24" }}>Extra line {line + 1} of message #{item.id}.</Text>
          {/each}
        </View>
      {/snippet}
    </List>
  {/key}
</View>
