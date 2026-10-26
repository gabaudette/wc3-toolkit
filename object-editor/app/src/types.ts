// Mirrors the Rust types in src-tauri/src/view.rs and commands.rs.

export type UnitKind =
  | "hero"
  | "unit"
  | "building"
  | "item"
  | "upgrade"
  | "buff"
  | "destructable"
  | "doodad";

/** Objects with a value per level (upgrades) or per variation (doodads), on the ability page. */
export type LeveledCatalog = "upgrades" | "doodads";

/** Objects without levels other than units, which share the unit page. */
export type Catalog = "items" | "buffs" | "destructables";

/** "game" holds the game's own items, which have no melee or campaign split. */
export type UnitGroup = "map" | "melee" | "campaign" | "special" | "hidden" | "game";

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
  | "flags"
  | "flagList"
  | "codes"
  | "abilities";

export type Choice = {
  value: string;
  label: string;
  /** Other spellings the game accepts, e.g. "invu" for "invulnerable". */
  aliases: string[];
};

export type Field = {
  id: string;
  name: string;
  label: string;
  display: string;
  raw: string;
  modified: boolean;
  /** The base object's value, formatted like `display`. */
  base: string;
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
  /** The game ability it copies; its own ID for game abilities. */
  base: string;
  name: string;
  suffix: string | null;
  icon: string | null;
  hero: boolean;
  item: boolean;
  race: string;
  levels: number;
  /** Added by the map (`A000`). */
  custom: boolean;
  /** Added or changed by the map. */
  inMap: boolean;
  changedFields: number;
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

export type LevelCell = {
  raw: string;
  display: string;
  modified: boolean;
};

/** A field with a value per level; `field` holds its names, control and limits. */
export type LevelField = {
  field: Field;
  cells: LevelCell[];
};

export type AbilitySection = {
  category: string;
  label: string;
  fields: Field[];
  levelFields: LevelField[];
};

/** A base ability the spell wizard suggests for a kind of spell. */
export type SpellBase = {
  card: AbilityCard;
  /** What it brings, and when to pick it over the others. */
  why: string;
};

/** What a spell does, as picked on the spell wizard's first step. */
export type SpellKind = {
  id: string;
  label: string;
  description: string;
  bases: SpellBase[];
};

export type AbilityDetail = {
  card: AbilityCard;
  levels: number;
  sections: AbilitySection[];
};

/** Objects that move between maps: custom units and what they use. */
export type TransferKind = "unit" | "ability" | "buff" | "item" | "upgrade";

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

/** What importing a file does: added, already in the map, or the map keeps its own. */
export type FileStatus = "new" | "same" | "different";

/** A file imported into a map (model, texture, icon) that travels with objects. */
export type TransferFile = {
  path: string;
  size: number;
  /** Set when importing. */
  status: FileStatus | null;
};

export type SharePreview = {
  code: string;
  objects: TransferObject[];
  /** In bundle files only; share codes leave them out. */
  files: TransferFile[];
  warnings: string[];
};

export type ImportPreview = {
  objects: TransferObject[];
  files: TransferFile[];
  warnings: string[];
};

export type ImportSource =
  | { kind: "code"; code: string }
  | { kind: "bundle"; path: string }
  | { kind: "map"; path: string; ids: string[] };
