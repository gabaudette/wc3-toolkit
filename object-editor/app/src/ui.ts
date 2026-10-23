// Small UI helpers shared by components.

import type { Field, UnitDetail } from "./types";

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

/**
 * Columns of every field row and the header above them: name, value, the
 * base object's value, and the reset button.
 */
export const FIELD_ROW_GRID =
  "grid grid-cols-[minmax(12rem,1fr)_minmax(9rem,16rem)_minmax(3.5rem,7rem)_1.75rem] items-start gap-x-3";

/** "1 object", "3 objects". */
export function objectCount(count: number): string {
  if (count === 1) {
    return "1 object";
  }

  return `${count} objects`;
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

/** File types the map pickers offer. */
export const MAP_FILTERS = [{ name: "Warcraft III maps", extensions: ["w3m", "w3x"] }];

/** File types the bundle file pickers offer. */
export const BUNDLE_FILTERS = [{ name: "Object bundles", extensions: ["w3obj"] }];

const KILOBYTE = 1024;
const MEGABYTE = KILOBYTE * KILOBYTE;

/** "820 bytes", "35 KB", "1.5 MB". */
export function fileSize(bytes: number): string {
  if (bytes >= MEGABYTE) {
    return `${(bytes / MEGABYTE).toFixed(1)} MB`;
  }

  if (bytes >= KILOBYTE) {
    return `${Math.round(bytes / KILOBYTE)} KB`;
  }

  return `${bytes} bytes`;
}

export const GAME_DIR_KEY = "wc3-object-editor.gameDir";
export const MAP_PATH_KEY = "wc3-object-editor.mapPath";
export const HD_KEY = "wc3-object-editor.hd";

/** The page with one field replaced by its new state after an edit. */
export function replaceField(detail: UnitDetail, updated: Field): UnitDetail {
  return {
    ...detail,
    sections: detail.sections.map((section) => ({
      ...section,
      fields: section.fields.map((f) => {
        if (f.id === updated.id) {
          return updated;
        }

        return f;
      }),
    })),
  };
}
