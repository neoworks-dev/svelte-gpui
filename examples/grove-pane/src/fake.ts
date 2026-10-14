// A stand-in for what the main process offers `window.workbench`. Anything the
// pane asks for that is not written out below answers with an empty promise, and
// subscriptions hand back an unsubscribe function, so the pane can be run before
// the real bridge exists.

import fixtures from "./fixtures.json";

const globals = globalThis as any;

const storage = new Map<string, string>();
globals.localStorage = {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => void storage.set(key, String(value)),
  removeItem: (key: string) => void storage.delete(key),
};

const repository = {
  nameWithOwner: fixtures.repo,
  url: `https://github.com/${fixtures.repo}`,
};

/** The calls the GitHub pane makes on load, answered from what `gh` read. */
const github: Record<string, (...args: any[]) => unknown> = {
  status: () => ({ installed: true, authenticated: true, repo: repository, error: null }),
  dashboard: ({ state }: { state: string }) => {
    const wanted = (item: { state: string }): boolean => {
      if (state === "open") return item.state === "OPEN";
      if (state === "closed") return item.state === "CLOSED" || item.state === "MERGED";
      if (state === "merged") return item.state === "MERGED";
      return true;
    };
    return {
      repo: repository,
      viewer: fixtures.viewer,
      issues: fixtures.issues.filter(wanted),
      pulls: fixtures.pulls.filter(wanted),
      capabilities: {
        projects: false,
        issueTypes: false,
        subIssues: false,
        linkedBranches: false,
        notifications: false,
        issueFields: false,
      },
      issueFields: [],
      issueDateFields: [],
      fetchedAt: Date.now(),
    };
  },
  item: (kind: string, number: number) => {
    const known = (fixtures.details as Record<string, unknown>)[`${kind}:${number}`];
    if (known !== undefined) {
      return known;
    }
    const list = kind === "pull" ? fixtures.pulls : fixtures.issues;
    const item = list.find((candidate) => candidate.number === number);
    return { ...item, id: `fake-${kind}-${number}`, body: "", authorActor: { login: item?.author, avatarUrl: null }, authorAssociation: "MEMBER", locked: false, timeline: [] };
  },
  labels: () => fixtures.labels,
  milestones: () => [],
  mentionables: () => [],
};

function namespace(name: string): any {
  return new Proxy(
    {},
    {
      get(_target, method: string) {
        if (method.startsWith("on")) {
          return () => () => {};
        }
        if (name === "github" && github[method] !== undefined) {
          return async (...args: unknown[]) => github[method](...args);
        }
        return async (...args: unknown[]) => {
          console.log(`workbench.${name}.${method}(${args.length} args)`);
          return undefined;
        };
      },
    },
  );
}

globals.workbench = new Proxy(
  {},
  {
    get(_target, name: string) {
      if (name.startsWith("on")) {
        return () => () => {};
      }
      return namespace(name);
    },
  },
);
