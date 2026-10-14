// Bundles Grove's main process and the socket server into one CommonJS file.
// Grove's dependencies stay external and are found through NODE_PATH, which is how
// the build can sit outside Grove's tree; the extension-system kernel is ESM only,
// so it is bundled in.

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));
const grove = process.env.GROVE ?? here("../../grove");
const manifest = JSON.parse(readFileSync(`${grove}/package.json`, "utf8"));
const dependencies = Object.keys({ ...manifest.dependencies, ...manifest.devDependencies }).filter(
  (name) => name !== "@neoworks/extension-system",
);

await esbuild.build({
  entryPoints: [here("./server.ts")],
  outfile: here("./dist/server.cjs"),
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node22",
  alias: { electron: here("./electron-main.ts"), "@grove": grove },
  external: [...dependencies, ...dependencies.map((name) => `${name}/*`)],
  logLevel: "info",
});
