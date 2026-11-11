import { start, target } from "@neoworks/svelte-gpui";
import App from "./App.svelte";

start(App, { target }, { window: { title: "Fixture app" } });
