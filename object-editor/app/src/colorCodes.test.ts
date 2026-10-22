import { describe, expect, test } from "bun:test";
import { applyColor, removeColor, stripColorCodes } from "./colorCodes";

describe("color codes", () => {
  test("wraps the selection and selects the result", () => {
    const text = "Heals 200 hit points";
    const start = text.indexOf("200");
    const edit = applyColor(text, start, start + "200".length, "FFCC00");

    expect(edit.text).toBe("Heals |cffffcc00200|r hit points");
    expect(edit.text.slice(edit.start, edit.end)).toBe("|cffffcc00200|r");
  });

  test("recoloring replaces the old code instead of nesting", () => {
    const text = "|cffffcc00Holy|r Light";
    const edit = applyColor(text, 0, text.indexOf(" "), "00ff00");

    expect(edit.text).toBe("|cff00ff00Holy|r Light");
  });

  test("removing color keeps the words", () => {
    const text = "Attacks |cffffcc00land|r units";
    const edit = removeColor(text, 0, text.length);

    expect(edit.text).toBe("Attacks land units");
    expect(edit.end).toBe(edit.text.length);
  });

  test("strips upper- and lower-case codes", () => {
    expect(stripColorCodes("|CFF00FF00a|Rb|cff123456c|r")).toBe("abc");
  });
});
