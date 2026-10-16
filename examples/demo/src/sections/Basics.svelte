<!-- Runes, blocks and components work as they do on the web: this is Svelte 5. -->
<script lang="ts">
  import { Button, Icon, Input, Text, View } from "@neoworks/svelte-gpui/ui";
  import plus from "@phosphor-icons/core/regular/plus.svg";
  import minus from "@phosphor-icons/core/regular/minus.svg";
  import trash from "@phosphor-icons/core/regular/trash.svg";
  import Card from "../components/Card.svelte";
  import Checkbox from "../components/Checkbox.svelte";
  import { button, buttonActive, buttonHover, caption, colors, field, fieldFocus, mono, row } from "../theme";

  let count = $state(0);
  const doubled = $derived(count * 2);
  const parity = $derived(count % 2 === 0 ? "even" : "odd");

  interface Todo {
    id: number;
    text: string;
    done: boolean;
  }

  let nextId = 4;
  let todos = $state<Todo[]>([
    { id: 1, text: "Write a component", done: true },
    { id: 2, text: "Compile it with Svelte", done: true },
    { id: 3, text: "Draw it with GPUI", done: false },
  ]);
  let draft = $state("");
  const left = $derived(todos.filter((todo) => !todo.done).length);

  function add(): void {
    const text = draft.trim();
    if (text === "") {
      return;
    }
    todos.push({ id: nextId++, text, done: false });
    draft = "";
  }
</script>

<View style={{ direction: "row", gap: 16, align: "start" }}>
  <Card title="$state and $derived" note="A press changes one number; only the text nodes that show it are sent to the host." style={{ flex: 1 }}>
    <View style={row}>
      <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => count--}>
        <Icon svg={minus} size={14} />
      </Button>
      <Text style={{ fontSize: 28, fontWeight: "bold", minWidth: 56, textAlign: "center" }}>{count}</Text>
      <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => count++}>
        <Icon svg={plus} size={14} />
      </Button>
    </View>
    <Text style={{ color: colors.dim }}>
      doubled is <Text style={{ color: colors.accentText, fontWeight: "semibold" }}>{doubled}</Text>, and the count is
      <Text style={{ color: parity === "even" ? colors.green : colors.amber }}>{parity}</Text>
    </Text>
    {#if count >= 5}
      <Text style={{ color: colors.green }}>{"{#if}"} shows this from five on.</Text>
    {:else if count < 0}
      <Text style={{ color: colors.red }}>Below zero.</Text>
    {/if}
  </Card>

  <Card title="A keyed list" note="Add, tick and remove items; each row keeps its own state." style={{ flex: 1 }}>
    <View style={row}>
      <Input
        bind:value={draft}
        placeholder="Something to do"
        style={{ ...field, flex: 1 }}
        focus={fieldFocus}
        onsubmit={add}
      />
      <Button style={button} hover={buttonHover} active={buttonActive} onpress={add}>
        <Text>Add</Text>
      </Button>
    </View>
    <View style={{ gap: 4 }}>
      {#each todos as todo (todo.id)}
        <View style={{ ...row, justify: "between", paddingY: 2 }}>
          <Checkbox bind:checked={todo.done} label={todo.text} />
          <Button
            style={{ padding: 4, radius: 4, color: colors.faint }}
            hover={{ color: colors.red, background: colors.raised }}
            onpress={() => (todos = todos.filter((other) => other !== todo))}
          >
            <Icon svg={trash} size={14} />
          </Button>
        </View>
      {:else}
        <Text style={caption}>Nothing left.</Text>
      {/each}
    </View>
    <Text style={{ ...mono, color: colors.faint }}>{left} of {todos.length} left</Text>
  </Card>
</View>
