import { unreleasedHasSharedLine } from "./changelog";

const BASE = "origin/main";
const CHANGELOG = "CHANGELOG.md";
const DEVLOG = "docs/devlog.md";
const FAILURE = 1;

function git(...args: string[]): string {
  const result = Bun.spawnSync(["git", ...args], { stderr: "ignore" });

  return result.stdout.toString().trim();
}

function changelogCovered(changed: string[]): boolean {
  if (changed.includes(CHANGELOG)) {
    return true;
  }

  return unreleasedHasSharedLine(git("show", `${BASE}:${CHANGELOG}`));
}

git("fetch", "--quiet", "origin", "main");

const changed = git("diff", "--name-only", `${BASE}...HEAD`).split("\n").filter(Boolean);
const missing = [];

if (!changelogCovered(changed)) {
  missing.push(CHANGELOG);
}

if (!changed.includes(DEVLOG)) {
  missing.push(DEVLOG);
}

if (changed.length > 0 && missing.length > 0) {
  process.stderr.write(
    [
      `This branch changes files but not ${missing.join(" or ")}.`,
      "Add an entry to both before pushing:",
      "- CHANGELOG.md: what users notice. Performance or internal work is one line",
      '  ("Performance improvements.", "Internal improvements."), left out when',
      "  Unreleased already has it.",
      "- docs/devlog.md: the technical details.",
      "",
    ].join("\n"),
  );
  process.exit(FAILURE);
}
