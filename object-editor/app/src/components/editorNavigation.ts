import { useCallback, useMemo, useState } from "react";
import type { AssetUse, AssetUseKind, Catalog, LeveledCatalog } from "../types";
import type { Tab } from "./EditorTabs";
import {
  BUFF_LIST,
  DESTRUCTABLE_LIST,
  DOODAD_LIST,
  ITEM_LIST,
  type ListConfig,
  UPGRADE_LIST,
} from "./UnitList";

export const LEVELED_TABS: { catalog: LeveledCatalog; list: ListConfig }[] = [
  { catalog: "upgrades", list: UPGRADE_LIST },
  { catalog: "doodads", list: DOODAD_LIST },
];

export const PLAIN_TABS: { catalog: Catalog; list: ListConfig }[] = [
  { catalog: "items", list: ITEM_LIST },
  { catalog: "buffs", list: BUFF_LIST },
  { catalog: "destructables", list: DESTRUCTABLE_LIST },
];

const PLAIN_USES: Partial<Record<AssetUseKind, Catalog>> = {
  item: "items",
  buff: "buffs",
  destructable: "destructables",
};

const LEVELED_USES: Partial<Record<AssetUseKind, LeveledCatalog>> = {
  upgrade: "upgrades",
  doodad: "doodads",
};

const NO_LEVELED_SELECTION: Record<LeveledCatalog, string | null> = {
  upgrades: null,
  doodads: null,
};

const NO_PLAIN_SELECTION: Record<Catalog, string | null> = {
  items: null,
  buffs: null,
  destructables: null,
};

export function ignoreSelection(): void {}

export function useNavigation(setTab: (tab: Tab) => void) {
  const [unit, setUnit] = useState<string | null>(null);
  const [ability, setAbility] = useState<string | null>(null);
  const [plain, setPlain] = useState(NO_PLAIN_SELECTION);
  const [leveled, setLeveled] = useState(NO_LEVELED_SELECTION);

  const openUnit = useCallback(
    (id: string) => {
      setUnit(id);
      setTab("units");
    },
    [setTab],
  );

  const openAbility = useCallback(
    (id: string) => {
      setAbility(id);
      setTab("abilities");
    },
    [setTab],
  );

  const openPlain = useCallback(
    (catalog: Catalog, id: string) => {
      setPlain((current) => ({ ...current, [catalog]: id }));
      setTab(catalog);
    },
    [setTab],
  );

  const openLeveled = useCallback(
    (catalog: LeveledCatalog, id: string) => {
      setLeveled((current) => ({ ...current, [catalog]: id }));
      setTab(catalog);
    },
    [setTab],
  );

  const selectPlain = useMemo(
    () =>
      new Map(PLAIN_TABS.map(({ catalog }) => [catalog, (id: string) => openPlain(catalog, id)])),
    [openPlain],
  );

  const selectLeveled = useMemo(
    () =>
      new Map(
        LEVELED_TABS.map(({ catalog }) => [catalog, (id: string) => openLeveled(catalog, id)]),
      ),
    [openLeveled],
  );

  function openUse(use: AssetUse) {
    const plainCatalog = PLAIN_USES[use.kind];
    const leveledCatalog = LEVELED_USES[use.kind];

    if (use.kind === "unit") {
      openUnit(use.id);
      return;
    }

    if (use.kind === "ability") {
      openAbility(use.id);
      return;
    }

    if (plainCatalog) {
      openPlain(plainCatalog, use.id);
      return;
    }

    if (leveledCatalog) {
      openLeveled(leveledCatalog, use.id);
    }
  }

  function clear() {
    setUnit(null);
    setAbility(null);
    setPlain(NO_PLAIN_SELECTION);
    setLeveled(NO_LEVELED_SELECTION);
  }

  return {
    unit,
    ability,
    plain,
    leveled,
    setUnit,
    setAbility,
    openUnit,
    openAbility,
    openPlain,
    openLeveled,
    openUse,
    selectPlain,
    selectLeveled,
    clear,
  };
}
