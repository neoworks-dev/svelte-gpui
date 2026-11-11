import { cp } from "node:fs/promises";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { build, buildWeb } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));
const watch = process.argv.includes("--watch");

// `--web` builds the same app for a browser instead of the native window: bun build.mjs --web, then
// svelte-gpui --web dist/web, which prints the address to open.
if (process.argv.includes("--web")) {
  const outdir = here("./dist/web");
  await buildWeb({ entry: here("./src/main.ts"), outdir, title: "svelte-gpui demo", watch });
  // Images the app shows by path are resolved next to the page.
  await cp(here("./assets"), here("./dist/web/assets"), { recursive: true });
  console.log(`built for the browser; serve it with: svelte-gpui --web ${outdir}`);
  if (!watch) {
    process.exit(0);
  }
  await new Promise(() => {});
}

const outfile = here("./dist/demo.js");

// Builds first and then watches, so the bundle exists when the host starts.
await build({ entry: here("./src/main.ts"), outfile, watch });

// Images the app shows by path; the host resolves them relative to the bundle.
await cp(here("./assets"), here("./dist/assets"), { recursive: true });

if (watch) {
  // With `--host PATH`, the host runs alongside and reloads the window on every build.
  const flag = process.argv.indexOf("--host");
  const host = flag === -1 ? undefined : process.argv[flag + 1];
  if (host === undefined) {
    console.log(`watching; run the host with: svelte-gpui --bundle ${outfile} --watch`);
  } else {
    const child = spawn(host, ["--bundle", outfile, "--watch"], { stdio: "inherit" });
    child.on("exit", () => process.exit(0));
    process.on("SIGINT", () => child.kill());
  }
}
