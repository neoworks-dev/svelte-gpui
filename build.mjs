// Bundles a svelte-gpui app into a single script QuickJS can run. Everything a
// browser build would take for granted — the DOM, `esm-env`'s conditions, a
// TypeScript-aware Svelte preprocessor — is wired up here explicitly.
// Adapted from svelte-gtk4's build.

import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { basename, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";
import { compile, compileModule, preprocess } from "svelte/compiler";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

const sveltePlugin = {
  name: "svelte",
  setup(build) {
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
      const { js, warnings } = compiled;
      for (const warning of warnings) {
        // Accessibility advice is about a browser's assistive tools, which this is not.
        if (warning.code.startsWith("a11y_")) {
          continue;
        }
        console.warn(`${args.path}: ${warning.message}`);
      }
      return { contents: await withSourceMap(js, source), loader: "js" };
    }));
  },
};

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
      build.onResolve({ filter: /^svelte($|\/)/ }, (args) => {
        if (args.pluginData === "deduped") {
          return null;
        }
        return build.resolve(args.path, {
          kind: args.kind,
          resolveDir: root,
          pluginData: "deduped",
        });
      });
    },
  };
}

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
      "esm-env": here("./runtime/esm-env.ts"),
      ...options.alias,
    },
    define: options.define,
    external: options.external,
    loader: { ".svg": "text", ...options.loader },
    plugins: [
      dedupeSvelte(dirname(options.entry)),
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
