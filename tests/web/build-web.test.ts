import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { cpSync, existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { buildWeb } from "../../build.mjs";

// An app built for the browser by a consumer outside this repository: the fixture is copied to a
// temporary directory, where it has no node_modules, no copy of this package and no `svelte` of its own.

let root: string;
let app: string;
let outdir: string;
let built: Awaited<ReturnType<typeof buildWeb>>;

/** Another copy of the fixture app, to change. */
function copyOfApp(name: string): string {
  const copy = join(root, name);
  cpSync(new URL("./fixtures/app", import.meta.url).pathname, copy, { recursive: true });
  return copy;
}

beforeAll(async () => {
  root = mkdtempSync(join(tmpdir(), "svelte-gpui-app-"));
  app = copyOfApp("app");
  outdir = join(app, "out");
  built = await buildWeb({ entry: join(app, "main.ts"), outdir, title: "Fixture <app>", logLevel: "silent" });
});

afterAll(() => rmSync(root, { recursive: true, force: true }));

describe("buildWeb", () => {
  test("writes the page, the script and the stylesheet, and the fonts the page names", () => {
    expect(built).toMatchObject({ outdir, html: join(outdir, "index.html"), js: join(outdir, "app.js"), css: join(outdir, "app.css") });
    expect(existsSync(join(outdir, "index.html"))).toBe(true);
    expect(existsSync(join(outdir, "app.js"))).toBe(true);
    expect(existsSync(join(outdir, "app.css"))).toBe(true);
    expect(existsSync(join(outdir, "fonts", "Geist-400.ttf"))).toBe(true);
    expect(existsSync(join(outdir, "fonts", "JetBrainsMono-Regular.ttf"))).toBe(true);
  });

  test("the page loads the script as a module and the stylesheet, under an escaped title", () => {
    const html = readFileSync(join(outdir, "index.html"), "utf8");
    expect(html).toContain('<script type="module" src="app.js"></script>');
    expect(html).toContain('<link rel="stylesheet" href="app.css" />');
    expect(html).toContain("<title>Fixture &lt;app></title>");
  });

  test("a component's style block reaches the stylesheet", () => {
    expect(readFileSync(join(outdir, "app.css"), "utf8")).toContain("margin:0");
  });

  test("the script holds the DOM primitives and none of the native ones", () => {
    const script = readFileSync(join(outdir, "app.js"), "utf8");
    expect(script).toContain("g-view");
    expect(script).toContain("g-scroll");
    // The native primitives are host tags, which would be written as elements of their own.
    expect(script).not.toContain("<g-view");
    expect(script).not.toContain("<g-text");
    // The host connection, the browser's `start` and the composed List are all in it.
    expect(script).toContain("svelte-gpui:token");
    expect(script).toContain("Running on");
  });

  test("a build with no stylesheet links none", async () => {
    const plain = copyOfApp("plain");
    const entry = join(plain, "plain.ts");
    await Bun.write(entry, `import { start } from "@neoworks/svelte-gpui";\nstart(class {} as never);\n`);
    const result = await buildWeb({ entry, outdir: join(plain, "dist"), logLevel: "silent" });
    expect(result.css).toBeNull();
    expect(readFileSync(result.html, "utf8")).not.toContain("stylesheet");
  });

  test("an alias the app gives is used", async () => {
    const aliased = copyOfApp("aliased");
    await Bun.write(join(aliased, "answer.ts"), `export const answer = "forty-two";\n`);
    const entry = join(aliased, "aliased.ts");
    await Bun.write(entry, `import { answer } from "the-answer";\nconsole.log(answer);\n`);
    const result = await buildWeb({
      entry,
      outdir: join(aliased, "dist"),
      alias: { "the-answer": join(aliased, "answer.ts") },
      logLevel: "silent",
    });
    expect(readFileSync(result.js, "utf8")).toContain("forty-two");
  });

  test("a component that does not compile fails the build", async () => {
    const broken = copyOfApp("broken");
    await Bun.write(join(broken, "Broken.svelte"), `<View>{#if}</View>\n`);
    const entry = join(broken, "broken.ts");
    await Bun.write(entry, `import Broken from "./Broken.svelte";\nconsole.log(Broken);\n`);
    await expect(buildWeb({ entry, outdir: join(broken, "dist"), logLevel: "silent" })).rejects.toThrow();
  });
});
