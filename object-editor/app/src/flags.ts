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

/** Whether a choice is on in a bit flags value like "5" (bits 0 and 2). */
export function hasBit(raw: string, choice: Choice): boolean {
  const bit = bitOf(choice);
  return bit !== null && Math.floor(bitsOf(raw) / BINARY ** bit) % BINARY === 1;
}

/** The bit flags value with one choice switched on or off. */
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

/** The choice's value and its other spellings, lowercase. */
function spellings(choice: Choice): string[] {
  return [choice.value, ...choice.aliases].map((s) => s.toLowerCase());
}

/** Whether a choice is in a list value like "air,ground" (case does not matter). */
export function inList(raw: string, choice: Choice): boolean {
  const wanted = spellings(choice);
  return items(raw).some((item) => wanted.includes(item.toLowerCase()));
}

/**
 * The list value with one choice added or removed. Other items stay as they
 * are, including spellings the editor does not know.
 */
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
