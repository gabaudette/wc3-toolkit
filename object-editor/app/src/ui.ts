import { attemptSync } from "./result";
import type { Field, UnitDetail } from "./types";

export function classes(...parts: (string | false | null | undefined)[]): string {
  return parts.filter(Boolean).join(" ");
}

export function levelCount(levels: number): string {
  if (levels === 1) {
    return "1 level";
  }

  return `${levels} levels`;
}

export const FIELD_ROW_GRID =
  "grid grid-cols-[minmax(12rem,1fr)_minmax(9rem,16rem)_minmax(3.5rem,7rem)_1.75rem] items-start gap-x-3";

const MODEL_EXTENSION = /\.md[lx]$/i;

/**
 * The model file a preview loads for a destructable or doodad: they name their
 * model without the extension and number its variations (`AshenTree` →
 * `AshenTree0.mdl` when there are several).
 */
export function variationModel(raw: string, variations: number): string {
  if (MODEL_EXTENSION.test(raw)) {
    return raw;
  }

  let file = raw;

  if (variations > 1) {
    file = `${file}0`;
  }

  return `${file}.mdl`;
}

export function besides(path: string, name: string): string {
  const at = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));

  if (at < 0) {
    return name;
  }

  return `${path.slice(0, at + 1)}${name}`;
}

export function counted(count: number, word: string): string {
  if (count === 1) {
    return `1 ${word}`;
  }

  return `${count} ${word}s`;
}

export function objectCount(count: number): string {
  if (count === 1) {
    return "1 object";
  }

  return `${count} objects`;
}

export function capitalized(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

export const remembered = {
  get(key: string): string | null {
    const stored = attemptSync(() => window.localStorage.getItem(key));
    if (stored.ok) {
      return stored.value;
    }

    return null;
  },
  set(key: string, value: string): void {
    attemptSync(() => window.localStorage.setItem(key, value));
  },
};

export const MAP_FILTERS = [{ name: "Warcraft III maps", extensions: ["w3m", "w3x"] }];

export const BUNDLE_FILTERS = [{ name: "Object bundles", extensions: ["w3obj"] }];
export const DESIGN_FILTERS = [{ name: "Interface designs", extensions: ["w3ui"] }];

const KILOBYTE = 1024;
const MEGABYTE = KILOBYTE * KILOBYTE;

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
