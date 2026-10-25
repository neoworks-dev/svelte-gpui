// The library must be imported before anything that pulls in Svelte's client runtime.
import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

start(App, {}, { window: { title: "NeoWorks theme", width: 1280, height: 900 } });
