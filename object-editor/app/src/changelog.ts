// Reads CHANGELOG.md (bundled at build time) for the "What's new" dialog. It
// understands only what the changelog uses: `## version`, `### group`, `- item`
// (wrapped onto indented lines), inline `code`, **bold** and [links](url).

export type ChangelogGroup = {
  title: string;
  items: string[];
};

export type ChangelogRelease = {
  /** "Unreleased" or a version such as "0.2.0 - 2026-11-01". */
  title: string;
  groups: ChangelogGroup[];
};

/** A run of item text; `code` runs show in monospace. */
export type InlineRun = {
  text: string;
  code: boolean;
};

const RELEASE = "## ";
const GROUP = "### ";
const ITEM = "- ";

/** Releases in file order (newest first); text before the first release is skipped. */
export function parseChangelog(text: string): ChangelogRelease[] {
  const releases: ChangelogRelease[] = [];

  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trimEnd();
    const release = releases.at(-1);
    const group = release?.groups.at(-1);

    if (line.startsWith(RELEASE)) {
      releases.push({ title: line.slice(RELEASE.length).trim(), groups: [] });
    } else if (line.startsWith(GROUP) && release) {
      release.groups.push({ title: line.slice(GROUP.length).trim(), items: [] });
    } else if (line.startsWith(ITEM) && group) {
      group.items.push(line.slice(ITEM.length).trim());
    } else if (line.startsWith(" ") && line.trim() && group && group.items.length > 0) {
      // A wrapped item continues on an indented line.
      const last = group.items.length - 1;
      group.items[last] = `${group.items[last]} ${line.trim()}`;
    }
  }

  return releases;
}

/** Splits item text into plain and `code` runs; bold marks and link targets are dropped. */
export function parseInline(text: string): InlineRun[] {
  const plain = text.replace(/\*\*(.+?)\*\*/g, "$1").replace(/\[(.+?)\]\([^)]*\)/g, "$1");
  return plain
    .split("`")
    .map((part, i) => ({ text: part, code: i % 2 === 1 }))
    .filter((run) => run.text !== "");
}
