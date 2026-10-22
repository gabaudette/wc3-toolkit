import { describe, expect, test } from "bun:test";
import { hasBit, inList, withBit, withListItem } from "./flags";

const choice = (value: string, aliases: string[] = []) => ({ value, label: value, aliases });

describe("bit flags", () => {
  test("5 has bits 0 and 2", () => {
    expect([0, 1, 2].map((bit) => hasBit("5", choice(String(bit))))).toEqual([true, false, true]);
  });

  test("switching bits on and off", () => {
    expect(withBit("5", choice("1"), true)).toBe("7");
    expect(withBit("5", choice("0"), false)).toBe("4");
    expect(withBit("5", choice("2"), true)).toBe("5");
    expect(withBit("", choice("3"), true)).toBe("8");
  });
});

describe("choice lists", () => {
  test("membership ignores case and spaces", () => {
    expect(inList("air, Ground", choice("ground"))).toBe(true);
    expect(inList("air", choice("ground"))).toBe(false);
  });

  test("other spellings count as the choice", () => {
    const invulnerable = choice("invulnerable", ["invu"]);

    expect(inList("air,invu", invulnerable)).toBe(true);
    expect(withListItem("air,invu", invulnerable, true)).toBe("air,invu");
    expect(withListItem("air,invu", invulnerable, false)).toBe("air");
  });

  test("adding and removing keeps unknown items", () => {
    expect(withListItem("air,enem", choice("ground"), true)).toBe("air,enem,ground");
    expect(withListItem("air,enem,ground", choice("air"), false)).toBe("enem,ground");
    expect(withListItem("", choice("air"), true)).toBe("air");
  });
});
