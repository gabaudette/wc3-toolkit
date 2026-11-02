import { describe, expect, test } from "bun:test";
import {
  diceProblem,
  diceStats,
  FIELD_GROUPS,
  hexToRgb,
  partProblem,
  rgbToHex,
  rollStats,
} from "./fieldGroups";

describe("field groups", () => {
  test("the Paladin's damage: 2 dice of 6 sides plus 22 Strength", () => {
    const paladin = { base: 0, dice: 2, sides: 6, strength: 22 };
    const stats = diceStats(paladin.base, paladin.dice, paladin.sides, paladin.strength);

    expect(stats).toEqual({ min: 24, max: 34, average: 29 });
    expect(rollStats(paladin.dice, paladin.sides)).toEqual({ min: 2, max: 12, average: 7 });
  });

  test("the Footman's damage: 11 base, 1 die of 2 sides", () => {
    expect(diceStats(11, 1, 2, 0)).toEqual({ min: 12, max: 13, average: 12.5 });
  });

  test("no dice leaves the base", () => {
    expect(diceStats(25, 0, 6, 0)).toEqual({ min: 25, max: 25, average: 25 });
  });

  test("dice need sides, and cannot be negative", () => {
    expect(diceProblem(2, 0)).toBe("A die needs at least 1 side.");
    expect(diceProblem(-1, 6)).toBe("Dice cannot be fewer than 0.");
    expect(diceProblem(0, 0)).toBeNull();
    expect(diceProblem(2, 6)).toBeNull();
  });

  test("typed values outside a field's limits are explained", () => {
    const sides = { label: "Sides", min: 1, max: 100, integer: true };

    expect(partProblem("6", sides)).toBeNull();
    expect(partProblem("0", sides)).toBe("Sides must be a whole number from 1 to 100.");
    expect(partProblem("2.5", sides)).toBe("Sides must be a whole number from 1 to 100.");
    expect(partProblem("", sides)).toBe("Sides must be a whole number from 1 to 100.");
    expect(partProblem("-3", { label: "Base", min: null, max: null, integer: true })).toBeNull();
    expect(partProblem("x", { label: "Base", min: 0, max: null, integer: false })).toBe(
      "Base must be a number of 0 or more.",
    );
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
