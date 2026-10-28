import { describe, expect, test } from "bun:test";
import { itemLabel, parseRandomItem, percent, randomItemCode } from "./loot";

describe("random item codes", () => {
  test("name a class and level like the World Editor", () => {
    expect(randomItemCode({ classId: "Charged", level: 3 })).toBe("YjI3");
    expect(randomItemCode({ classId: null, level: 8 })).toBe("YYI8");
    expect(parseRandomItem("YkI1")).toEqual({ classId: "PowerUp", level: 1 });
    expect(parseRandomItem("YYI2")).toEqual({ classId: null, level: 2 });
  });

  test("leave real items alone", () => {
    expect(parseRandomItem("ratc")).toBeNull();
    expect(parseRandomItem("YzI1")).toBeNull();
  });

  test("label random drops for people", () => {
    expect(itemLabel("YiI2", new Map()).name).toBe("Random level 2 item, permanent");
    expect(itemLabel("YYI8", new Map()).random).toBe(true);
  });
});

describe("percents", () => {
  test("keep small chances readable", () => {
    expect(percent(0.4)).toBe("0.40%");
    expect(percent(46.55)).toBe("46.6%");
  });
});
