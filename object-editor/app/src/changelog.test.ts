import { describe, expect, test } from "bun:test";
import changelog from "../../../CHANGELOG.md?raw";
import { parseChangelog, parseInline } from "./changelog";

describe("parseChangelog", () => {
  test("reads releases, groups and wrapped items, skipping the intro", () => {
    const text = [
      "# Changelog",
      "",
      "Intro text.",
      "",
      "## Unreleased",
      "",
      "### Added",
      "",
      "- One item",
      "  that wraps.",
      "- Two",
      "",
      "### Fixed",
      "",
      "- Performance improvements.",
      "",
      "## 0.1.0 - 2026-10-23",
      "",
      "### Added",
      "",
      "- First release.",
    ].join("\n");

    expect(parseChangelog(text)).toEqual([
      {
        title: "Unreleased",
        groups: [
          { title: "Added", items: ["One item that wraps.", "Two"] },
          { title: "Fixed", items: ["Performance improvements."] },
        ],
      },
      { title: "0.1.0 - 2026-10-23", groups: [{ title: "Added", items: ["First release."] }] },
    ]);
  });

  test("the real changelog parses into releases with items", () => {
    const releases = parseChangelog(changelog);

    expect(releases.length).toBeGreaterThan(0);
    expect(releases.every((r) => r.groups.every((g) => g.items.length > 0))).toBe(true);
  });
});

describe("parseInline", () => {
  test("splits code runs and drops bold marks and link targets", () => {
    expect(parseInline("Bundle files (`.w3obj`), **new**, see [docs](docs/a.md)")).toEqual([
      { text: "Bundle files (", code: false },
      { text: ".w3obj", code: true },
      { text: "), new, see docs", code: false },
    ]);
  });
});
