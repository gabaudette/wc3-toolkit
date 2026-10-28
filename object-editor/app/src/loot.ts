import type { DropView, ItemChoice, Tier, TierItem } from "./types";

// Random drops are written as `Y<class>I<level>`: `YYI` for any class, then
// one letter per item class in the order of JASS's ITEM_TYPE constants.
export const ITEM_CLASSES = [
  { id: "Permanent", letter: "i", label: "Permanent" },
  { id: "Charged", letter: "j", label: "Charged" },
  { id: "PowerUp", letter: "k", label: "Power-up" },
  { id: "Artifact", letter: "l", label: "Artifact" },
  { id: "Purchasable", letter: "m", label: "Purchasable" },
  { id: "Campaign", letter: "n", label: "Campaign" },
  { id: "Miscellaneous", letter: "o", label: "Miscellaneous" },
];

const ANY_CLASS_LETTER = "Y";
const RANDOM_PREFIX = "Y";
const RANDOM_MARK = "I";
const MAX_ITEM_LEVEL = 9;
const SLOT_COUNT = 6;
export const ITEM_LEVELS = Array.from({ length: MAX_ITEM_LEVEL + 1 }, (_, level) => level);
export const HERO_SLOTS = Array.from({ length: SLOT_COUNT }, (_, slot) => slot);
export const FULL_CHANCE = 100;

export type RandomItem = { classId: string | null; level: number };

export function randomItemCode(random: RandomItem): string {
  const letter = ITEM_CLASSES.find((c) => c.id === random.classId)?.letter ?? ANY_CLASS_LETTER;

  return `${RANDOM_PREFIX}${letter}${RANDOM_MARK}${random.level}`;
}

export function parseRandomItem(code: string): RandomItem | null {
  const [prefix, letter, mark, digit, ...rest] = code;

  if (prefix !== RANDOM_PREFIX || mark !== RANDOM_MARK || rest.length > 0 || !digit) {
    return null;
  }

  const level = Number(digit);

  if (!Number.isInteger(level)) {
    return null;
  }

  if (letter === ANY_CLASS_LETTER) {
    return { classId: null, level };
  }

  const itemClass = ITEM_CLASSES.find((c) => c.letter === letter);

  if (!itemClass) {
    return null;
  }

  return { classId: itemClass.id, level };
}

export function classLabel(classId: string | null): string {
  return ITEM_CLASSES.find((c) => c.id === classId)?.label ?? "any class";
}

export type ItemLabel = { name: string; icon: string | null; random: boolean };

export function itemLabel(code: string, choices: Map<string, ItemChoice>): ItemLabel {
  const choice = choices.get(code);

  if (choice) {
    return { name: choice.name, icon: choice.icon, random: false };
  }

  const random = parseRandomItem(code);

  if (random) {
    return {
      name: `Random level ${random.level} item, ${classLabel(random.classId).toLowerCase()}`,
      icon: null,
      random: true,
    };
  }

  return { name: `Unknown item ${code}`, icon: null, random: false };
}

export function setTotal(set: DropView[]): number {
  return set.reduce((sum, drop) => sum + drop.chance, 0);
}

export function percent(value: number): string {
  if (value > 0 && value < 1) {
    return `${value.toFixed(2)}%`;
  }

  return `${Math.round(value * 10) / 10}%`;
}

export type DropDraft = DropView & { key: string };

export type DropSetDraft = { key: string; drops: DropDraft[] };

export type TierItemDraft = TierItem & { key: string };

export type TierDraft = { key: string; name: string; chance: number; items: TierItemDraft[] };

export function newKey(): string {
  return crypto.randomUUID();
}

export function setDrafts(sets: DropView[][]): DropSetDraft[] {
  return sets.map((set) => ({
    key: newKey(),
    drops: set.map((drop) => ({ ...drop, key: newKey() })),
  }));
}

export function setsOf(drafts: DropSetDraft[]): DropView[][] {
  return drafts.map((set) => set.drops.map(({ item, chance }) => ({ item, chance })));
}

export function tierDrafts(tiers: Tier[]): TierDraft[] {
  return tiers.map((tier) => ({
    ...tier,
    key: newKey(),
    items: tier.items.map((entry) => ({ ...entry, key: newKey() })),
  }));
}

export function tiersOf(drafts: TierDraft[]): Tier[] {
  return drafts.map(({ name, chance, items }) => ({
    name,
    chance,
    items: items.map(({ item, weight }) => ({ item, weight })),
  }));
}
