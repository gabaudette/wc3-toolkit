import { describe, expect, test } from "bun:test";
import { inside, moved, typesCharacter } from "./grid";

describe("grid movement", () => {
  const start = { row: 1, column: 1 };

  test("arrows move one cell and stop at the edges", () => {
    expect(moved(start, "ArrowRight", 3, 3)).toEqual({ row: 1, column: 2 });
    expect(moved({ row: 0, column: 0 }, "ArrowUp", 3, 3)).toEqual({ row: 0, column: 0 });
    expect(moved({ row: 2, column: 2 }, "ArrowDown", 3, 3)).toEqual({ row: 2, column: 2 });
  });

  test("Home and End jump to the first and last level", () => {
    expect(moved(start, "Home", 3, 5)).toEqual({ row: 1, column: 0 });
    expect(moved(start, "End", 3, 5)).toEqual({ row: 1, column: 4 });
  });

  test("other keys do not move", () => {
    expect(moved(start, "Enter", 3, 3)).toBeNull();
  });

  test("typing starts editing, shortcuts do not", () => {
    expect(typesCharacter("5", false)).toBe(true);
    expect(typesCharacter("c", true)).toBe(false);
    expect(typesCharacter("Delete", false)).toBe(false);
  });
});

describe("inside", () => {
  test("leaves a cell inside the table alone", () => {
    expect(inside({ row: 1, column: 2 }, 3, 3)).toBeNull();
  });

  test("moves a cell back inside after rows or levels shrink", () => {
    expect(inside({ row: 5, column: 4 }, 3, 2)).toEqual({ row: 2, column: 1 });
  });

  test("settles at the first cell of an empty table", () => {
    expect(inside({ row: 0, column: 0 }, 0, 0)).toBeNull();
    expect(inside({ row: 2, column: 1 }, 0, 0)).toEqual({ row: 0, column: 0 });
  });
});
