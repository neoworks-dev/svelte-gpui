// The library must be imported before anything that pulls in Svelte's client
// runtime: it installs the DOM globals that runtime reads at module scope.
import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

// The demo only touches its own data directory, and folders picked in a dialog. It opens
// its window at 1100 by 760, and the user cannot make it smaller than 720 by 480.
start(
  App,
  {},
  {
    files: { allow: ["$data/svelte-gpui-demo"] },
    window: { title: "svelte-gpui demo", width: 1100, height: 760, minWidth: 720, minHeight: 480 },
  },
);
