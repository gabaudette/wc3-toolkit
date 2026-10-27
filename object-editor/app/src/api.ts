// The only module that talks to the Rust backend.

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  AbilityCard,
  AbilityDetail,
  AbilitySummary,
  Catalog,
  Design,
  DesignTransfer,
  Field,
  GameSummary,
  HotkeyClash,
  IconChoice,
  ImportPreview,
  ImportSource,
  LeveledCatalog,
  MapSummary,
  SearchHit,
  SharePreview,
  SpellFit,
  SpellKind,
  SpellTemplate,
  TemplateValues,
  TestCopy,
  TransferObject,
  UnitDetail,
  UnitItem,
} from "./types";

const ICON_SCHEME = "wc3icon";

/** Errors from commands arrive as plain messages. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "Something went wrong.";
}

export function iconUrl(gamePath: string): string {
  return convertFileSrc(gamePath, ICON_SCHEME);
}

export const api = {
  detectGameDir: () => invoke<string | null>("detect_game_dir"),
  openGame: (dir: string, hd: boolean) => invoke<GameSummary>("open_game", { dir, hd }),
  openMap: (path: string) => invoke<MapSummary>("open_map", { path }),
  mapStatus: () => invoke<MapSummary>("map_status"),
  listUnits: () => invoke<UnitItem[]>("list_units"),
  unitDetail: (id: string) => invoke<UnitDetail>("unit_detail", { id }),
  searchUnits: (query: string) => invoke<SearchHit[]>("search_units", { query }),
  listAbilities: () => invoke<AbilitySummary[]>("list_abilities"),
  abilityCards: (ids: string[]) => invoke<AbilityCard[]>("ability_cards", { ids }),
  abilityDetail: (id: string) => invoke<AbilityDetail>("ability_detail", { id }),
  searchAbilities: (query: string) => invoke<SearchHit[]>("search_abilities", { query }),
  /** `level` is 0 for fields without levels. */
  setAbilityField: (id: string, field: string, level: number, value: string) =>
    invoke<AbilityDetail>("set_ability_field", { id, field, level, value }),
  resetAbilityField: (id: string, field: string, level: number) =>
    invoke<AbilityDetail>("reset_ability_field", { id, field, level }),
  createAbility: (base: string) => invoke<AbilitySummary>("create_ability", { base }),
  spellKinds: () => invoke<SpellKind[]>("spell_kinds"),
  /** Icons for a spell's button: the map's imported images first, then the game's. */
  spellIcons: () => invoke<IconChoice[]>("spell_icons"),
  /** The units, heroes and buildings that have the spell, the map's first. */
  spellUnits: (spell: string) => invoke<string[]>("spell_units", { spell }),
  spellFit: (spell: string, unit: string) => invoke<SpellFit>("spell_fit", { spell, unit }),
  giveSpell: (spell: string, unit: string) => invoke<SpellFit>("give_spell", { spell, unit }),
  takeSpell: (spell: string, unit: string) => invoke<SpellFit>("take_spell", { spell, unit }),
  spellTemplates: () => invoke<SpellTemplate[]>("spell_templates"),
  /** Creates a copy of Channel set up for the template: name, button, costs, tooltips. */
  createTriggeredSpell: (template: string) =>
    invoke<AbilitySummary>("create_triggered_spell", { template }),
  /** Rewrites the spell's tooltips for new values. */
  setTriggeredValues: (spell: string, template: string, values: TemplateValues) =>
    invoke<AbilityDetail>("set_triggered_values", { spell, template, values }),
  triggeredSpellLua: (spell: string, template: string, values: TemplateValues) =>
    invoke<string>("triggered_spell_lua", { spell, template, values }),
  /** Writes the spell's Lua module into `dir` and returns the file's path. */
  writeTriggeredSpellLua: (spell: string, template: string, values: TemplateValues, dir: string) =>
    invoke<string>("write_triggered_spell_lua", { spell, template, values, dir }),
  defaultLuaFolder: () => invoke<string | null>("default_lua_folder"),
  /** Builds the saved map's `[DEV]` copy with the modules in `dir`. */
  buildTestCopy: (dir: string) => invoke<TestCopy>("build_test_copy", { dir }),
  /** The Interface tab's designs, from `<map> UI.json`. */
  uiDesigns: () => invoke<Design[]>("ui_designs"),
  /** Replaces the designs; they are written with the map on Save. */
  setUiDesigns: (designs: Design[]) => invoke<void>("set_ui_designs", { designs }),
  uiLua: (id: string) => invoke<string>("ui_lua", { id }),
  /** Writes every design's Lua module into `dir`; returns the file names. */
  writeUiLua: (dir: string) => invoke<string[]>("write_ui_lua", { dir }),
  uiExportPreview: (ids: string[]) => invoke<DesignTransfer>("ui_export_preview", { ids }),
  /** Saves the designs, the dialogs they show and the map's files they use as a `.w3ui` file. */
  saveUiFile: (ids: string[], path: string) => invoke<void>("save_ui_file", { ids, path }),
  uiImportPreview: (path: string) => invoke<DesignTransfer>("ui_import_preview", { path }),
  /** Adds a design file's designs to the map; returns the map's designs. */
  importUiFile: (path: string) => invoke<Design[]>("import_ui_file", { path }),
  /** Removes a custom ability the spell wizard created, when it is cancelled. */
  discardSpell: (id: string) => invoke<void>("discard_spell", { id }),
  fillPlaceholders: (text: string) => invoke<string>("fill_placeholders", { text }),
  hotkeyClashes: (id: string) => invoke<HotkeyClash[]>("hotkey_clashes", { id }),
  setField: (id: string, field: string, value: string) =>
    invoke<Field>("set_field", { id, field, value }),
  resetField: (id: string, field: string) => invoke<Field>("reset_field", { id, field }),
  createUnit: (base: string) => invoke<UnitItem>("create_unit", { base }),
  /** Items, buffs and destructables: objects without levels, on the unit page. */
  listPlain: (catalog: Catalog) => invoke<UnitItem[]>("list_plain", { catalog }),
  plainDetail: (catalog: Catalog, id: string) =>
    invoke<UnitDetail>("plain_detail", { catalog, id }),
  searchPlain: (catalog: Catalog, query: string) =>
    invoke<SearchHit[]>("search_plain", { catalog, query }),
  setPlainField: (catalog: Catalog, id: string, field: string, value: string) =>
    invoke<Field>("set_plain_field", { catalog, id, field, value }),
  resetPlainField: (catalog: Catalog, id: string, field: string) =>
    invoke<Field>("reset_plain_field", { catalog, id, field }),
  createPlain: (catalog: Catalog, base: string) =>
    invoke<UnitItem>("create_plain", { catalog, base }),
  /** Removes a custom object, such as a buff the spell wizard created when it is cancelled. */
  discardPlain: (catalog: Catalog, id: string) => invoke<void>("discard_plain", { catalog, id }),
  listLeveled: (catalog: LeveledCatalog) => invoke<UnitItem[]>("list_leveled", { catalog }),
  leveledDetail: (catalog: LeveledCatalog, id: string) =>
    invoke<AbilityDetail>("leveled_detail", { catalog, id }),
  searchLeveled: (catalog: LeveledCatalog, query: string) =>
    invoke<SearchHit[]>("search_leveled", { catalog, query }),
  /** `level` is 0 for fields without levels. */
  setLeveledField: (
    catalog: LeveledCatalog,
    id: string,
    field: string,
    level: number,
    value: string,
  ) => invoke<AbilityDetail>("set_leveled_field", { catalog, id, field, level, value }),
  resetLeveledField: (catalog: LeveledCatalog, id: string, field: string, level: number) =>
    invoke<AbilityDetail>("reset_leveled_field", { catalog, id, field, level }),
  createLeveled: (catalog: LeveledCatalog, base: string) =>
    invoke<UnitItem>("create_leveled", { catalog, base }),
  saveMap: () => invoke<MapSummary>("save_map"),
  shareCode: (ids: string[]) => invoke<SharePreview>("share_code", { ids }),
  saveBundle: (ids: string[], path: string) => invoke<void>("save_bundle", { ids, path }),
  mapCustomUnits: (path: string) => invoke<UnitItem[]>("map_custom_units", { path }),
  previewImport: (source: ImportSource) => invoke<ImportPreview>("preview_import", { source }),
  importObjects: (source: ImportSource) => invoke<TransferObject[]>("import_objects", { source }),
};
