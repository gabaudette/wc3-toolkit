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
  /** The base object's stored value, as `raw` is. */
  baseRaw: string;
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

/** A setting of a triggered spell's choice parameter, such as "strength". */
export type TemplateChoice = { label: string; lua: string };

/** A value a triggered spell's Lua module uses, edited in the wizard. */
export type TemplateParam = {
  id: string;
  label: string;
  /** The default for each level; empty for a choice. */
  values: number[];
  /** Only whole numbers make sense (counts). */
  whole: boolean;
  /** One setting for every level instead of values; the first is the default. */
  choices: TemplateChoice[];
};

/** A triggered spell: Channel plus a Lua module made from a template. */
export type SpellTemplate = {
  id: string;
  label: string;
  description: string;
  file: string;
  /** Channel's target type. */
  target: "none" | "unit" | "point";
  targets: string;
  icon: string;
  hotkey: string;
  range: number;
  area: number;
  cooldown: number;
  mana: number;
  tooltip: string;
  params: TemplateParam[];
};

/** Each parameter's value per level, or for a choice the chosen index alone. */
export type TemplateValues = Record<string, number[]>;

/** The `[DEV]` copy of the map the harness built. */
export type TestCopy = { path: string; modules: string[] };

/** A button on a command card or learn menu preview. */
export type CardButton = {
  name: string;
  icon: string | null;
  hotkey: string | null;
  x: number;
  y: number;
  /** The spell being placed. */
  spell: boolean;
};

/** How a spell sits, or would sit, on a unit's command card and learn menu. */
export type SpellFit = {
  /** The unit already has the spell. */
  has: boolean;
  /** The spell goes in the hero's learnable abilities rather than the normal ones. */
  learned: boolean;
  card: CardButton[];
  /** Empty unless the spell is learned. */
  learn: CardButton[];
  warnings: string[];
  /** Why the spell cannot be given, when it cannot (a hero with five hero spells). */
  blocked: string | null;
};

/** An icon the spell wizard offers for a spell's button. */
export type IconChoice = {
  /** As data files name it, e.g. `ReplaceableTextures\\CommandButtons\\BTNStormBolt.blp`. */
  path: string;
  /** Its file name without folders or extension, for searching. */
  name: string;
  /** Imported into the open map rather than from the game. */
  fromMap: boolean;
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

/** What a dialog button does once clicked; the game hides the dialog first. */
export type ButtonAction =
  | { kind: "close" }
  | { kind: "showDialog"; dialog: string }
  | { kind: "runFunction"; function: string };

export type DialogButton = {
  text: string;
  /** One letter or digit, or empty for none. */
  hotkey: string;
  /** The hotkey's letter in the text turns gold, as in the game's own dialogs. */
  highlightHotkey: boolean;
  action: ButtonAction;
};

/** A dialog designed in the Interface tab: a message and a column of buttons. */
export type DialogDesign = {
  kind: "dialog";
  /** Stable, for references between designs. */
  id: string;
  /** The name in the tab; the Lua name (`UI.ChoosePath`) comes from it. */
  name: string;
  /** The message at the top, as plain text. */
  title: string;
  /** The message's color as `RRGGBB`; the game's own gold when null. */
  titleColor: string | null;
  buttons: DialogButton[];
  /** Shown to every player shortly after the game starts. */
  showAtStart: boolean;
};

/** A cell of a multiboard. */
export type BoardCell = {
  /** On a player's row, `{player}` is that player's name in their color. */
  text: string;
  /** `RRGGBB`, or null for white. */
  color: string | null;
  icon: string | null;
};

export type BoardColumn = {
  /** How Lua names it: `SetValue(row, "Kills", 5)`. */
  name: string;
  /** Width in percent of the screen's width. */
  width: number;
};

export type BoardRow = {
  /** How Lua names a fixed row; rows per player are named by the player. */
  name: string;
  /** One row for every playing player, in player order, instead of one. */
  perPlayer: boolean;
  /** One per column. */
  cells: BoardCell[];
};

/** A multiboard: a titled board of rows and columns in the top right corner. */
export type MultiboardDesign = {
  kind: "multiboard";
  id: string;
  name: string;
  title: string;
  /** `RRGGBB`, or null for the game's own color. */
  titleColor: string | null;
  columns: BoardColumn[];
  rows: BoardRow[];
  showAtStart: boolean;
  /** Shown folded to its title bar. */
  minimized: boolean;
};

export type Design = DialogDesign | MultiboardDesign;
