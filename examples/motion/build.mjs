import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({ entry: here("./src/main.ts"), outfile: here("./dist/motion.js") });
execFileSync(
  here("../../node_modules/.bin/tailwindcss"),
  ["-i", here("./app.css"), "-o", here("./dist/motion.css")],
  { stdio: "inherit" },
);
