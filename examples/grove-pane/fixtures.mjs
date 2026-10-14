// Reads the real neoworks-dev/grove issues and pull requests with `gh` (reading only)
// and writes them in the shapes Grove's main process sends, for the fake bridge.

import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const repo = "neoworks-dev/grove";
const gh = (...args) => JSON.parse(execFileSync("gh", args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }));
const login = (author) => (author?.login ?? "ghost").replace("app/", "");
const actor = (author) => ({ login: login(author), avatarUrl: `https://github.com/${login(author)}.png` });

const shared = (item, kind) => ({
  kind,
  number: item.number,
  title: item.title,
  url: item.url,
  state: item.state,
  author: login(item.author),
  createdAt: item.createdAt,
  updatedAt: item.updatedAt,
  labels: item.labels.map((label) => ({ name: label.name, color: label.color })),
  assignees: item.assignees.map(actor),
  milestone: null,
  projects: [],
  commentCount: item.comments.length,
});

const fields = "number,title,url,state,author,createdAt,updatedAt,labels,assignees,comments";
const issues = gh("issue", "list", "-R", repo, "--state", "all", "--limit", "60", "--json", fields).map((item) => shared(item, "issue"));
const pulls = gh("pr", "list", "-R", repo, "--state", "all", "--limit", "20", "--json", `${fields},isDraft,reviewDecision,headRefName,baseRefName,additions,deletions`).map((item) => ({
  ...shared(item, "pull"),
  isDraft: item.isDraft,
  reviewDecision: item.reviewDecision || null,
  checks: null,
  headRefName: item.headRefName,
  baseRefName: item.baseRefName,
  additions: item.additions,
  deletions: item.deletions,
}));

const details = {};
for (const item of [...issues.slice(0, 10), ...pulls.slice(0, 6)]) {
  const command = item.kind === "pull" ? "pr" : "issue";
  const full = gh(command, "view", String(item.number), "-R", repo, "--json", "id,body,comments,author");
  details[`${item.kind}:${item.number}`] = {
    ...item,
    id: full.id,
    body: full.body ?? "",
    authorActor: actor(full.author),
    authorAssociation: "MEMBER",
    locked: false,
    timeline: full.comments.map((comment) => ({
      type: "comment",
      at: comment.createdAt,
      comment: {
        id: comment.id,
        author: actor(comment.author),
        body: comment.body,
        createdAt: comment.createdAt,
        url: comment.url,
        authorAssociation: comment.authorAssociation ?? "MEMBER",
      },
    })),
    ...(item.kind === "pull" ? { changedFiles: 0, mergeStateStatus: "CLEAN", maintainerCanModify: true, headRepository: null } : {}),
  };
}

const viewer = execFileSync("gh", ["api", "user", "-q", ".login"], { encoding: "utf8" }).trim();
const labels = new Map();
for (const item of [...issues, ...pulls]) {
  for (const label of item.labels) {
    labels.set(label.name, { ...label, description: "" });
  }
}
const here = (path) => fileURLToPath(new URL(path, import.meta.url));
writeFileSync(here("./src/fixtures.json"), JSON.stringify({ repo, viewer, issues, pulls, details, labels: [...labels.values()] }));
console.log(issues.length, "issues,", pulls.length, "pulls,", Object.keys(details).length, "threads");
