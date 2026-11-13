import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

start(App, {}, { window: { title: "Scrolling", width: 1100, height: 760 } });
