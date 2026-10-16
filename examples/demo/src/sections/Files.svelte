<!--
  Files, through `files` from @neoworks/svelte-gpui: a note saved in the app's data
  directory, and a folder browser that follows changes on disk with `watch`. The
  demo limits itself to its data directory and what you pick in a dialog.
-->
<script lang="ts">
  import { files, type FileEntry, type FilePaths } from "@neoworks/svelte-gpui";
  import { Button, Icon, Input, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
  import folder from "@phosphor-icons/core/regular/folder.svg";
  import file from "@phosphor-icons/core/regular/file.svg";
  import up from "@phosphor-icons/core/regular/arrow-up.svg";
  import Card from "../components/Card.svelte";
  import { unsaved } from "../unsaved.svelte";
  import { button, buttonActive, buttonHover, caption, colors, field, fieldFocus, mono, primaryButton, primaryHover } from "../theme";

  let paths = $state<FilePaths | null>(null);
  let note = $state("");
  let saved = $state("");
  // What the note file holds, to tell whether the field has changes the file does not.
  let onDisk = $state("");
  let notePath = $state("");

  let directory = $state<string | null>(null);
  let entries = $state<FileEntry[]>([]);
  let changes = $state(0);
  let problem = $state("");

  $effect(() => {
    void load();
  });

  // The window asks before closing while the note differs from the file (see App.svelte).
  $effect(() => {
    unsaved.dirty = note !== onDisk;
  });
  $effect(() => () => {
    unsaved.dirty = false;
  });

  async function load(): Promise<void> {
    paths = await files.paths();
    const home = `${paths.data}/svelte-gpui-demo`;
    notePath = `${home}/note.md`;
    try {
      await files.mkdir(home, { recursive: true });
      if (await files.exists(notePath)) {
        note = await files.readText(notePath);
        onDisk = note;
        saved = "Loaded";
      }
    } catch (error) {
      // Refused when the host was started with an --allow-files that leaves this out.
      saved = (error as Error).message;
    }
  }

  async function save(): Promise<void> {
    try {
      await files.writeText(notePath, note);
      onDisk = note;
      saved = `Saved at ${new Date().toLocaleTimeString()}`;
    } catch (error) {
      saved = (error as Error).message;
    }
  }

  async function choose(): Promise<void> {
    const picked = await files.pickFiles({ directories: true, prompt: "Show" });
    if (picked !== null && picked.length > 0) {
      directory = picked[0];
    }
  }

  async function refresh(path: string): Promise<void> {
    try {
      entries = (await files.list(path)).sort((a, b) => Number(b.kind === "directory") - Number(a.kind === "directory"));
      problem = "";
    } catch (error) {
      entries = [];
      problem = (error as Error).message;
    }
  }

  // The listing follows the disk: a file created, changed or removed in the folder
  // shows up without asking.
  $effect(() => {
    const path = directory;
    if (path === null) {
      return;
    }
    void refresh(path);
    return files.watch(path, () => {
      changes++;
      void refresh(path);
    });
  });

  function parent(path: string): string {
    return path.replace(/\/[^/]+\/?$/, "") || "/";
  }

  function size(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }
</script>

<View style={{ gap: 16 }}>
  <View style={{ direction: "row", gap: 16, align: "start" }}>
    <Card title="A note on disk" note="Saved in the app's data directory when focus leaves the field, or with the button." style={{ flex: 1 }}>
      <Input bind:value={note} multiline rows={6} placeholder="Write something" style={field} focus={fieldFocus} oncommit={save} />
      <View style={{ direction: "row", align: "center", gap: 8 }}>
        <Button style={primaryButton} hover={primaryHover} onpress={save}><Text>Save note</Text></Button>
        <Text style={caption}>{saved}</Text>
      </View>
      <Text style={{ ...mono, color: colors.faint, truncate: true }}>{notePath}</Text>
    </Card>

    <Card title="Where things go" note="files.paths(), for building absolute paths." style={{ width: 340 }}>
      {#if paths}
        {#each Object.entries(paths) as [name, path] (name)}
          <View style={{ direction: "row", gap: 8 }}>
            <Text style={{ ...mono, width: 56, color: colors.faint }}>{name}</Text>
            <Text style={{ ...mono, flex: 1, minWidth: 0, truncate: true }}>{path}</Text>
          </View>
        {/each}
      {/if}
    </Card>
  </View>

  <Card title="A folder, watched" note="Pick a folder; the list follows files being created, changed and removed in it.">
    <View style={{ direction: "row", align: "center", gap: 8 }}>
      <Button style={button} hover={buttonHover} active={buttonActive} onpress={choose}><Text>Choose a folder…</Text></Button>
      {#if directory !== null}
        <Button style={button} hover={buttonHover} active={buttonActive} onpress={() => (directory = parent(directory!))}>
          <Icon svg={up} size={14} />
        </Button>
        <Text style={{ ...mono, flex: 1, minWidth: 0, truncate: true, color: colors.dim }}>{directory}</Text>
        <Text style={caption}>{changes} changes seen</Text>
      {/if}
    </View>
    {#if problem}
      <Text style={{ color: colors.red }}>{problem}</Text>
    {/if}
    {#if directory !== null}
      <Scroll style={{ height: 260, radius: 6, background: colors.window, borderWidth: 1, borderColor: colors.border }}>
        {#each entries.slice(0, 500) as entry (entry.path)}
          <Button
            style={{ gap: 8, paddingX: 10, height: 28, shrink: 0 }}
            hover={{ background: colors.raised }}
            onpress={() => entry.kind === "directory" && (directory = entry.path)}
          >
            <Icon svg={entry.kind === "directory" ? folder : file} size={14} color={entry.kind === "directory" ? colors.accentText : colors.faint} />
            <Text style={{ flex: 1, minWidth: 0, truncate: true }}>{entry.name}</Text>
            {#if entry.kind === "file"}<Text style={{ ...mono, color: colors.faint }}>{size(entry.size)}</Text>{/if}
          </Button>
        {/each}
      </Scroll>
    {/if}
  </Card>
</View>
