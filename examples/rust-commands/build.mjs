import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({
  entry: here("./ui/main.ts"),
  outfile: here("./dist/rust-commands.js"),
});
