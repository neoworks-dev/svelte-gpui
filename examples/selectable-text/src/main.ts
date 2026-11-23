import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

start(App, {}, { window: { title: "Selectable text", width: 600, height: 600 } });
