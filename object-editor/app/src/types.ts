// Mirrors the Rust types in src-tauri/src/view.rs and commands.rs.

export type UnitKind = "hero" | "unit" | "building" | "item";

export type UnitGroup = "map" | "melee" | "campaign" | "special" | "hidden";

export type UnitItem = {
  id: string;
  base: string;
  name: string;
  kind: UnitKind;
  race: string;
  group: UnitGroup;
  custom: boolean;
  changedFields: number;
  icon: string | null;
};

export type FieldControl =
  | "bool"
  | "number"
  | "text"
  | "longText"
  | "select"
  | "codes"
  | "abilities";

export type Choice = {
  value: string;
  label: string;
};

export type Field = {
  id: string;
  name: string;
  label: string;
  display: string;
  raw: string;
  modified: boolean;
  valueType: string;
  control: FieldControl;
  options: Choice[];
  min: number | null;
  max: number | null;
  integer: boolean;
  description: string | null;
  advanced: boolean;
  unverified: boolean;
  preview: boolean;
};

export type Section = {
  category: string;
  label: string;
  fields: Field[];
};

export type Stat = {
  label: string;
  value: string;
};

export type UnitDetail = {
  item: UnitItem;
  summary: Stat[];
  sections: Section[];
};

export type SearchHit = {
  id: string;
  reason: string;
};

export type GameSummary = {
  dir: string;
  units: number;
  mapsDir: string | null;
};

export type MapSummary = {
  path: string;
  name: string;
  unsaved: boolean;
};

export type AbilitySummary = {
  id: string;
  name: string;
  suffix: string | null;
  icon: string | null;
  hero: boolean;
  item: boolean;
  race: string;
  levels: number;
};

export type LevelRow = {
  label: string;
  values: string[];
};

export type AbilityCard = {
  summary: AbilitySummary;
  known: boolean;
  hotkey: string | null;
  requiredLevel: number;
  description: string | null;
  rows: LevelRow[];
};
