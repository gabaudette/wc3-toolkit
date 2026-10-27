import type { Design, DialogButton, DialogDesign } from "./types";

/**
 * `Choose your path!` → `ChooseYourPath`, as the Lua module names it
 * (`interface.rs`, `pascal_case`); `Design` when nothing is left.
 */
function pascalCase(name: string): string {
  let out = name
    .split(/[^A-Za-z0-9]/)
    .filter(Boolean)
    .map((word) => word[0]?.toUpperCase() + word.slice(1))
    .join("");
  if (out === "" || /^[0-9]/.test(out)) {
    out = `Design${out}`;
  }

  return out;
}

/** Each design's Lua name by ID, numbered when two match, like `lua_names` in Rust. */
export function luaNames(designs: Design[]): Map<string, string> {
  const taken = new Set<string>();
  const names = new Map<string, string>();
  for (const design of designs) {
    const base = pascalCase(design.name);
    let name = base;
    let n = 2;
    while (taken.has(name.toLowerCase())) {
      name = `${base}${n}`;
      n++;
    }

    taken.add(name.toLowerCase());
    names.set(design.id, name);
  }

  return names;
}

/** The gold the game highlights hotkeys with. */
const HOTKEY_COLOR = "ffcc00";

/** The message as the game gets it, like `title_text` in Rust. */
export function titleText(dialog: DialogDesign): string {
  if (!dialog.titleColor) {
    return dialog.title;
  }

  return `|cff${dialog.titleColor.toLowerCase()}${dialog.title}|r`;
}

/** A button's text as the game gets it, like `button_text` in Rust. */
export function buttonText(button: DialogButton): string {
  const key = button.hotkey.trim();
  if (!button.highlightHotkey || [...key].length !== 1) {
    return button.text;
  }

  const at = button.text.toLowerCase().indexOf(key.toLowerCase());
  if (at < 0) {
    return button.text;
  }

  const letter = button.text.slice(at, at + key.length);
  return `${button.text.slice(0, at)}|cff${HOTKEY_COLOR}${letter}|r${button.text.slice(at + key.length)}`;
}
