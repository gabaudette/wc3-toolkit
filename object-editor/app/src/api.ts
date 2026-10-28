import { convertFileSrc, type InvokeArgs, invoke } from "@tauri-apps/api/core";
import { attempt, type Result } from "./result";
import type {
  AbilityCard,
  AbilityDetail,
  AbilitySummary,
  AssetList,
  Catalog,
  ConstantsView,
  Design,
  DesignTransfer,
  DropsView,
  Field,
  GameSummary,
  HotkeyClash,
  IconChoice,
  ImportCandidate,
  ImportPreview,
  ImportRequest,
  ImportSource,
  ItemChoice,
  ItemTableInput,
  ItemTableList,
  LeveledCatalog,
  MapInfoView,
  MapSummary,
  PlacedList,
  PropertiesView,
  SearchHit,
  SharePreview,
  SpellFit,
  SpellKind,
  SpellTemplate,
  TemplateValues,
  TestCopy,
  TextureStatus,
  TransferObject,
  UnitDetail,
  UnitItem,
} from "./types";

const ICON_SCHEME = "wc3icon";

export type LogLevel = "warn" | "error";

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
  setAbilityField: (id: string, field: string, level: number, value: string) =>
    call<AbilityDetail>("set_ability_field", { id, field, level, value }),
  resetAbilityField: (id: string, field: string, level: number) =>
    call<AbilityDetail>("reset_ability_field", { id, field, level }),
  createAbility: (base: string) => call<AbilitySummary>("create_ability", { base }),
  spellKinds: () => call<SpellKind[]>("spell_kinds"),
  spellIcons: () => call<IconChoice[]>("spell_icons"),
  spellUnits: (spell: string) => call<string[]>("spell_units", { spell }),
  spellFit: (spell: string, unit: string) => call<SpellFit>("spell_fit", { spell, unit }),
  giveSpell: (spell: string, unit: string) => call<SpellFit>("give_spell", { spell, unit }),
  takeSpell: (spell: string, unit: string) => call<SpellFit>("take_spell", { spell, unit }),
  spellTemplates: () => call<SpellTemplate[]>("spell_templates"),
  createTriggeredSpell: (template: string) =>
    call<AbilitySummary>("create_triggered_spell", { template }),
  setTriggeredValues: (spell: string, template: string, values: TemplateValues) =>
    call<AbilityDetail>("set_triggered_values", { spell, template, values }),
  triggeredSpellLua: (spell: string, template: string, values: TemplateValues) =>
    call<string>("triggered_spell_lua", { spell, template, values }),
  writeTriggeredSpellLua: (spell: string, template: string, values: TemplateValues, dir: string) =>
    call<string>("write_triggered_spell_lua", { spell, template, values, dir }),
  defaultLuaFolder: () => call<string | null>("default_lua_folder"),
  buildTestCopy: (dir: string) => call<TestCopy>("build_test_copy", { dir }),
  uiDesigns: () => call<Design[]>("ui_designs"),
  setUiDesigns: (designs: Design[]) => call<void>("set_ui_designs", { designs }),
  uiLua: (id: string) => call<string>("ui_lua", { id }),
  writeUiLua: (dir: string) => call<string[]>("write_ui_lua", { dir }),
  uiExportPreview: (ids: string[]) => call<DesignTransfer>("ui_export_preview", { ids }),
  saveUiFile: (ids: string[], path: string) => call<void>("save_ui_file", { ids, path }),
  uiImportPreview: (path: string) => call<DesignTransfer>("ui_import_preview", { path }),
  importUiFile: (path: string) => call<Design[]>("import_ui_file", { path }),
  discardSpell: (id: string) => call<void>("discard_spell", { id }),
  fillPlaceholders: (text: string) => call<string>("fill_placeholders", { text }),
  hotkeyClashes: (id: string) => call<HotkeyClash[]>("hotkey_clashes", { id }),
  setField: (id: string, field: string, value: string) =>
    call<Field>("set_field", { id, field, value }),
  resetField: (id: string, field: string) => call<Field>("reset_field", { id, field }),
  createUnit: (base: string) => call<UnitItem>("create_unit", { base }),
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
  discardPlain: (catalog: Catalog, id: string) => call<void>("discard_plain", { catalog, id }),
  listLeveled: (catalog: LeveledCatalog) => call<UnitItem[]>("list_leveled", { catalog }),
  leveledDetail: (catalog: LeveledCatalog, id: string) =>
    call<AbilityDetail>("leveled_detail", { catalog, id }),
  searchLeveled: (catalog: LeveledCatalog, query: string) =>
    call<SearchHit[]>("search_leveled", { catalog, query }),
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
  mapInfo: () => call<MapInfoView>("map_info"),
  setMapInfo: (properties: PropertiesView) => call<MapSummary>("set_map_info", { properties }),
  gameplayConstants: () => call<ConstantsView>("gameplay_constants"),
  setGameplayConstant: (id: string, value: string | null) =>
    call<MapSummary>("set_gameplay_constant", { id, value }),
  setCustomConstants: (custom: boolean) => call<MapSummary>("set_custom_constants", { custom }),
  listPlacedUnits: () => call<PlacedList>("list_placed_units"),
  setPlacedInventory: (index: number, items: (string | null)[]) =>
    call<MapSummary>("set_placed_inventory", { index, items }),
  setPlacedDrops: (index: number, drops: DropsView) =>
    call<MapSummary>("set_placed_drops", { index, drops }),
  listItemTables: () => call<ItemTableList>("list_item_tables"),
  saveItemTable: (table: ItemTableInput) => call<number>("save_item_table", { table }),
  removeItemTable: (number: number) => call<void>("remove_item_table", { number }),
  itemChoices: () => call<ItemChoice[]>("item_choices"),
  listAssets: () => call<AssetList>("list_assets"),
  assetTextures: (path: string) => call<TextureStatus[]>("asset_textures", { path }),
  planImport: (files: string[], folder: string | null) =>
    call<ImportCandidate[]>("plan_import", { files, folder }),
  readTextFile: (path: string) => call<string>("read_text_file", { path }),
  importAssets: (files: ImportRequest[]) => call<void>("import_assets", { files }),
  removeAsset: (path: string) => call<void>("remove_asset", { path }),
  replaceAsset: (path: string, diskPath: string) => call<void>("replace_asset", { path, diskPath }),
  renameAsset: (path: string, newPath: string) => call<number>("rename_asset", { path, newPath }),
  exportAsset: (path: string, dest: string, png: boolean) =>
    call<void>("export_asset", { path, dest, png }),
  exportGameFile: (path: string, dest: string, png: boolean) =>
    call<void>("export_game_file", { path, dest, png }),
  importObjects: (source: ImportSource) => call<TransferObject[]>("import_objects", { source }),
};
