import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));

await build({
  entry: here("./src/main.ts"),
  outfile: here("./dist/hello.js"),
});

// The real Tailwind compiler, over the same tokens Grove declares; the host
// reads the stylesheet it writes and resolves each element's classes against it.
execFileSync(
  here("../../node_modules/.bin/tailwindcss"),
  ["-i", here("./app.css"), "-o", here("./dist/hello.css")],
  { stdio: "inherit" },
);
