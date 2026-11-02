import { describe, expect, test } from "bun:test";
import { assistantPrompt, mapSummary } from "./assistantPrompt";
import type { AssistantContext } from "./types";

const context: AssistantContext = {
  balance: null,
  map: {
    name: "Lanes.w3x",
    script: "Lua",
    players: ['Player 1 (Red) "Radiant": user, any race'],
    forces: ['"Radiant": Player 1 (Red), Player 2 (Blue), allied'],
    objects: [
      { kind: "units", custom: 3, changed: 1 },
      { kind: "items", custom: 0, changed: 0 },
    ],
    placed: 12,
    importedFiles: 2,
  },
};

describe("assistant prompt", () => {
  test("fills the goal and the map summary", () => {
    const prompt = assistantPrompt("A Dota 2 style map with three lanes", context);

    expect(prompt).toContain("A Dota 2 style map with three lanes");
    expect(prompt).toContain("- File: Lanes.w3x");
    expect(prompt).toContain("- Objects: units 3 custom and 1 changed.");
    expect(prompt).not.toContain("{{");
  });

  test("reads well without a goal or a map", () => {
    const prompt = assistantPrompt("  ", null);

    expect(prompt).toContain("ask me what kind of map I want to make");
    expect(prompt).toContain("No map is open in Tinkerbench yet");
  });

  test("lists players and forces under their headings", () => {
    expect(mapSummary(context)).toContain(
      '- Players:\n  - Player 1 (Red) "Radiant": user, any race',
    );
  });
});
