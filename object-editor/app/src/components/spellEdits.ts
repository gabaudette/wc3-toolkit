import { api } from "../api";
import { ok, type Result } from "../result";
import type { AbilityDetail, Field, LevelField, SpellTemplate } from "../types";
import { fillLevels } from "./objectPane";
import type { BuffChoice } from "./SpellBuffStep";

export const NAME = "anam";
export const LEVELS = "alev";
const COMMON_VALUES = ["atar", "aran", "acdn", "amcs", "aare", "adur", "ahdu"];
const SPELL_DATA = "data";
/** The tooltip title and text, per level. */
export const TOOLTIPS = ["atp1", "aub1"];
/** The tooltip text, which a triggered spell's values fill in. */
export const TOOLTIP_TEXT = "aub1";
/** Tooltip texts that show values: normal, turned off, and the hero's learn text. */
const PLACEHOLDER_TEXTS = ["aub1", "auu1", "arut"];
/** Texts that usually repeat the spell's name: tooltip titles and the learn title. */
const NAMED_TEXTS = ["atp1", "aret"];
export const NO_LEVEL = 0;
/** The buffs a spell leaves on its target, per level, as a comma list of IDs. */
export const BUFFS = "abuf";
/** Fields of a spell's own buff named after it: the editor's name and the tooltip title. */
export const BUFF_NAMES = ["fnam", "ftip"];
const ICON = "aart";
/** A hero spell's icon in its learn menu; it follows the button's icon. */
const LEARN_ICON = "arar";
/** Texts that may show the hotkey highlighted, as older tooltips do: `(|cffffcc00T|r)`. */
const HOTKEY_TEXTS = ["atp1", "aret", "aut1"];
const HOTKEY = "ahky";
/** A hero spell's hotkey in its learn menu; it follows the cast hotkey when they matched. */
const LEARN_HOTKEY = "arhk";

export function plainField(detail: AbilityDetail, id: string): Field | undefined {
  return detail.sections.flatMap((s) => s.fields).find((f) => f.id === id);
}

export function levelField(detail: AbilityDetail, id: string): LevelField | undefined {
  return detail.sections.flatMap((s) => s.levelFields).find((f) => f.field.id === id);
}

export function triggeredFields(detail: AbilityDetail, template: SpellTemplate): LevelField[] {
  const ids = ["acdn", "amcs"];

  if (template.target !== "none") {
    ids.push("aran", "atar");
  }

  if (template.area > 0) {
    ids.push("aare");
  }

  return ids.flatMap((id) => {
    const field = levelField(detail, id);

    if (field) {
      return [field];
    }

    return [];
  });
}

export function valueFields(detail: AbilityDetail): LevelField[] {
  const all = detail.sections.flatMap((s) =>
    s.levelFields.map((f) => ({ f, category: s.category })),
  );
  const common = COMMON_VALUES.flatMap((id) => all.filter(({ f }) => f.field.id === id));
  const own = all.filter(
    ({ category, f }) => category === SPELL_DATA && !TOOLTIPS.includes(f.field.id),
  );

  return [...common, ...own].map(({ f }) => f);
}

function namedTexts(detail: AbilityDetail, id: string): { raw: string; level: number }[] {
  const perLevel = levelField(detail, id);

  if (perLevel) {
    return perLevel.cells.map((cell, index) => ({ raw: cell.raw, level: index + 1 }));
  }

  const plain = plainField(detail, id);

  if (plain) {
    return [{ raw: plain.raw, level: NO_LEVEL }];
  }

  return [];
}

export function idList(text: string): string[] {
  return text
    .split(",")
    .map((id) => id.trim())
    .filter(Boolean);
}

/**
 * The spell's buff field for a choice, or `null` to keep the base ability's.
 * Its own buff replaces the base's first and keeps any others (a few spells
 * leave two); another buff replaces them all.
 */
export function chosenBuffs(choice: BuffChoice, baseBuffs: string[]): string | null {
  if (choice.kind === "own") {
    return [choice.id, ...baseBuffs.slice(1)].join(",");
  }

  if (choice.kind === "other") {
    return choice.id;
  }

  return null;
}

export function buffShown(choice: BuffChoice, baseBuffs: string[]): string | null {
  if (choice.kind !== "base") {
    return choice.id;
  }

  return baseBuffs[0] ?? null;
}

async function replaceInTexts(
  id: string,
  detail: AbilityDetail,
  fields: string[],
  replace: (raw: string) => string,
  updated: Result<AbilityDetail>,
): Promise<Result<AbilityDetail>> {
  let last = updated;

  for (const fieldId of fields) {
    for (const { raw, level } of namedTexts(detail, fieldId)) {
      const replaced = replace(raw);

      if (replaced !== raw) {
        last = await api.setAbilityField(id, fieldId, level, replaced);
        if (!last.ok) {
          return last;
        }
      }
    }
  }

  return last;
}

/**
 * Points the copied tooltips at the spell's own values. Game tooltips name
 * their ability in placeholders (`<AHtb,DataA1>`), so a copy would keep
 * showing the original's numbers, in the editor and in game.
 */
function ownValues(
  id: string,
  base: string,
  detail: AbilityDetail,
): Promise<Result<AbilityDetail>> {
  const reference = new RegExp(`<${base},`, "gi");

  return replaceInTexts(
    id,
    detail,
    PLACEHOLDER_TEXTS,
    (raw) => raw.replace(reference, `<${id},`),
    ok(detail),
  );
}

export type SpellCopy = { name: string; detail: AbilityDetail };

export async function copyBase(base: string, onCreated: () => void): Promise<Result<SpellCopy>> {
  const created = await api.createAbility(base);

  if (!created.ok) {
    return created;
  }

  onCreated();
  const copied = await api.abilityDetail(created.value.id);

  if (!copied.ok) {
    return copied;
  }

  const own = await ownValues(created.value.id, base, copied.value);

  if (!own.ok) {
    return own;
  }

  return ok({ name: created.value.name, detail: own.value });
}

/**
 * Renames the spell, and its tooltip titles with it: they start as the base
 * ability's ("Storm Bolt - [Level 1]"), which would otherwise stay.
 */
export async function renamed(
  id: string,
  before: AbilityDetail | null,
  name: string,
): Promise<Result<AbilityDetail>> {
  const updated = await api.setAbilityField(id, NAME, NO_LEVEL, name);
  const old = before && plainField(before, NAME)?.raw;

  if (!updated.ok || !before || !old || old === name) {
    return updated;
  }

  return replaceInTexts(id, before, NAMED_TEXTS, (raw) => raw.replaceAll(old, name), updated);
}

export async function filled(
  id: string,
  field: string,
  fromLevel: number,
  levels: number,
  value: string,
): Promise<Result<AbilityDetail>> {
  return fillLevels((level) => api.setAbilityField(id, field, level, value), fromLevel, levels);
}

export async function withIcon(
  id: string,
  path: string,
  hero: boolean,
): Promise<Result<AbilityDetail>> {
  const updated = await api.setAbilityField(id, ICON, NO_LEVEL, path);

  if (updated.ok && hero) {
    return api.setAbilityField(id, LEARN_ICON, NO_LEVEL, path);
  }

  return updated;
}

/**
 * Sets a hotkey, and moves its highlight in tooltips that show it, as older
 * tooltips do (`Storm Bolt (|cffffcc00T|r)`); Reforged's leave it out. A
 * hero spell's learn hotkey follows the cast hotkey when they were the same.
 */
export async function withHotkey(
  id: string,
  before: AbilityDetail | null,
  field: string,
  key: string,
): Promise<Result<AbilityDetail>> {
  let updated = await api.setAbilityField(id, field, NO_LEVEL, key);
  const old = before && plainField(before, field)?.raw;

  if (!updated.ok || !before || !old || old === key) {
    return updated;
  }

  const learn = plainField(before, LEARN_HOTKEY)?.raw;

  if (field === HOTKEY && before.card.summary.hero && learn === old) {
    updated = await api.setAbilityField(id, LEARN_HOTKEY, NO_LEVEL, key);
    if (!updated.ok) {
      return updated;
    }
  }

  const highlight = `(|cffffcc00${old}|r)`;

  return replaceInTexts(
    id,
    before,
    HOTKEY_TEXTS,
    (raw) => raw.replaceAll(highlight, `(|cffffcc00${key}|r)`),
    updated,
  );
}

export async function setBuffs(
  id: string,
  buffs: string | null,
  levels: number,
): Promise<Result<AbilityDetail> | null> {
  let updated: Result<AbilityDetail> | null = null;

  for (let level = 1; level <= levels; level++) {
    if (buffs === null) {
      updated = await api.resetAbilityField(id, BUFFS, level);
    } else {
      updated = await api.setAbilityField(id, BUFFS, level, buffs);
    }

    if (!updated.ok) {
      return updated;
    }
  }

  return updated;
}
