import { describe, expect, test } from "bun:test";
import { diceRange, FIELD_GROUPS, hexToRgb, rgbToHex } from "./fieldGroups";

describe("field groups", () => {
  test("the Paladin's damage: 2 dice of 6 sides plus 22 Strength", () => {
    const paladin = { base: 0, dice: 2, sides: 6, strength: 22 };
    const range = diceRange(paladin.base, paladin.dice, paladin.sides, paladin.strength);

    expect(range).toEqual({ min: 24, max: 34 });
  });

  test("colors convert both ways", () => {
    expect(rgbToHex([255, 204, 0])).toBe("#ffcc00");
    expect(hexToRgb("#FFCC00")).toEqual([255, 204, 0]);
    expect(hexToRgb("not a color")).toBeNull();
  });

  test("channels are clamped to 0–255", () => {
    expect(rgbToHex([300, -5, 12.6])).toBe("#ff000d");
  });

  test("no field belongs to two groups", () => {
    const ids = FIELD_GROUPS.flatMap((g) => g.ids);

    expect(new Set(ids).size).toBe(ids.length);
  });
});
