import { createContext } from "react";
import type { ObjectType } from "../types";

export type ExportScope = {
  type: ObjectType | null;
  shown: string[];
  selected: { id: string; name: string } | null;
};

export const ExportContext = createContext<((scope: ExportScope) => void) | null>(null);

export const OBJECT_TYPES: { type: ObjectType; singular: string }[] = [
  { type: "units", singular: "unit" },
  { type: "items", singular: "item" },
  { type: "buffs", singular: "buff" },
  { type: "destructables", singular: "destructable" },
  { type: "abilities", singular: "ability" },
  { type: "upgrades", singular: "upgrade" },
  { type: "doodads", singular: "doodad" },
];

export function shownCount(count: number, type: ObjectType): string {
  const singular = OBJECT_TYPES.find((t) => t.type === type)?.singular ?? type;

  if (count === 1) {
    return `1 ${singular}`;
  }

  return `${count} ${type}`;
}
