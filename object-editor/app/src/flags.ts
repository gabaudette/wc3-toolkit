// Fields whose value is a set of choices: bit flags stored as a number
// (`stackFlags`: choice n is bit n) and lists of names (`targetList`: "air,ground").

import type { Choice } from "./types";

const LIST_SEPARATOR = ",";
const BINARY = 2;

function bitOf(choice: Choice): number | null {
  const bit = Number.parseInt(choice.value, 10);

  if (Number.isInteger(bit) && bit >= 0) {
    return bit;
  }

  return null;
}

function bitsOf(raw: string): number {
  const bits = Number.parseInt(raw, 10);

  if (Number.isInteger(bits)) {
    return bits;
  }

  return 0;
}

export function hasBit(raw: string, choice: Choice): boolean {
  const bit = bitOf(choice);

  return bit !== null && Math.floor(bitsOf(raw) / BINARY ** bit) % BINARY === 1;
}

export function withBit(raw: string, choice: Choice, on: boolean): string {
  const bit = bitOf(choice);

  if (bit === null || hasBit(raw, choice) === on) {
    return String(bitsOf(raw));
  }

  const value = BINARY ** bit;

  if (on) {
    return String(bitsOf(raw) + value);
  }

  return String(bitsOf(raw) - value);
}

function items(raw: string): string[] {
  return raw
    .split(LIST_SEPARATOR)
    .map((item) => item.trim())
    .filter(Boolean);
}

function spellings(choice: Choice): string[] {
  return [choice.value, ...choice.aliases].map((s) => s.toLowerCase());
}

export function inList(raw: string, choice: Choice): boolean {
  const wanted = spellings(choice);

  return items(raw).some((item) => wanted.includes(item.toLowerCase()));
}

export function withListItem(raw: string, choice: Choice, on: boolean): string {
  if (on && inList(raw, choice)) {
    return items(raw).join(LIST_SEPARATOR);
  }

  const wanted = spellings(choice);
  const others = items(raw).filter((item) => !wanted.includes(item.toLowerCase()));

  if (on) {
    return [...others, choice.value].join(LIST_SEPARATOR);
  }

  return others.join(LIST_SEPARATOR);
}

export type ChoiceGroup = { label: string | null; choices: Choice[] };

const TARGET_LIST = "targetList";

const TARGET_GROUPS = [
  {
    label: "Owner",
    values: ["self", "notself", "player", "allies", "friend", "enemies", "neutral"],
  },
  {
    label: "Kind",
    values: [
      "air",
      "ground",
      "structure",
      "hero",
      "nonhero",
      "ward",
      "item",
      "tree",
      "debris",
      "wall",
      "bridge",
      "decoration",
      "terrain",
    ],
  },
  {
    label: "State",
    values: [
      "alive",
      "dead",
      "vulnerable",
      "invulnerable",
      "organic",
      "mechanical",
      "sapper",
      "nonsapper",
      "ancient",
      "nonancient",
    ],
  },
];

export function choiceGroups(valueType: string, options: Choice[]): ChoiceGroup[] {
  if (valueType !== TARGET_LIST) {
    return [{ label: null, choices: options }];
  }

  const grouped = TARGET_GROUPS.flatMap((group) => group.values);
  const groups = TARGET_GROUPS.map(({ label, values }) => ({
    label,
    choices: values.flatMap((value) => options.filter((choice) => choice.value === value)),
  }));
  const rest = options.filter((choice) => !grouped.includes(choice.value));

  return [...groups, { label: "Other", choices: rest }].filter((group) => group.choices.length > 0);
}
