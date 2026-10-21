// Small UI helpers shared by components.

/** Joins class names, skipping falsy entries (instead of ternaries in JSX). */
export function classes(...parts: (string | false | null | undefined)[]): string {
  return parts.filter(Boolean).join(" ");
}

/** "1 level", "3 levels". */
export function levelCount(levels: number): string {
  if (levels === 1) {
    return "1 level";
  }

  return `${levels} levels`;
}

/** "human" → "Human". */
export function capitalized(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/** Remembered paths; browser storage can be unavailable, so failures are ignored. */
export const remembered = {
  get(key: string): string | null {
    try {
      return window.localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  set(key: string, value: string): void {
    try {
      window.localStorage.setItem(key, value);
    } catch {
      // Not remembering a path is harmless; the user picks it again next time.
    }
  },
};

export const GAME_DIR_KEY = "wc3-object-editor.gameDir";
export const MAP_PATH_KEY = "wc3-object-editor.mapPath";
export const HD_KEY = "wc3-object-editor.hd";
