// The fake has to exist before any Grove module is evaluated.
import "./fake";
import { start } from "@neoworks/svelte-gpui";
import Harness from "./Harness.svelte";

start(Harness);
