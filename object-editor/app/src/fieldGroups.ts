export type GroupKind = "dice" | "color" | "button" | "pair";

export type FieldGroup = {
  kind: GroupKind;
  label: string;
  ids: string[];
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

export type DiceStats = {
  min: number;
  max: number;
  average: number;
};

/** Each of `dice` dice rolls 1 to `sides`, evenly. */
export function rollStats(dice: number, sides: number): DiceStats {
  return { min: dice, max: dice * sides, average: (dice * (sides + 1)) / 2 };
}

export function diceStats(base: number, dice: number, sides: number, bonus: number): DiceStats {
  const roll = rollStats(dice, sides);
  const added = base + bonus;

  return { min: added + roll.min, max: added + roll.max, average: added + roll.average };
}

export function diceProblem(dice: number, sides: number): string | null {
  if (dice < 0) {
    return "Dice cannot be fewer than 0.";
  }

  if (dice > 0 && sides < 1) {
    return "A die needs at least 1 side.";
  }

  return null;
}

export type PartLimits = {
  label: string;
  min: number | null;
  max: number | null;
  integer: boolean;
};

function limitsText(limits: PartLimits): string {
  const { min, max } = limits;

  if (min !== null && max !== null) {
    return ` from ${min} to ${max}`;
  }

  if (min !== null) {
    return ` of ${min} or more`;
  }

  if (max !== null) {
    return ` of ${max} or less`;
  }

  return "";
}

export function partProblem(text: string, limits: PartLimits): string | null {
  const value = Number(text);
  let kind = "a number";

  if (limits.integer) {
    kind = "a whole number";
  }

  const wanted = `${limits.label} must be ${kind}${limitsText(limits)}.`;

  if (!text.trim() || !Number.isFinite(value)) {
    return wanted;
  }

  if (limits.integer && !Number.isInteger(value)) {
    return wanted;
  }

  if ((limits.min !== null && value < limits.min) || (limits.max !== null && value > limits.max)) {
    return wanted;
  }

  return null;
}

function clampChannel(value: number): number {
  return Math.min(CHANNEL_MAX, Math.max(0, Math.round(value)));
}

export function rgbToHex(channels: number[]): string {
  const hex = channels
    .map((c) => clampChannel(c).toString(HEX_RADIX).padStart(HEX_DIGITS, "0"))
    .join("");

  return `#${hex}`;
}

export function hexToRgb(hex: string): number[] | null {
  const match = /^#?([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex.trim());

  if (!match) {
    return null;
  }

  return match.slice(1).map((part) => Number.parseInt(part, HEX_RADIX));
}
