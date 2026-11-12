import { describe, expect, test } from "bun:test";
import { unreleasedHasSharedLine } from "./changelog";

const lines = (...text: string[]) => text.join("\n");

describe("unreleasedHasSharedLine", () => {
  test("finds the internal line under Unreleased", () => {
    const changelog = lines(
      "# Changelog",
      "",
      "## Unreleased",
      "",
      "### Changed",
      "",
      "- Something users notice.",
      "- Internal improvements.",
      "",
      "## 0.3.0 - 2026-11-04",
    );

    expect(unreleasedHasSharedLine(changelog)).toBe(true);
  });

  test("finds the performance line with Windows line endings", () => {
    const changelog = lines("## Unreleased\r", "", "- Performance improvements.\r", "");

    expect(unreleasedHasSharedLine(changelog)).toBe(true);
  });

  test("ignores the line in a released version", () => {
    const changelog = lines(
      "## Unreleased",
      "",
      "- Something users notice.",
      "",
      "## 0.3.0 - 2026-11-04",
      "",
      "- Internal improvements.",
    );

    expect(unreleasedHasSharedLine(changelog)).toBe(false);
  });

  test("needs an Unreleased section", () => {
    const changelog = lines("## 0.3.0 - 2026-11-04", "", "- Internal improvements.");

    expect(unreleasedHasSharedLine(changelog)).toBe(false);
  });

  test("ignores the words inside a longer entry", () => {
    const changelog = lines("## Unreleased", "", "- Internal improvements to the map view.");

    expect(unreleasedHasSharedLine(changelog)).toBe(false);
  });
});
