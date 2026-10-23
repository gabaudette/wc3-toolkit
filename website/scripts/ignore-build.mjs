import { spawnSync } from "node:child_process";

// Vercel's convention is 0 = skip, 1 = build. No dependencies are installed yet.
const SKIP = 0;
const BUILD = 1;
const previous = process.env.VERCEL_GIT_PREVIOUS_SHA;
const current = process.env.VERCEL_GIT_COMMIT_SHA || "HEAD";
const sha = /^[a-f0-9]{7,64}$/i;

function finish(status, reason) {
  process.stdout.write(`${reason}\n`);
  process.exit(status);
}

if (process.env.DOCS_FORCE_BUILD === "1") {
  finish(BUILD, "Documentation build explicitly requested.");
}

if (!previous || !sha.test(previous) || (current !== "HEAD" && !sha.test(current))) {
  finish(BUILD, "No previous deployment to compare; building documentation.");
}

const diff = spawnSync(
  "git",
  [
    "diff",
    "--quiet",
    previous,
    current,
    "--",
    "website/",
    "object-editor/crates/wc3-data/data/",
    "vercel.json",
    ".vercelignore",
  ],
  { encoding: "utf8" },
);

if (diff.status === SKIP) {
  finish(SKIP, "No documentation inputs changed since the last deployment; skipping build.");
}

if (diff.status === BUILD) {
  finish(BUILD, "Documentation inputs changed; building.");
}

// Shallow clones may not contain the previous deployed commit. Never skip on uncertainty.
finish(BUILD, "Could not compare Git history; building documentation.");
