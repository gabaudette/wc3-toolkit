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

export type DamageBonus = {
  amount: number;
  attribute: string;
};

export type UnitDetail = {
  item: UnitItem;
  summary: Stat[];
  sections: Section[];
  damageBonus: DamageBonus | null;
};

/** Two or more buttons in one menu with the same hotkey. */
export type HotkeyClash = {
  key: string;
  menu: string;
  names: string[];
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

/** Objects that move between maps: custom units and what they use. */
export type TransferKind = "unit" | "ability" | "buff";

export type TransferObject = {
  kind: TransferKind;
  id: string;
  /** The ID it gets in this map, when importing. */
  newId: string | null;
  base: string;
  name: string;
  icon: string | null;
  /** Brought along because a chosen unit uses it. */
  dependency: boolean;
};

export type SharePreview = {
  code: string;
  objects: TransferObject[];
  warnings: string[];
};

export type ImportPreview = {
  objects: TransferObject[];
  warnings: string[];
};

export type ImportSource =
  | { kind: "code"; code: string }
  | { kind: "map"; path: string; ids: string[] };
