// Fields the World Editor lists separately but that belong together.

export type GroupKind = "dice" | "color" | "button" | "pair";

export type FieldGroup = {
  kind: GroupKind;
  label: string;
  /** Field IDs in display order. */
  ids: string[];
  /** Short labels for each part, in the same order. */
  parts: string[];
  /** Dice groups for attacks also add a hero's primary attribute. */
  attack?: boolean;
};

export const FIELD_GROUPS: FieldGroup[] = [
  {
    kind: "dice",
    label: "Attack 1 - Damage",
    ids: ["ua1b", "ua1d", "ua1s"],
    parts: ["Base", "Dice", "Sides"],
    attack: true,
  },
  {
    kind: "dice",
    label: "Attack 2 - Damage",
    ids: ["ua2b", "ua2d", "ua2s"],
    parts: ["Base", "Dice", "Sides"],
    attack: true,
  },
  {
    kind: "dice",
    label: "Gold Bounty Awarded",
    ids: ["ubba", "ubdi", "ubsi"],
    parts: ["Base", "Dice", "Sides"],
  },
  {
    kind: "dice",
    label: "Lumber Bounty Awarded",
    ids: ["ulba", "ulbd", "ulbs"],
    parts: ["Base", "Dice", "Sides"],
  },
  { kind: "color", label: "Tinting Color", ids: ["uclr", "uclg", "uclb"], parts: ["R", "G", "B"] },
  { kind: "button", label: "Button Position", ids: ["ubpx", "ubpy"], parts: ["Column", "Row"] },
  {
    kind: "button",
    label: "Button Position - Normal",
    ids: ["abpx", "abpy"],
    parts: ["Column", "Row"],
  },
  {
    kind: "button",
    label: "Button Position - Research",
    ids: ["arpx", "arpy"],
    parts: ["Column", "Row"],
  },
  {
    kind: "button",
    label: "Button Position - Turn Off",
    ids: ["aubx", "auby"],
    parts: ["Column", "Row"],
  },
  { kind: "pair", label: "Shadow Image - Center", ids: ["ushx", "ushy"], parts: ["X", "Y"] },
  { kind: "pair", label: "Shadow Image - Size", ids: ["ushw", "ushh"], parts: ["Width", "Height"] },
];

/** Command card size: 4 columns by 3 rows. */
export const BUTTON_COLUMNS = 4;
export const BUTTON_ROWS = 3;

const HEX_RADIX = 16;
const CHANNEL_MAX = 255;
const HEX_DIGITS = 2;

export type DiceRange = {
  min: number;
  max: number;
};

/** The game rolls base + `dice` dice with `sides` sides: from base + dice to base + dice × sides. */
export function diceRange(base: number, dice: number, sides: number, bonus: number): DiceRange {
  return { min: base + bonus + dice, max: base + bonus + dice * sides };
}

function clampChannel(value: number): number {
  return Math.min(CHANNEL_MAX, Math.max(0, Math.round(value)));
}

/** [255, 204, 0] → "#ffcc00". */
export function rgbToHex(channels: number[]): string {
  const hex = channels
    .map((c) => clampChannel(c).toString(HEX_RADIX).padStart(HEX_DIGITS, "0"))
    .join("");
  return `#${hex}`;
}

/** "#ffcc00" → [255, 204, 0]; `null` for anything that is not a 6-digit hex color. */
export function hexToRgb(hex: string): number[] | null {
  const match = /^#?([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex.trim());
  if (!match) {
    return null;
  }

  return match.slice(1).map((part) => Number.parseInt(part, HEX_RADIX));
}
