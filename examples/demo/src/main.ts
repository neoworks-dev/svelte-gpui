// The library must be imported before anything that pulls in Svelte's client
// runtime: it installs the DOM globals that runtime reads at module scope.
import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

// The demo only touches its own data directory, and folders picked in a dialog.
start(App, {}, { files: { allow: ["$data/svelte-gpui-demo"] } });
