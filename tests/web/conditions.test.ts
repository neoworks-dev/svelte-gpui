import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { mkdtempSync, mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import * as esbuild from "esbuild";

// What the package's `exports` and `imports` resolve to under each condition: the DOM versions for a
// bundler that asks for `browser`, the native ones for any other. The components are stubbed, since only
// the choice of files is being tested, not what they compile to.

const repository = new URL("../..", import.meta.url).pathname.replace(/\/$/, "");
let project: string;

/** The files of the repository that a bundle of `entry` pulls in, relative to it. */
async function bundled(entry: string, conditions: string[]): Promise<string[]> {
  const result = await esbuild.build({
    entryPoints: [entry],
    bundle: true,
    write: false,
    metafile: true,
    platform: "neutral",
    conditions,
    logLevel: "silent",
    plugins: [
      {
        name: "stub-components",
        setup(build) {
          build.onLoad({ filter: /\.svelte$/ }, () => ({ contents: "export default {};", loader: "js" }));
        },
      },
    ],
  });
  return Object.keys(result.metafile.inputs)
    .map((file) => (file.startsWith("/") ? file.slice(repository.length + 1) : file))
    .map((file) => file.replace(/^.*?(?=runtime\/|tests\/)/, ""));
}

beforeAll(() => {
  // A project that has the package the way an app would: in its node_modules.
  project = mkdtempSync(join(tmpdir(), "svelte-gpui-conditions-"));
  mkdirSync(join(project, "node_modules", "@neoworks"), { recursive: true });
  symlinkSync(repository, join(project, "node_modules", "@neoworks", "svelte-gpui"));
  writeFileSync(
    join(project, "entry.ts"),
    `import * as ui from "@neoworks/svelte-gpui/ui";\nimport * as root from "@neoworks/svelte-gpui";\nconsole.log(ui, root);\n`,
  );
});

afterAll(() => rmSync(project, { recursive: true, force: true }));

describe("the browser export condition", () => {
  test("resolves the package and its ui to the DOM versions", async () => {
    const files = await bundled(join(project, "entry.ts"), ["browser"]);
    expect(files).toContain("runtime/web/index.ts");
    expect(files).toContain("runtime/web/ui/index.ts");
    expect(files).not.toContain("runtime/index.ts");
    expect(files).not.toContain("runtime/ui/index.ts");
  });

  test("leaves the native versions as the default", async () => {
    const files = await bundled(join(project, "entry.ts"), []);
    expect(files).toContain("runtime/index.ts");
    expect(files).toContain("runtime/ui/index.ts");
    expect(files).not.toContain("runtime/web/index.ts");
    expect(files).not.toContain("runtime/web/ui/index.ts");
  });

  test("swaps the primitives the composed components use", async () => {
    const seam = new URL("./fixtures/seam.ts", import.meta.url).pathname;
    expect(await bundled(seam, ["browser"])).toContain("runtime/web/ui/primitives.ts");
    expect(await bundled(seam, ["browser"])).not.toContain("runtime/ui/primitives.ts");
    expect(await bundled(seam, [])).toContain("runtime/ui/primitives.ts");
    expect(await bundled(seam, [])).not.toContain("runtime/web/ui/primitives.ts");
  });
});
