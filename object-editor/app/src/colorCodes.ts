// Editing Warcraft III color codes in text: `|cAARRGGBB` ... `|r`.

/** Opaque alpha, as the World Editor writes it. */
const ALPHA = "ff";
const COLOR_START = /\|c[0-9a-f]{8}/gi;
const COLOR_END = /\|r/gi;

export type Swatch = {
  name: string;
  hex: string;
};

/** The gold used for highlights in game tooltips, the 12 player colors, and white. */
export const SWATCHES: Swatch[] = [
  { name: "Gold (tooltip highlight)", hex: "ffcc00" },
  { name: "Red", hex: "ff0303" },
  { name: "Blue", hex: "0042ff" },
  { name: "Teal", hex: "1ce6b9" },
  { name: "Purple", hex: "540081" },
  { name: "Yellow", hex: "fffc00" },
  { name: "Orange", hex: "fe8a0e" },
  { name: "Green", hex: "20c000" },
  { name: "Pink", hex: "e55bb0" },
  { name: "Gray", hex: "959697" },
  { name: "Light blue", hex: "7ebff1" },
  { name: "Dark green", hex: "106246" },
  { name: "Brown", hex: "4e2a04" },
  { name: "White", hex: "ffffff" },
];

export type Edit = {
  text: string;
  /** The selection after the edit, covering the changed text. */
  start: number;
  end: number;
};

/** Removes color codes from a piece of text, keeping the words. */
export function stripColorCodes(text: string): string {
  return text.replace(COLOR_START, "").replace(COLOR_END, "");
}

/**
 * Colors the selected part of `text`. Codes already inside the selection are
 * removed first so colors never nest.
 */
export function applyColor(text: string, start: number, end: number, hex: string): Edit {
  const selected = stripColorCodes(text.slice(start, end));
  const colored = `|c${ALPHA}${hex.toLowerCase()}${selected}|r`;

  return {
    text: text.slice(0, start) + colored + text.slice(end),
    start,
    end: start + colored.length,
  };
}

/** Removes color codes from the selected part of `text`. */
export function removeColor(text: string, start: number, end: number): Edit {
  const plain = stripColorCodes(text.slice(start, end));

  return {
    text: text.slice(0, start) + plain + text.slice(end),
    start,
    end: start + plain.length,
  };
}
