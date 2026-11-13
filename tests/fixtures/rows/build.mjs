import { fileURLToPath } from "node:url";
import { build } from "../../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({
  entry: here("./main.ts"),
  outfile: here("./dist/rows.js"),
});
