import { cp } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({
  entry: here("./src/main.ts"),
  outfile: here("./dist/demo.js"),
});

// Images the app shows by path; the host resolves them relative to the bundle.
await cp(here("./assets"), here("./dist/assets"), { recursive: true });
