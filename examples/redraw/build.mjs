import { cp } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({ entry: here("./src/main.ts"), outfile: here("./dist/redraw.js"), watch: process.argv.includes("--watch") });

// The icons the rows show by path; the host resolves them relative to the bundle.
await cp(here("./assets"), here("./dist/assets"), { recursive: true });
