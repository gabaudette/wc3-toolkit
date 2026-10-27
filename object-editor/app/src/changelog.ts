export type ChangelogGroup = {
  title: string;
  items: string[];
};

export type ChangelogRelease = {
  title: string;
  groups: ChangelogGroup[];
};

export type InlineRun = {
  text: string;
  code: boolean;
};

const RELEASE = "## ";
const GROUP = "### ";
const ITEM = "- ";

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
      const last = group.items.length - 1;

      group.items[last] = `${group.items[last]} ${line.trim()}`;
    }
  }

  return releases;
}

export function parseInline(text: string): InlineRun[] {
  const plain = text.replace(/\*\*(.+?)\*\*/g, "$1").replace(/\[(.+?)\]\([^)]*\)/g, "$1");

  return plain
    .split("`")
    .map((part, i) => ({ text: part, code: i % 2 === 1 }))
    .filter((run) => run.text !== "");
}
