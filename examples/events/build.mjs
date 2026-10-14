import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({ entry: here("./src/main.ts"), outfile: here("./dist/events.js") });
execFileSync(
  here("../../node_modules/.bin/tailwindcss"),
  ["-i", here("./app.css"), "-o", here("./dist/events.css")],
  { stdio: "inherit" },
);
