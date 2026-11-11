// Bundles a svelte-gpui app into a single script QuickJS can run. Everything a
// browser build would take for granted — the DOM, `esm-env`'s conditions, a
// TypeScript-aware Svelte preprocessor — is wired up here explicitly.
// Adapted from svelte-gtk4's build.

import { cp, mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";
import { compile, compileModule, preprocess } from "svelte/compiler";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

// `emitCss` is for a browser build, where a component's `<style>` block becomes a stylesheet;
// the host has no CSS, so the native build leaves them out.
const makeSveltePlugin = ({ emitCss }) => ({
  name: "svelte",
  setup(build) {
    const stylesheets = new Map();
    build.onResolve({ filter: /\.svelte\.css$/ }, (args) => ({ path: args.path, namespace: "svelte-css" }));
    build.onLoad({ filter: /.*/, namespace: "svelte-css" }, (args) => ({
      contents: stylesheets.get(args.path) ?? "",
      loader: "css",
    }));

    // A `.svelte.ts` module is ordinary TypeScript that may use runes, so it is
    // stripped of its types and then run through the compiler's module half.
    build.onLoad({ filter: /\.svelte\.[jt]s$/ }, reportingErrors(async (args) => {
      const source = await readFile(args.path, "utf8");
      const { code, map } = await transpileScript({
        content: source,
        attributes: { lang: "ts" },
        filename: args.path,
      });
      const { js } = compileModule(code, {
        filename: args.path,
        generate: "client",
        dev: false,
      });
      return { contents: await withSourceMap(js, source, map, args.path), loader: "js" };
    }));

    build.onLoad({ filter: /\.svelte$/ }, reportingErrors(async (args) => {
      const source = await readFile(args.path, "utf8");
      const processed = await preprocess(
        source,
        { script: transpileScript },
        { filename: args.path },
      );
      let compiled;
      try {
        compiled = compile(processed.code, {
          filename: args.path,
          generate: "client",
          runes: true,
          dev: false,
          sourcemap: processed.map,
        });
      } catch (error) {
        throw await inOriginalSource(error, processed.map);
      }
      const { js, css, warnings } = compiled;
      for (const warning of warnings) {
        // Accessibility advice is about a browser's assistive tools, which this is not.
        if (warning.code.startsWith("a11y_")) {
          continue;
        }
        console.warn(`${args.path}: ${warning.message}`);
      }
      let contents = await withSourceMap(js, source);
      if (emitCss && css?.code) {
        stylesheets.set(`${args.path}.css`, css.code);
        contents += `\nimport ${JSON.stringify(`${args.path}.css`)};`;
      }
      return { contents, loader: "js" };
    }));
  },
});

const sveltePlugin = makeSveltePlugin({ emitCss: false });

// The compiler reports positions in the code it was given, which is the file with its types
// stripped; this moves one back to the file itself, when the map to do that can be read.
async function inOriginalSource(error, map) {
  if (!error?.start || !map) {
    return error;
  }
  try {
    const { TraceMap, originalPositionFor } = await import("@jridgewell/trace-mapping");
    const found = originalPositionFor(new TraceMap(map), { line: error.start.line, column: error.start.column });
    if (found.line !== null) {
      error.start = { line: found.line, column: found.column };
    }
  } catch {
    // Without the package, the position is in the code the compiler saw.
  }
  return error;
}

// A file that does not compile is reported against the file, at the position the compiler
// gave, instead of as an exception inside this plugin.
function reportingErrors(load) {
  return async (args) => {
    try {
      return await load(args);
    } catch (error) {
      if (Array.isArray(error?.errors)) {
        // esbuild's own messages, such as a syntax error in the script.
        return { errors: error.errors };
      }
      const source = await readFile(args.path, "utf8").catch(() => "");
      const start = error?.start;
      const location = start
        ? { file: args.path, line: start.line, column: start.column, lineText: source.split("\n")[start.line - 1] ?? "" }
        : { file: args.path };
      return { errors: [{ text: error?.message ?? String(error), location }] };
    }
  };
}

// The compiled code with a map back to the file it came from, as a comment esbuild reads
// and folds into the bundle's map. `source` is the file's text. `earlier` is the map of a step
// before the compiler (the types being stripped), which the compiler's map is chained onto
// when `remapping` is there.
async function withSourceMap(js, source, earlier, path) {
  if (!js.map) {
    return js.code;
  }
  let map = js.map;
  if (earlier) {
    try {
      const { default: remapping } = await import("@jridgewell/remapping");
      map = remapping([js.map, earlier], () => null);
      map.sources = [basename(path)];
    } catch {
      // Without the package, the map names the code before the types were stripped.
    }
  }
  // The map holds the text that went into the last step, which is not what the file says.
  if (map.sources?.length === 1) {
    map.sourcesContent = [source];
  }
  const text = typeof map.toUrl === "function" ? map.toUrl() : `data:application/json;base64,${Buffer.from(JSON.stringify(map)).toString("base64")}`;
  return `${js.code}\n//# sourceMappingURL=${text}`;
}

// The Svelte compiler parses markup, not TypeScript, so `lang="ts"` blocks are
// stripped of their types first.
async function transpileScript({ content, attributes, filename }) {
  if (attributes.lang !== "ts") {
    return { code: content };
  }
  const { code, map } = await esbuild.transform(content, {
    loader: "ts",
    target: "esnext",
    sourcemap: true,
    sourcefile: filename === undefined ? undefined : basename(filename),
    tsconfigRaw: { compilerOptions: { verbatimModuleSyntax: true } },
  });
  return { code, map };
}

// The library and the app each resolve `svelte` from their own node_modules,
// and two copies of the runtime means two `active_effect` variables. Every
// svelte import is resolved from one directory instead.
function dedupeSvelte(root) {
  return {
    name: "dedupe-svelte",
    setup(build) {
      build.onResolve({ filter: /^svelte($|\/)/ }, async (args) => {
        if (args.pluginData === "deduped") {
          return null;
        }
        const options = { kind: args.kind, pluginData: "deduped" };
        const found = await build.resolve(args.path, { ...options, resolveDir: root });
        if (found.errors.length === 0) {
          return found;
        }
        // An app that has no `svelte` of its own runs on the one this library was installed with.
        return build.resolve(args.path, { ...options, resolveDir: here("./") });
      });
    },
  };
}

// The composed components import their primitives as `#primitives`, which package.json points at
// the DOM versions under the `browser` condition. This build asks for that condition too, because
// Svelte's own client runtime needs it, so the native primitives are named here.
const nativeSeam = {
  name: "native-primitives",
  setup(build) {
    build.onResolve({ filter: /^#primitives$/ }, () => ({ path: here("./runtime/ui/primitives.ts") }));
  },
};

// Writes each output to a temporary file and renames it into place, so a host that watches
// the bundle never reads one half written. When the build has errors, the bundle is left as
// it was, for the host to keep running, and the errors go to `OUTFILE.error` instead; the
// next good build removes that file.
function writeAtomically(options) {
  return {
    name: "write-atomically",
    setup(build) {
      build.initialOptions.write = false;
      build.onEnd(async (result) => {
        const errorFile = `${options.outfile}.error`;
        if (result.errors.length > 0) {
          // A host watching the bundle shows this in its window, since nobody reads the terminal.
          const messages = await esbuild.formatMessages(result.errors, { kind: "error", terminalWidth: 100 });
          await mkdir(dirname(errorFile), { recursive: true });
          await writeFile(`${errorFile}.${process.pid}.tmp`, messages.join("\n"));
          await rename(`${errorFile}.${process.pid}.tmp`, errorFile);
          return;
        }
        await rm(errorFile, { force: true });
        // The map goes first: the host reloads when the bundle changes and reads the map
        // when it needs it, so the bundle must not appear before its map.
        const files = [...(result.outputFiles ?? [])].sort((a, b) => Number(b.path.endsWith(".map")) - Number(a.path.endsWith(".map")));
        for (const file of files) {
          await mkdir(dirname(file.path), { recursive: true });
          const temporary = `${file.path}.${process.pid}.tmp`;
          await writeFile(temporary, file.contents);
          await rename(temporary, file.path);
        }
        if (options.watch) {
          console.log(`[watch] wrote ${options.outfile} at ${new Date().toISOString()}`);
        }
      });
    },
  };
}

// Builds the app. With `watch: true` it then rebuilds whenever a source file changes, until
// the process ends, and resolves to a function that stops watching.
export async function build(options) {
  const config = {
    metafile: Boolean(process.env.ANALYZE),
    // A map beside the bundle lets the host show an error's stack in the app's own files.
    sourcemap: options.sourcemap ?? (options.watch ? "external" : false),
    sourcesContent: true,
    entryPoints: [options.entry],
    outfile: options.outfile,
    bundle: true,
    // QuickJS runs a script, not a module, and the host evaluates it directly.
    format: "iife",
    platform: "neutral",
    target: "es2022",
    conditions: ["svelte", "browser", "production", "default"],
    mainFields: ["browser", "module", "main"],
    alias: {
      "@neoworks/svelte-gpui": here("./runtime"),
      "@neoworks/svelte-gpui-ui": here("./packages/svelte-gpui-ui/src"),
      "@neoworks/svelte-gpui-editor": here("./packages/svelte-gpui-editor/src"),
      "esm-env": here("./runtime/esm-env.ts"),
      ...options.alias,
    },
    define: options.define,
    external: options.external,
    loader: { ".svg": "text", ...options.loader },
    plugins: [
      dedupeSvelte(dirname(options.entry)),
      nativeSeam,
      sveltePlugin,
      ...(options.plugins ?? []),
      writeAtomically(options),
    ],
    logLevel: "info",
  };
  if (options.watch) {
    // `watch()` starts with a build; wait for it, so the bundle exists when this resolves.
    let firstBuild;
    const built = new Promise((resolve) => (firstBuild = resolve));
    config.plugins.push({ name: "first-build", setup: (build) => build.onEnd(() => firstBuild()) });
    const context = await esbuild.context(config);
    await context.watch();
    await built;
    return () => context.dispose();
  }
  const result = await esbuild.build(config);
  if (result.metafile) {
    await writeFile(`${options.outfile}.meta.json`, JSON.stringify(result.metafile));
  }
}

// The browser target resolves `@neoworks/svelte-gpui` and `@neoworks/svelte-gpui/ui` to the DOM
// versions, as the `browser` export condition in package.json does for any other bundler. Naming
// them here as well keeps a build working for an app that has no copy of this package in its own
// node_modules, which is how a consumer outside this repository builds.
function browserPackage(alias) {
  const files = {
    "@neoworks/svelte-gpui": here("./runtime/web/index.ts"),
    "@neoworks/svelte-gpui/ui": here("./runtime/web/ui/index.ts"),
  };
  return {
    name: "browser-package",
    setup(build) {
      build.onResolve({ filter: /^@neoworks\/svelte-gpui(\/ui)?$/ }, (args) => {
        // An alias the app gave wins over this default.
        return alias[args.path] === undefined ? { path: files[args.path] } : null;
      });
    },
  };
}

function pageFor({ title, js, css }) {
  const escaped = String(title).replace(/&/g, "&amp;").replace(/</g, "&lt;");
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${escaped}</title>${css ? `\n    <link rel="stylesheet" href="${css}" />` : ""}
    <script type="module" src="${js}"></script>
  </head>
  <body></body>
</html>
`;
}

/**
 * Bundles an app for the browser: the same entry that `build` bundles for the native host, with the
 * DOM primitives under it. Writes `index.html`, `app.js` and, when a component has a `<style>` block
 * or the app imports CSS, `app.css` into `outdir`, which the host serves with `svelte-gpui --web DIR`.
 *
 * Options: `entry` (an absolute path) and `outdir`; `alias`, `define`, `loader` and `plugins`, which are
 * esbuild's; `title`, for the page until the app sets its own with `start(…, { window: { title } })`;
 * `sourcemap` (`"linked"` when watching) and `minify` (on unless watching); `logLevel`, which is esbuild's;
 * and `watch: true` to rebuild whenever a source file changes.
 *
 * Resolves to `{ outdir, html, js, css, stop }`. `css` is `null` without a stylesheet, and `stop` ends the
 * watching, when there is any.
 */
export async function buildWeb(options) {
  const entry = resolve(options.entry);
  const outdir = resolve(options.outdir);
  const watch = options.watch === true;
  const alias = {
    "@neoworks/svelte-gpui-ui": here("./packages/svelte-gpui-ui/src"),
    "@neoworks/svelte-gpui-editor": here("./packages/svelte-gpui-editor/src"),
    ...options.alias,
  };
  const title = options.title ?? "svelte-gpui app";
  const written = { outdir, html: join(outdir, "index.html"), js: join(outdir, "app.js"), css: null, stop: undefined };

  const writePage = {
    name: "write-page",
    setup(build) {
      build.onEnd(async (result) => {
        if (result.errors.length > 0) {
          return;
        }
        const css = Object.keys(result.metafile?.outputs ?? {}).some((file) => file.endsWith(".css"));
        written.css = css ? join(outdir, "app.css") : null;
        await mkdir(outdir, { recursive: true });
        await writeFile(written.html, pageFor({ title, js: "app.js", css: css ? "app.css" : null }));
        // The fonts the host draws with, which the page asks for by name (see runtime/web/ui/base.ts).
        await cp(here("./host/assets/fonts"), join(outdir, "fonts"), { recursive: true, force: false, errorOnExist: false });
        if (watch) {
          console.log(`[watch] wrote ${outdir} at ${new Date().toISOString()}`);
        }
      });
    },
  };

  const config = {
    entryPoints: { app: entry },
    outdir,
    entryNames: "[name]",
    assetNames: "assets/[name]-[hash]",
    bundle: true,
    metafile: true,
    format: "esm",
    platform: "browser",
    target: "es2022",
    conditions: ["svelte", "browser", "production", "default"],
    mainFields: ["svelte", "browser", "module", "main"],
    alias,
    define: options.define,
    sourcemap: options.sourcemap ?? (watch ? "linked" : false),
    sourcesContent: true,
    minify: options.minify ?? !watch,
    legalComments: "none",
    loader: {
      ".svg": "text",
      ".png": "file",
      ".jpg": "file",
      ".jpeg": "file",
      ".gif": "file",
      ".webp": "file",
      ".woff": "file",
      ".woff2": "file",
      ".ttf": "file",
      ...options.loader,
    },
    plugins: [
      dedupeSvelte(dirname(entry)),
      browserPackage(alias),
      makeSveltePlugin({ emitCss: true }),
      ...(options.plugins ?? []),
      writePage,
    ],
    logLevel: options.logLevel ?? "info",
  };

  if (watch) {
    // `watch()` starts with a build; wait for it, so the page exists when this resolves.
    let firstBuild;
    const built = new Promise((resolve) => (firstBuild = resolve));
    config.plugins.push({ name: "first-build", setup: (build) => build.onEnd(() => firstBuild()) });
    const context = await esbuild.context(config);
    await context.watch();
    await built;
    written.stop = () => context.dispose();
    return written;
  }
  const result = await esbuild.build(config);
  if (result.errors.length > 0) {
    throw new Error(`the browser build of ${entry} failed`);
  }
  return written;
}
