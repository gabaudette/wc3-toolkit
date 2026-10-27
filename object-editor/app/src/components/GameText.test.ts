import { describe, expect, test } from "bun:test";
import { parseGameText, plainGameText, readableColor } from "./GameText";

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

describe("plainGameText", () => {
  test("drops codes and turns line breaks into spaces", () => {
    expect(plainGameText("|cffffcc00Holy|r Paladin|nLevel 2")).toBe("Holy Paladin Level 2");
  });
});

describe("readableColor", () => {
  test("keeps colors that already read well", () => {
    expect(readableColor("#ffcc00")).toBe("#ffcc00");
  });

  test("lightens near-black colors until they read on dark panels", () => {
    const lightened = readableColor("#200000");

    expect(lightened).not.toBe("#200000");
    // Still red: the red channel stays the strongest.
    const [r, g, b] = [1, 3, 5].map((i) => Number.parseInt(lightened.slice(i, i + 2), 16));

    expect(r).toBeGreaterThan(g ?? 0);
    expect(g).toBe(b);
  });
});
