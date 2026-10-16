// The library must be imported before anything that pulls in Svelte's client
// runtime: it installs the DOM globals that runtime reads at module scope.
import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";
import "./commands";

start(App, {}, { window: { title: "Rust commands", width: 760, height: 820 } });
