<script lang="ts">
  // Form controls with their bindings, each echoing what the app sees.
  let name = $state('Ada');
  let note = $state('One line\nand another');
  let secret = $state('');
  let count = $state(3);
  let agreed = $state(true);
  let size = $state('m');
  let changes = $state(0);
  let pasted = $state('nothing yet');
  let copied = $state('nothing yet');
  let fruit = $state('pear');
  let picks = $state(0);
</script>

<div class="flex h-full flex-col gap-2 overflow-y-auto bg-surface p-3 text-xs text-default">
  <label class="flex flex-col gap-1">
    <span class="text-dim">Name (bind:value, maxlength 12)</span>
    <input
      class="rounded-md border border-line bg-input px-2 py-1 text-xs outline-none"
      bind:value={name}
      maxlength="12"
      placeholder="Your name"
      onchange={() => (changes += 1)}
    />
  </label>
  <div class="text-dim">
    name = <span class="font-mono text-green">{JSON.stringify(name)}</span>
    change events <span class="font-mono text-green">{changes}</span>
  </div>

  <label class="flex flex-col gap-1">
    <span class="text-dim">Note (textarea)</span>
    <textarea
      class="rounded-md border border-line bg-input px-2 py-1 text-xs outline-none"
      rows="3"
      bind:value={note}
    ></textarea>
  </label>
  <div class="text-dim">note has <span class="font-mono text-green">{note.length}</span> characters</div>

  <div class="flex items-center gap-2">
    <input class="w-28 rounded-md border border-line bg-input px-2 py-1" type="password" bind:value={secret} placeholder="Password" />
    <input class="w-16 rounded-md border border-line bg-input px-2 py-1" type="number" bind:value={count} />
    <input class="w-28 rounded-md border border-line bg-input px-2 py-1 text-dim" value="Disabled" disabled />
    <span class="text-dim">
      secret <span class="font-mono text-green">{secret.length}</span>
      count <span class="font-mono text-green">{count}</span>
    </span>
  </div>

  <div class="flex items-center gap-4">
    <label class="flex items-center gap-1.5">
      <input type="checkbox" bind:checked={agreed} />
      <span>Agree <span class="font-mono text-green">{agreed}</span></span>
    </label>
    {#each ['s', 'm', 'l'] as option (option)}
      <label class="flex items-center gap-1.5">
        <input type="radio" name="size" value={option} bind:group={size} />
        <span>{option}</span>
      </label>
    {/each}
    <span class="text-dim">size <span class="font-mono text-green">{size}</span></span>
  </div>

  <div class="flex flex-col gap-1">
    <input
      class="rounded-md border border-line bg-input px-2 py-1"
      value="Select some of this text, then Ctrl+C or Ctrl+X"
      oncopy={() => (copied = 'copy event')}
      oncut={() => (copied = 'cut event')}
      onpaste={(event) => (pasted = JSON.stringify(event.clipboardData.getData('text/plain')))}
    />
    <div class="text-dim">
      copied: <span class="font-mono text-green">{copied}</span>
      pasted: <span class="font-mono text-green">{pasted}</span>
    </div>
  </div>

  <div class="flex items-center gap-2">
    <select
      id="fruit"
      class="w-40 rounded-md border border-line bg-input px-2 py-1"
      bind:value={fruit}
      onchange={() => (picks += 1)}
    >
      <optgroup label="Pome">
        <option value="apple">Apple</option>
        <option value="pear">Pear</option>
      </optgroup>
      <optgroup label="Stone">
        <option value="plum">Plum</option>
        <option value="cherry" disabled>Cherry (sold out)</option>
        <option value="peach">Peach</option>
      </optgroup>
    </select>
    <span class="text-dim">
      fruit <span class="font-mono text-green">{fruit}</span>
      changes <span class="font-mono text-green">{picks}</span>
    </span>
  </div>

  <button
    class="self-start rounded-md border border-line px-2 py-1 hover:bg-hover"
    onclick={() => navigator.clipboard.writeText('from the app: ' + name)}
  >
    Copy the name with navigator.clipboard
  </button>
</div>
