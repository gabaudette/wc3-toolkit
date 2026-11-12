const UNRELEASED = "## Unreleased";
const SECTION = "## ";
const SHARED_LINES = ["- Internal improvements.", "- Performance improvements."];

function unreleasedLines(changelog: string): string[] {
  const lines = changelog.split(/\r?\n/).map((line) => line.trimEnd());
  const start = lines.indexOf(UNRELEASED);

  if (start === -1) {
    return [];
  }

  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => line.startsWith(SECTION));

  if (end === -1) {
    return rest;
  }

  return rest.slice(0, end);
}

export function unreleasedHasSharedLine(changelog: string): boolean {
  return unreleasedLines(changelog).some((line) => SHARED_LINES.includes(line));
}
