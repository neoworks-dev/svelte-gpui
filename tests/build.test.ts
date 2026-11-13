import { describe, expect, test } from "bun:test";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { build } from "../build.mjs";

describe("the build", () => {
  test("patches Svelte's block manager, so a block does not hold itself in a cycle only a heap scan frees", async () => {
    const outfile = join(mkdtempSync(join(tmpdir(), "svelte-gpui-build-")), "app.js");
    await build({ entry: new URL("./fixtures/primitives-app.ts", import.meta.url).pathname, outfile, sourcemap: false });
    const bundle = readFileSync(outfile, "utf8");
    expect(bundle).toContain("#commit(batch) {");
    expect(bundle).toContain("#discard(batch) {");
    expect(bundle).not.toContain("#commit = (batch) =>");
    expect(bundle).not.toContain("#discard = (batch) =>");
  });
});
