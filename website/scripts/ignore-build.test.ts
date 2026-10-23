import { afterEach, beforeEach, expect, test } from "bun:test";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const script = fileURLToPath(new URL("./ignore-build.mjs", import.meta.url));
let directory = "";
let baseline = "";

function git(...args: string[]) {
  return execFileSync("git", args, { cwd: directory, encoding: "utf8" }).trim();
}

function write(path: string, text: string) {
  const target = join(directory, path);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, text);
}

function commit() {
  git("add", ".");
  git("commit", "--quiet", "-m", "fixture");
  return git("rev-parse", "HEAD");
}

function decision(previous = baseline, force = "") {
  return spawnSync("node", [script], {
    cwd: directory,
    encoding: "utf8",
    env: {
      ...process.env,
      VERCEL_GIT_PREVIOUS_SHA: previous,
      VERCEL_GIT_COMMIT_SHA: git("rev-parse", "HEAD"),
      DOCS_FORCE_BUILD: force,
    },
  });
}

beforeEach(() => {
  directory = mkdtempSync(join(tmpdir(), "tinkerbench-ignore-"));
  git("init", "--quiet");
  git("config", "user.name", "Documentation test");
  git("config", "user.email", "docs@example.test");
  const hooks = join(directory, "empty-hooks");
  mkdirSync(hooks);
  git("config", "core.hooksPath", hooks);
  write("website/page.md", "Original guide");
  write("object-editor/app/main.ts", "original");
  baseline = commit();
});

afterEach(() => {
  rmSync(directory, { recursive: true, force: true });
});

test("skips editor-only changes", () => {
  write("object-editor/app/main.ts", "Editor change");
  commit();
  expect(decision().status).toBe(0);
});

test.each([
  "website/page.md",
  "object-editor/crates/wc3-data/data/unit-fields.toml",
  "vercel.json",
  ".vercelignore",
])("builds when %s changes", (path) => {
  write(path, "Changed documentation input");
  commit();
  expect(decision().status).toBe(1);
});

test("includes relevant changes earlier in a multi-commit push", () => {
  write("website/page.md", "Updated guide");
  commit();
  write("object-editor/app/main.ts", "Later unrelated edit");
  commit();
  expect(decision().status).toBe(1);
});

test("builds after a documentation file is deleted", () => {
  rmSync(join(directory, "website/page.md"));
  commit();
  expect(decision().status).toBe(1);
});

test("builds if history is missing or a manual build is requested", () => {
  expect(decision("").status).toBe(1);
  expect(decision("a".repeat(40)).status).toBe(1);
  expect(decision(baseline, "1").status).toBe(1);
});
