import { missingBlankLines } from "./blankLines";

const REPOSITORY = `${import.meta.dir}/../../..`;
const SOURCES = ["*.rs", "*.ts", "*.tsx"];
const CHECKED = /^object-editor\//;
const SKIPPED = /(^|\/)vendor\//;
const FAILURE = 1;

function sourceFiles(): string[] {
  const listed = Bun.spawnSync(["git", "ls-files", "--full-name", ...SOURCES], {
    cwd: REPOSITORY,
  });

  return listed.stdout
    .toString()
    .split("\n")
    .filter((path) => CHECKED.test(path) && !SKIPPED.test(path));
}

const fix = Bun.argv.includes("--fix");
let total = 0;

for (const path of sourceFiles()) {
  const file = Bun.file(`${REPOSITORY}/${path}`);
  const text = await file.text();
  const found = missingBlankLines(text, path);

  total += found.size;
  if (found.size === 0) {
    continue;
  }

  if (fix) {
    const lines = text.split("\n");

    for (const at of [...found.keys()].sort((a, b) => b - a)) {
      lines.splice(at, 0, "");
    }

    await Bun.write(file, lines.join("\n"));
    continue;
  }

  for (const [at, rule] of [...found].sort((a, b) => a[0] - b[0])) {
    process.stdout.write(`${path}:${at}: ${rule}\n`);
  }
}

if (fix) {
  process.stdout.write(`Added ${total} blank line(s).\n`);
} else if (total > 0) {
  process.stderr.write(
    `${total} missing blank line(s); \`bun scripts/check-blank-lines.ts --fix\` adds them.\n`,
  );
  process.exit(FAILURE);
}
