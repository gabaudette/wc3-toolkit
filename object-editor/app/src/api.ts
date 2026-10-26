// The only module that talks to the Rust backend.

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  AbilityCard,
  AbilityDetail,
  AbilitySummary,
  Catalog,
  Field,
  GameSummary,
  HotkeyClash,
  ImportPreview,
  ImportSource,
  LeveledCatalog,
  MapSummary,
  SearchHit,
  SharePreview,
  SpellKind,
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
