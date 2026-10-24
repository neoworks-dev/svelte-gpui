import { beforeAll, describe, expect, test } from "bun:test";

interface Built {
  rows: string[];
  above: number;
  below: number;
  anchor: string | null;
}

interface Run {
  opened: { chat: Built; plain: Built; fixed: Built; scrollRequests: number };
  measured: { chat: Built; scrollRequests: number };
  appended: { chat: Built; scrollRequests: number };
  scrolledUp: { chat: Built; scrollRequests: number };
  appendedWhileUp: { chat: Built; scrollRequests: number };
  backAtEnd: { chat: Built; scrollRequests: number };
  toEnd: { chat: Built; scrollRequests: number; offset: number };
}

let run: Run;

beforeAll(async () => {
  const driver = new URL("./fixtures/drive-list.ts", import.meta.url).pathname;
  const child = Bun.spawn(["bun", driver], { stdout: "pipe", stderr: "pipe" });
  const [output, errors] = await Promise.all([new Response(child.stdout).text(), new Response(child.stderr).text()]);
  expect(await child.exited, errors).toBe(0);
  run = JSON.parse(output.slice(output.lastIndexOf("@@") + 2));
});

describe("List with stickToEnd", () => {
  test("anchors its scroller to the end, so the host keeps the end in view", () => {
    expect(run.opened.chat.anchor).toBe("end");
  });

  test("opens with the last rows built, not the first, and no others", () => {
    const { rows, above, below } = run.opened.chat;
    expect(rows[rows.length - 1]).toBe("chat 4999");
    expect(rows.length).toBeLessThan(60);
    expect(rows).not.toContain("chat 0");
    expect(above).toBeGreaterThan(0);
    expect(below).toBe(0);
  });

  test("never asks the host to scroll while rows are measured, added and the window moves", () => {
    expect(run.opened.scrollRequests).toBe(0);
    expect(run.measured.scrollRequests).toBe(0);
    expect(run.appended.scrollRequests).toBe(0);
    expect(run.scrolledUp.scrollRequests).toBe(0);
    expect(run.appendedWhileUp.scrollRequests).toBe(0);
    expect(run.backAtEnd.scrollRequests).toBe(0);
  });

  test("keeps building the end as rows are measured, so measuring does not leave it", () => {
    const { rows, below } = run.measured.chat;
    expect(rows[rows.length - 1]).toBe("chat 4999");
    expect(below).toBe(0);
    expect(rows.length).toBeLessThan(run.opened.chat.rows.length);
  });

  test("builds an item added while at the end", () => {
    const { rows } = run.appended.chat;
    expect(rows[rows.length - 1]).toBe("chat 5000");
  });

  test("builds only rows near the window once scrolled up, with the rest as spacers", () => {
    const { rows, above, below } = run.scrolledUp.chat;
    expect(rows.length).toBeLessThan(60);
    expect(rows).not.toContain("chat 5000");
    expect(above).toBeGreaterThan(0);
    expect(below).toBeGreaterThan(0);
  });

  test("builds nothing new for an item added while scrolled up, and leaves the window where it is", () => {
    expect(run.appendedWhileUp.chat.rows[0]).toBe(run.scrolledUp.chat.rows[0]);
    expect(run.appendedWhileUp.chat.rows).not.toContain("chat 5001");
  });

  test("takes the end again when the host reports it is back there", () => {
    const { rows, below } = run.backAtEnd.chat;
    expect(rows[rows.length - 1]).toBe("chat 5001");
    expect(below).toBe(0);
  });

  test("scrollToIndex to the last row asks the host for the end itself, however tall the rows turn out", () => {
    // One call to `scrollTo` is two operations, one for each axis.
    expect(run.toEnd.scrollRequests).toBe(2);
    expect(run.toEnd.offset).toBeGreaterThan(1e12);
    const { rows, below } = run.toEnd.chat;
    expect(rows[rows.length - 1]).toBe("chat 5001");
    expect(below).toBe(0);
  });
});

describe("List without stickToEnd", () => {
  test("keeps the content in view while rows above change height, but does not open at the end", () => {
    expect(run.opened.plain.anchor).toBe("content");
    expect(run.opened.plain.rows[0]).toBe("plain 0");
  });

  test("with fixed rows has nothing above that can change, so it anchors nothing", () => {
    expect(run.opened.fixed.anchor).toBeNull();
    expect(run.opened.fixed.rows[0]).toBe("fixed 0");
  });
});
