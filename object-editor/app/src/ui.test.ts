import { describe, expect, test } from "bun:test";
import { besides, counted } from "./ui";

describe("besides", () => {
  test("puts a file name in the same folder, with either separator", () => {
    expect(besides("C:\\Maps\\test.w3m", "Scores.w3ui")).toBe("C:\\Maps\\Scores.w3ui");
    expect(besides("/home/me/maps/test.w3m", "Scores.w3ui")).toBe("/home/me/maps/Scores.w3ui");
    expect(besides("test.w3m", "Scores.w3ui")).toBe("Scores.w3ui");
  });
});

describe("counted", () => {
  test("says one or many", () => {
    expect(counted(1, "design")).toBe("1 design");
    expect(counted(3, "design")).toBe("3 designs");
  });
});
