// Pre-push check (lefthook.yml): a branch must update both the public
// changelog and the devlog compared with main. Skip it with `git push --no-verify`.

const BASE = "origin/main";
const LOGS = ["CHANGELOG.md", "docs/devlog.md"];
const FAILURE = 1;

function git(...args: string[]): string {
  const result = Bun.spawnSync(["git", ...args], { stderr: "ignore" });

  return result.stdout.toString().trim();
}

// Compare with the latest main when the network allows; the last fetch otherwise.
git("fetch", "--quiet", "origin", "main");

const changed = git("diff", "--name-only", `${BASE}...HEAD`).split("\n").filter(Boolean);
const missing = LOGS.filter((log) => !changed.includes(log));

if (changed.length > 0 && missing.length > 0) {
  process.stderr.write(
    [
      `This branch changes files but not ${missing.join(" or ")}.`,
      "Add an entry to both before pushing:",
      "- CHANGELOG.md: what users notice. Performance or internal work is one line",
      '  ("Performance improvements.", "Internal improvements.").',
      "- docs/devlog.md: the technical details.",
      "",
    ].join("\n"),
  );
  process.exit(FAILURE);
}
