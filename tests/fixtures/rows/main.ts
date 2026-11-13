// The library must be imported before anything that pulls in Svelte's client runtime.
import { start } from "@neoworks/svelte-gpui";
import RowsPage from "./RowsPage.svelte";

start(RowsPage, {}, { window: { title: "Rows", width: 1000, height: 640 } });
