import { describe, expect, test } from "bun:test";
import { parseJass, signatures } from "./jass";

describe("JASS declarations", () => {
  test("retains signatures and types without implementation code or comments", () => {
    const declarations = parseJass(
      `\uFEFFtype unit extends widget\r
constant native GetUnitX takes unit whichUnit returns real // coordinate
globals
constant integer EXAMPLE = 7
integer internalState = 0
endglobals
function Example takes nothing returns nothing
  local string privateText = "implementation must stay local"
endfunction`,
      "synthetic.j",
    );
    expect(declarations.map((entry) => entry.name)).toEqual([
      "unit",
      "GetUnitX",
      "EXAMPLE",
      "Example",
    ]);
    expect(declarations[1].parameters).toEqual([{ name: "whichUnit", type: "unit" }]);
    expect(signatures(declarations[1])).toEqual({
      jass: "native GetUnitX takes unit whichUnit returns real",
      lua: "GetUnitX(whichUnit)",
    });
    expect(JSON.stringify(declarations)).not.toContain("implementation must stay local");
    expect(signatures(declarations[3]).lua).toBe("Example()");
  });

  test("rejects malformed and duplicate declarations with source context", () => {
    expect(() => parseJass("native Broken takes unit returns real", "bad.j")).toThrow("bad.j:1");
    expect(() => parseJass("type unit extends widget\ntype unit extends widget", "bad.j")).toThrow(
      "duplicate",
    );
    expect(() => parseJass("function Broken takes nothing returns nothing", "bad.j")).toThrow(
      "unterminated",
    );
    expect(() => parseJass("// empty", "bad.j")).toThrow("no declarations");
  });
});
