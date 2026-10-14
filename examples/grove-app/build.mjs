// Builds Grove's own shell: the kernel boot and App.svelte from the renderer, as
// Grove's main.ts does them, against a fake `window.workbench`.

import { readdir, readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));
const grove = process.env.GROVE ?? here("../../../grove");

// Fonts and pictures are files the host reads, so importing one gives its path.
const assets = {
  name: "grove-assets",
  setup(build) {
    build.onLoad({ filter: /\.(png|webp|jpg|jpeg|gif|svg|woff2?|ttf)$/ }, (args) => ({
      contents: `export default ${JSON.stringify(args.path)}`,
      loader: "js",
    }));
    // The nvim grid is drawn by the host, so Grove's canvas renderer is swapped for one
    // that sends it rows.
    build.onResolve({ filter: /\/canvasRenderer$/ }, (args) => {
      if (!args.importer.includes("/lib/nvim/")) {
        return null;
      }
      return { path: here("../../bridge/hostGridRenderer.ts") };
    });
    // There is no WebAssembly, so Shiki matches its grammars with its JavaScript regex
    // engine instead of Oniguruma, which also keeps the inlined wasm out of the bundle.
    build.onResolve({ filter: /^@shikijs\/engine-oniguruma(\/wasm-inlined)?$/ }, (args) => ({
      path: args.path,
      namespace: "grove-shiki",
    }));
    build.onLoad({ filter: /.*/, namespace: "grove-shiki" }, async (args) => {
      if (args.path.endsWith("/wasm-inlined")) {
        return { contents: "export default null;", loader: "js" };
      }
      const engine = (await readdir(here(`${grove}/node_modules/.bun`))).find((name) =>
        name.startsWith("@shikijs+engine-javascript@"),
      );
      const entry = `${grove}/node_modules/.bun/${engine}/node_modules/@shikijs/engine-javascript/dist/index.mjs`;
      return {
        loader: "js",
        resolveDir: grove,
        contents: `import { createJavaScriptRegexEngine } from ${JSON.stringify(entry)};
          export async function createOnigurumaEngine() { return createJavaScriptRegexEngine(); }
          export async function loadWasm() {}
          export function setDefaultWasmLoader() {}
          export function getDefaultWasmLoader() {}`,
      };
    });
    // `?url` gives a file's path and `?raw` its text, as Vite's suffixes do.
    build.onResolve({ filter: /\?(url|raw)$/ }, async (args) => {
      const kind = args.path.endsWith("?raw") ? "raw" : "url";
      const target = args.path.replace(/\?(url|raw)$/, "");
      const resolved = await build.resolve(target, { kind: args.kind, resolveDir: args.resolveDir });
      if (resolved.errors.length > 0) {
        return { errors: resolved.errors };
      }
      return { path: resolved.path, namespace: `grove-${kind}` };
    });
    build.onLoad({ filter: /.*/, namespace: "grove-url" }, (args) => ({
      contents: `export default ${JSON.stringify(args.path)}`,
      loader: "js",
    }));
    build.onLoad({ filter: /.*/, namespace: "grove-raw" }, async (args) => ({
      contents: `export default ${JSON.stringify(await readFile(args.path, "utf8"))}`,
      loader: "js",
    }));
  },
};

await build({
  entry: here("./src/main.ts"),
  outfile: here("./dist/app.js"),
  alias: { "@grove": grove, electron: here("../../bridge/electron-renderer.ts") },
  define: {
    "import.meta.env.DEV": "false",
    "import.meta.env.PROD": "true",
    "import.meta.env.MODE": '"production"',
    "import.meta.env": "{}",
    __APP_VERSION__: '"1.0.0"',
    __APP_COMMIT__: '"gpui"',
    __APP_BUILT_AT__: '"2026-10-14T00:00:00.000Z"',
  },
  loader: { ".css": "empty" },
  // Node built-ins some libraries import dynamically on a branch the host never takes.
  external: ["fs", "fs/promises", "module", "path", "url", "os", "crypto", "child_process", "worker_threads"],
  plugins: [assets],
});
