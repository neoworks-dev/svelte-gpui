// Regenerates src/tokens.generated.ts from the design system's tokens.css:
//   bun scripts/generate.mjs PATH/TO/packages/ui/src/styles/tokens.css
import { readFileSync, writeFileSync } from "node:fs";
import { generateTokens } from "./tokens.ts";

const source = process.argv[2];
if (source === undefined) {
  console.error("usage: bun scripts/generate.mjs PATH/TO/tokens.css");
  process.exit(1);
}

const { module, gamutWarnings } = generateTokens(readFileSync(source, "utf8"));
const target = new URL("../src/tokens.generated.ts", import.meta.url);
writeFileSync(target, module);
for (const warning of gamutWarnings) {
  console.warn(warning);
}
console.log(`wrote ${target.pathname}`);
