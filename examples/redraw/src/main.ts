// The library must be imported before anything that pulls in Svelte's client runtime.
import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

start(App, {}, { window: { title: "Redraw", width: 900, height: 600 } });
