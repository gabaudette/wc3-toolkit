import { expect, test } from "bun:test";
import { buttonText, luaNames, titleText } from "./interface";
import type { DialogDesign } from "./types";

const dialog = (id: string, name: string): DialogDesign => ({
  kind: "dialog",
  id,
  name,
  title: "",
  titleColor: null,
  buttons: [],
  showAtStart: false,
});

// The same cases as `lua_names_come_from_names_and_stay_unique` in interface.rs.
test("Lua names come from names and stay unique", () => {
  const names = luaNames([
    dialog("a", "Choose your path!"),
    dialog("b", "choose  YOUR path"),
    dialog("c", "3 lives"),
    dialog("d", ""),
  ]);

  expect(names.get("a")).toBe("ChooseYourPath");
  expect(names.get("b")).toBe("ChooseYOURPath2");
  expect(names.get("c")).toBe("Design3Lives");
  expect(names.get("d")).toBe("Design");
});

// The same cases as `hotkey_letters_turn_gold_when_highlighted` in interface.rs.
test("hotkey letters turn gold when highlighted", () => {
  const hard = {
    id: "hard",
    text: "Hard",
    hotkey: "h",
    highlightHotkey: true,
    action: { kind: "close" as const },
  };

  expect(buttonText(hard)).toBe("|cffffcc00H|rard");
  expect(buttonText({ ...hard, text: "Go Ahead" })).toBe("Go A|cffffcc00h|read");
  expect(buttonText({ ...hard, text: "Normal" })).toBe("Normal");
  expect(buttonText({ ...hard, highlightHotkey: false })).toBe("Hard");
});

test("titles take their color", () => {
  const d = { ...dialog("a", "A"), title: "Pick" } as const;

  expect(titleText(d)).toBe("Pick");
  expect(titleText({ ...d, titleColor: "FF0303" })).toBe("|cffff0303Pick|r");
});
