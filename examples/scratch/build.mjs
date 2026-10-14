import { fileURLToPath } from "node:url";
import { build } from "../../build.mjs";
const here = (p) => fileURLToPath(new URL(p, import.meta.url));
await build({ entry: here("./src/main.ts"), outfile: here("./dist/scratch.js"), alias: { "@grove": "/home/moritz/Documents/neoworks/grove" }, define: {"import.meta.env.DEV":"false","import.meta.env":"{}"} });
