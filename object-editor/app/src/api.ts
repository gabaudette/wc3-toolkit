// The only module that talks to the Rust backend.

import { convertFileSrc, type InvokeArgs, invoke } from "@tauri-apps/api/core";
import { attempt, type Result } from "./result";
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

export type LogLevel = "warn" | "error";

/** Writes a failure to the log file, once, from the layer that understands it (AGE-ERR-004). */
export function report(level: LogLevel, source: string, message: string): void {
  void attempt(() => invoke("log_failure", { level, source, message }));
}

async function call<T>(command: string, args?: InvokeArgs): Promise<Result<T>> {
  const result = await attempt(() => invoke<T>(command, args));

  if (!result.ok) {
    report("warn", command, result.error);
  }

  return result;
}

export function iconUrl(gamePath: string): string {
  return convertFileSrc(gamePath, ICON_SCHEME);
}

export const api = {
  detectGameDir: () => call<string | null>("detect_game_dir"),
  openGame: (dir: string, hd: boolean) => call<GameSummary>("open_game", { dir, hd }),
  openMap: (path: string) => call<MapSummary>("open_map", { path }),
  mapStatus: () => call<MapSummary>("map_status"),
  listUnits: () => call<UnitItem[]>("list_units"),
  unitDetail: (id: string) => call<UnitDetail>("unit_detail", { id }),
  searchUnits: (query: string) => call<SearchHit[]>("search_units", { query }),
  listAbilities: () => call<AbilitySummary[]>("list_abilities"),
  abilityCards: (ids: string[]) => call<AbilityCard[]>("ability_cards", { ids }),
  abilityDetail: (id: string) => call<AbilityDetail>("ability_detail", { id }),
  searchAbilities: (query: string) => call<SearchHit[]>("search_abilities", { query }),
  /** `level` is 0 for fields without levels. */
  setAbilityField: (id: string, field: string, level: number, value: string) =>
    call<AbilityDetail>("set_ability_field", { id, field, level, value }),
  resetAbilityField: (id: string, field: string, level: number) =>
    call<AbilityDetail>("reset_ability_field", { id, field, level }),
  createAbility: (base: string) => call<AbilitySummary>("create_ability", { base }),
  spellKinds: () => call<SpellKind[]>("spell_kinds"),
  /** Icons for a spell's button: the map's imported images first, then the game's. */
  spellIcons: () => call<IconChoice[]>("spell_icons"),
  /** The units, heroes and buildings that have the spell, the map's first. */
  spellUnits: (spell: string) => call<string[]>("spell_units", { spell }),
  spellFit: (spell: string, unit: string) => call<SpellFit>("spell_fit", { spell, unit }),
  giveSpell: (spell: string, unit: string) => call<SpellFit>("give_spell", { spell, unit }),
  takeSpell: (spell: string, unit: string) => call<SpellFit>("take_spell", { spell, unit }),
  spellTemplates: () => call<SpellTemplate[]>("spell_templates"),
  /** Creates a copy of Channel set up for the template: name, button, costs, tooltips. */
  createTriggeredSpell: (template: string) =>
    call<AbilitySummary>("create_triggered_spell", { template }),
  /** Rewrites the spell's tooltips for new values. */
  setTriggeredValues: (spell: string, template: string, values: TemplateValues) =>
    call<AbilityDetail>("set_triggered_values", { spell, template, values }),
  triggeredSpellLua: (spell: string, template: string, values: TemplateValues) =>
    call<string>("triggered_spell_lua", { spell, template, values }),
  /** Writes the spell's Lua module into `dir` and returns the file's path. */
  writeTriggeredSpellLua: (spell: string, template: string, values: TemplateValues, dir: string) =>
    call<string>("write_triggered_spell_lua", { spell, template, values, dir }),
  defaultLuaFolder: () => call<string | null>("default_lua_folder"),
  /** Builds the saved map's `[DEV]` copy with the modules in `dir`. */
  buildTestCopy: (dir: string) => call<TestCopy>("build_test_copy", { dir }),
  /** The Interface tab's designs, from `<map> UI.json`. */
  uiDesigns: () => call<Design[]>("ui_designs"),
  /** Replaces the designs; they are written with the map on Save. */
  setUiDesigns: (designs: Design[]) => call<void>("set_ui_designs", { designs }),
  uiLua: (id: string) => call<string>("ui_lua", { id }),
  /** Writes every design's Lua module into `dir`; returns the file names. */
  writeUiLua: (dir: string) => call<string[]>("write_ui_lua", { dir }),
  uiExportPreview: (ids: string[]) => call<DesignTransfer>("ui_export_preview", { ids }),
  /** Saves the designs, the dialogs they show and the map's files they use as a `.w3ui` file. */
  saveUiFile: (ids: string[], path: string) => call<void>("save_ui_file", { ids, path }),
  uiImportPreview: (path: string) => call<DesignTransfer>("ui_import_preview", { path }),
  /** Adds a design file's designs to the map; returns the map's designs. */
  importUiFile: (path: string) => call<Design[]>("import_ui_file", { path }),
  /** Removes a custom ability the spell wizard created, when it is cancelled. */
  discardSpell: (id: string) => call<void>("discard_spell", { id }),
  fillPlaceholders: (text: string) => call<string>("fill_placeholders", { text }),
  hotkeyClashes: (id: string) => call<HotkeyClash[]>("hotkey_clashes", { id }),
  setField: (id: string, field: string, value: string) =>
    call<Field>("set_field", { id, field, value }),
  resetField: (id: string, field: string) => call<Field>("reset_field", { id, field }),
  createUnit: (base: string) => call<UnitItem>("create_unit", { base }),
  /** Items, buffs and destructables: objects without levels, on the unit page. */
  listPlain: (catalog: Catalog) => call<UnitItem[]>("list_plain", { catalog }),
  plainDetail: (catalog: Catalog, id: string) => call<UnitDetail>("plain_detail", { catalog, id }),
  searchPlain: (catalog: Catalog, query: string) =>
    call<SearchHit[]>("search_plain", { catalog, query }),
  setPlainField: (catalog: Catalog, id: string, field: string, value: string) =>
    call<Field>("set_plain_field", { catalog, id, field, value }),
  resetPlainField: (catalog: Catalog, id: string, field: string) =>
    call<Field>("reset_plain_field", { catalog, id, field }),
  createPlain: (catalog: Catalog, base: string) =>
    call<UnitItem>("create_plain", { catalog, base }),
  /** Removes a custom object, such as a buff the spell wizard created when it is cancelled. */
  discardPlain: (catalog: Catalog, id: string) => call<void>("discard_plain", { catalog, id }),
  listLeveled: (catalog: LeveledCatalog) => call<UnitItem[]>("list_leveled", { catalog }),
  leveledDetail: (catalog: LeveledCatalog, id: string) =>
    call<AbilityDetail>("leveled_detail", { catalog, id }),
  searchLeveled: (catalog: LeveledCatalog, query: string) =>
    call<SearchHit[]>("search_leveled", { catalog, query }),
  /** `level` is 0 for fields without levels. */
  setLeveledField: (
    catalog: LeveledCatalog,
    id: string,
    field: string,
    level: number,
    value: string,
  ) => call<AbilityDetail>("set_leveled_field", { catalog, id, field, level, value }),
  resetLeveledField: (catalog: LeveledCatalog, id: string, field: string, level: number) =>
    call<AbilityDetail>("reset_leveled_field", { catalog, id, field, level }),
  createLeveled: (catalog: LeveledCatalog, base: string) =>
    call<UnitItem>("create_leveled", { catalog, base }),
  saveMap: () => call<MapSummary>("save_map"),
  shareCode: (ids: string[]) => call<SharePreview>("share_code", { ids }),
  saveBundle: (ids: string[], path: string) => call<void>("save_bundle", { ids, path }),
  mapCustomUnits: (path: string) => call<UnitItem[]>("map_custom_units", { path }),
  previewImport: (source: ImportSource) => call<ImportPreview>("preview_import", { source }),
  importObjects: (source: ImportSource) => call<TransferObject[]>("import_objects", { source }),
};
