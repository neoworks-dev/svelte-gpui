// Builds Grove's whole GitHub pane, with its real store, against a fake `window.workbench`.

import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));
const grove = process.env.GROVE ?? here("../../../grove");

await build({
  entry: here("./src/main.ts"),
  outfile: here("./dist/pane.js"),
  alias: { "@grove": grove },
  define: {
    "import.meta.env.DEV": "false",
    "import.meta.env.PROD": "true",
    "import.meta.env.MODE": '"production"',
    "import.meta.env": "{}",
  },
  loader: { ".woff2": "empty", ".png": "empty", ".svg": "empty", ".webp": "empty", ".css": "empty" },
});
