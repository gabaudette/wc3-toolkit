import { describe, expect, test } from "bun:test";
import { parseGameText } from "./GameText";

describe("parseGameText", () => {
  test("splits colored runs and line breaks", () => {
    expect(parseGameText("Heals |cffffcc00200|r hit points.|nDone")).toEqual([
      { text: "Heals ", color: null },
      { text: "200", color: "#ffcc00" },
      { text: " hit points.\nDone", color: null },
    ]);
  });

  test("codes are case-insensitive and a missing |r runs to the end", () => {
    expect(parseGameText("|CFF00FF00green|Nstill green")).toEqual([
      { text: "green\nstill green", color: "#00FF00" },
    ]);
  });

  test("text without codes is one plain run", () => {
    expect(parseGameText("Paladin")).toEqual([{ text: "Paladin", color: null }]);
  });

  test("a lone pipe stays as text", () => {
    expect(parseGameText("a | b")).toEqual([{ text: "a | b", color: null }]);
  });
});
