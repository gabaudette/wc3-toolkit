import { describe, expect, test } from "bun:test";
import { textProblems, unfilledPlaceholders } from "./gameTextCheck";

describe("game text checks", () => {
  test("well-formed text has no problems", () => {
    expect(
      textProblems("Deals <AHbz,DataA1> damage.|n|cffffcc00Level 1|r for <AHbz,DataB1,%>"),
    ).toEqual([]);
  });

  test("a color code needs 8 hex digits", () => {
    expect(textProblems("|cffcc00Gold|r")).toEqual([
      "“|cffcc00Gold”: a color needs 8 hex digits after |c, like |cffffcc00.",
      "“|r” ends a color, but no color is open there.",
    ]);
  });

  test("an open color and unknown codes are reported", () => {
    expect(textProblems("|cffff0000Red|x")).toEqual([
      "“|x” is not a text code; the codes are |c, |r and |n.",
      "A color is never ended with |r, so it runs to the end of the text.",
    ]);
  });

  test("placeholders must be closed and hold an ID and a column", () => {
    expect(textProblems("Deals <AHbz,DataA1 damage")).toEqual([
      "“<” starts a placeholder that is never closed with “>”.",
    ]);
    expect(textProblems("<Empty> slot")).toEqual([
      "“<Empty> slot” is not a placeholder like <AHbz,DataA1> or <AHbz,DataA1,%>.",
    ]);
  });

  test("placeholders left after filling are listed once", () => {
    expect(unfilledPlaceholders("Deals 30 and <XXXX,DataZ9> or <XXXX,DataZ9>")).toEqual([
      "<XXXX,DataZ9>",
    ]);
    expect(unfilledPlaceholders("Deals 30 damage.")).toEqual([]);
  });
});
