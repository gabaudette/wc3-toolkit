// The only module that talks to the Rust backend.

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { Field, GameSummary, MapSummary, SearchHit, UnitDetail, UnitItem } from "./types";

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
  setField: (id: string, field: string, value: string) =>
    invoke<Field>("set_field", { id, field, value }),
  resetField: (id: string, field: string) => invoke<Field>("reset_field", { id, field }),
  createUnit: (base: string) => invoke<UnitItem>("create_unit", { base }),
  saveMap: () => invoke<MapSummary>("save_map"),
};
