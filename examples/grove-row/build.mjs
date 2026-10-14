// Builds one of Grove's own components, unmodified, into a window. GROVE names a
// checkout of grove; the component is imported from there, and the stylesheet is
// the one Grove's own `main.css` compiles to (see the README for the command).

import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));
const grove = process.env.GROVE ?? here("../../../grove");

// Modules that reach for the main process are swapped for stubs; the component
// under test only needs them to exist.
const stubs = {
  name: "grove-stubs",
  setup(build) {
    build.onResolve({ filter: /store\.svelte$/ }, (args) => {
      if (!args.importer.includes("/plugins/github/")) {
        return null;
      }
      return { path: here("./src/store-stub.ts") };
    });
  },
};

await build({
  entry: here("./src/main.ts"),
  outfile: here("./dist/row.js"),
  alias: { "@grove": grove },
  plugins: [stubs],
});
