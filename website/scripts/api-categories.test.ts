import { expect, test } from "bun:test";
import { API_CATEGORIES, categoryFor } from "./api-categories";
import type { Declaration } from "./jass";

const api: { declarations: Declaration[] } = await Bun.file(
  new URL("../data/api.json", import.meta.url),
).json();

test.each([
  ["CreateUnit", "units-and-heroes"],
  ["CreateUnitAtLocSaveLast", "units-and-heroes"],
  ["UnitAddItem", "items-and-inventory"],
  ["SetUnitAbilityLevel", "abilities-and-upgrades"],
  ["BlzSetAbilityRealLevelField", "abilities-and-upgrades"],
  ["ABILITY_RLF_DAMAGE_HBZ2", "abilities-and-upgrades"],
  ["UnitDamageTarget", "combat"],
  ["SaveUnitHandle", "storage-and-sync"],
  ["LoadUnitHandle", "storage-and-sync"],
  ["RestoreUnit", "storage-and-sync"],
  ["BlzSendSyncData", "storage-and-sync"],
  ["GetTriggerUnit", "triggers-and-events"],
  ["GetEventDamage", "triggers-and-events"],
  ["TriggerRegisterTimerEvent", "triggers-and-events"],
  ["CreateTimerDialog", "timers"],
  ["GroupEnumUnitsInRange", "groups-and-forces"],
  ["SetPlayerAlliance", "players-and-alliances"],
  ["CreateQuest", "quests"],
  ["SetCameraPosition", "camera-and-cinematics"],
  ["CreateSound", "sound-and-music"],
  ["BlzCreateFrame", "ui-and-input"],
  ["BlzLoadTOCFile", "ui-and-input"],
  ["OSKEY_A", "ui-and-input"],
  ["CreateDestructable", "destructables"],
  ["AddSpecialEffect", "effects-and-animation"],
  ["SetTerrainType", "terrain-and-environment"],
  ["CreateFogModifierRect", "visibility-and-minimap"],
  ["Rect", "geometry"],
  ["SaveGame", "game-and-campaign"],
  ["SaveAndChangeLevelBJ", "game-and-campaign"],
  ["OrderId", "math-and-strings"],
  ["ConvertUnitState", "units-and-heroes"],
  ["GetHandleId", "runtime"],
])("%s is browsable under %s", (name, category) => {
  const declaration = api.declarations.find((entry) => entry.name === name);
  if (!declaration) {
    throw new Error(`Missing reference API used by the category test: ${name}`);
  }
  expect(categoryFor(declaration).id).toBe(category);
});

test("topic IDs and engine-type ownership are unambiguous", () => {
  const ids = API_CATEGORIES.map((category) => category.id);
  expect(new Set(ids).size).toBe(ids.length);
  const types = API_CATEGORIES.flatMap((category) => category.types);
  expect(new Set(types).size).toBe(types.length);
  for (const id of ids) {
    expect(id).toMatch(/^[a-z]+(?:-[a-z]+)*$/);
    expect(api.declarations.some((entry) => categoryFor(entry).id === id)).toBe(true);
  }
});

test("new symbols can use their engine signature or the general utility topic", () => {
  const declaration: Declaration = {
    name: "FutureOperation",
    kind: "native",
    source: "common.j",
    line: 1,
    parameters: [],
    returns: "nothing",
  };
  expect(categoryFor(declaration).id).toBe("runtime");
  expect(categoryFor({ ...declaration, returns: "unit" }).id).toBe("units-and-heroes");
  expect(
    categoryFor({ ...declaration, parameters: [{ name: "table", type: "hashtable" }] }).id,
  ).toBe("storage-and-sync");
});
