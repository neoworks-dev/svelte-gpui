import { start } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

start(App, {}, { window: { title: "Gradient", width: 700, height: 500 } });
