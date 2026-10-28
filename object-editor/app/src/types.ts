export type UnitKind =
  | "hero"
  | "unit"
  | "building"
  | "item"
  | "upgrade"
  | "buff"
  | "destructable"
  | "doodad";

export type LeveledCatalog = "upgrades" | "doodads";

export type Catalog = "items" | "buffs" | "destructables";

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
  aliases: string[];
};

export type Field = {
  id: string;
  name: string;
  label: string;
  display: string;
  raw: string;
  modified: boolean;
  base: string;
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
  base: string;
  name: string;
  suffix: string | null;
  icon: string | null;
  hero: boolean;
  item: boolean;
  race: string;
  levels: number;
  custom: boolean;
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

export type TemplateChoice = { label: string; lua: string };

export type TemplateParam = {
  id: string;
  label: string;
  values: number[];
  whole: boolean;
  choices: TemplateChoice[];
};

export type SpellTemplate = {
  id: string;
  label: string;
  description: string;
  file: string;
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

export type TemplateValues = Record<string, number[]>;

export type TestCopy = { path: string; modules: string[] };

export type CardButton = {
  name: string;
  icon: string | null;
  hotkey: string | null;
  x: number;
  y: number;
  spell: boolean;
};

export type SpellFit = {
  has: boolean;
  learned: boolean;
  card: CardButton[];
  learn: CardButton[];
  warnings: string[];
  blocked: string | null;
};

export type IconChoice = {
  path: string;
  name: string;
  fromMap: boolean;
};

export type SpellBase = {
  card: AbilityCard;
  why: string;
};

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

export type TransferKind = "unit" | "ability" | "buff" | "item" | "upgrade";

export type TransferObject = {
  kind: TransferKind;
  id: string;
  newId: string | null;
  base: string;
  name: string;
  icon: string | null;
  dependency: boolean;
};

export type FileStatus = "new" | "same" | "different";

export type TransferFile = {
  path: string;
  size: number;
  status: FileStatus | null;
};

export type SharePreview = {
  code: string;
  objects: TransferObject[];
  files: TransferFile[];
  warnings: string[];
};

export type ImportPreview = {
  objects: TransferObject[];
  files: TransferFile[];
  warnings: string[];
};

export type ListedDesign = {
  name: string;
  kind: string;
  luaName: string;
  shownBy: string | null;
  renamedFrom: string | null;
};

export type DesignTransfer = {
  designs: ListedDesign[];
  files: TransferFile[];
  needs: string[];
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
  id: string;
  text: string;
  hotkey: string;
  highlightHotkey: boolean;
  action: ButtonAction;
};

export type DialogDesign = {
  kind: "dialog";
  id: string;
  name: string;
  title: string;
  titleColor: string | null;
  buttons: DialogButton[];
  showAtStart: boolean;
};

export type BoardCell = {
  text: string;
  color: string | null;
  icon: string | null;
};

export type BoardColumn = {
  id: string;
  name: string;
  width: number;
};

export type BoardRow = {
  id: string;
  name: string;
  perPlayer: boolean;
  cells: BoardCell[];
};

export type MultiboardDesign = {
  kind: "multiboard";
  id: string;
  name: string;
  title: string;
  titleColor: string | null;
  columns: BoardColumn[];
  rows: BoardRow[];
  showAtStart: boolean;
  minimized: boolean;
};

export type LeaderboardLabel = {
  id: string;
  name: string;
  color: string | null;
  value: number;
};

export type LeaderboardDesign = {
  kind: "leaderboard";
  id: string;
  name: string;
  title: string;
  titleColor: string | null;
  players: boolean;
  playerValue: number;
  /** At most four: the game gives every row a player, and these use the neutral ones. */
  labels: LeaderboardLabel[];
  sort: "none" | "highFirst" | "lowFirst";
  showAtStart: boolean;
};

export type TimerWindowDesign = {
  kind: "timerWindow";
  id: string;
  name: string;
  title: string;
  titleColor: string | null;
  duration: number;
  repeats: boolean;
  startAtStart: boolean;
  hideWhenDone: boolean;
  onDone: string;
};

export type Design = DialogDesign | MultiboardDesign | LeaderboardDesign | TimerWindowDesign;

export type AssetKind = "model" | "image" | "other";

export type AssetUseKind =
  | "unit"
  | "ability"
  | "item"
  | "buff"
  | "destructable"
  | "upgrade"
  | "doodad";

export type AssetUse = { kind: AssetUseKind; id: string; name: string; field: string };

export type AssetItem = {
  path: string;
  kind: AssetKind;
  size: number;
  queued: boolean;
  uses: AssetUse[];
};

export type AssetList = { assets: AssetItem[]; listed: boolean };

export type TextureStatusKind = "batch" | "map" | "game" | "missing";

export type TextureStatus = { path: string; status: TextureStatusKind };

export type ImportCandidate = {
  diskPath: string;
  importPath: string;
  kind: AssetKind;
  size: number;
  convert: boolean;
  exists: boolean;
  textures: TextureStatus[];
};

export type ImportRequest = { diskPath: string; importPath: string; converted?: string };
