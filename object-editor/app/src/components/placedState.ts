import { type KeyboardEvent, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import { confirm } from "../dialogs";
import { type DropSetDraft, setsOf, type TierDraft, tierDrafts, tiersOf } from "../loot";
import { type Result, settle } from "../result";
import type {
  DropsView,
  ItemChoice,
  ItemTableList,
  ItemTableView,
  MapView,
  PlacedList,
  PlacedUnit,
  PlacementEdit,
} from "../types";
import { counted } from "../ui";
import { plainGameText } from "./GameText";
import { type Placing, placementKey } from "./PlacedMap";

type ErrorHandler = (message: string | null) => void;

const NEW_TABLE_NAME = "Item table";
const NEW_CASCADE_NAME = "Loot";
const STARTER_TIERS = [
  { name: "Legendary", chance: 2, items: [] },
  { name: "Epic", chance: 5, items: [] },
  { name: "Rare", chance: 15, items: [] },
  { name: "Common", chance: 100, items: [] },
];

export function usePlacedData(editable: boolean, onMapChanged: () => void, onError: ErrorHandler) {
  const [placed, setPlaced] = useState<PlacedList | null>(null);
  const [tables, setTables] = useState<ItemTableList | null>(null);
  const [choices, setChoices] = useState<ItemChoice[]>([]);
  const [view, setView] = useState<MapView | null>(null);
  const [viewError, setViewError] = useState<string | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());

  const reloadUnits = useCallback(() => {
    void settle(api.listPlacedUnits(), setPlaced, onError);
  }, [onError]);

  const reloadTables = useCallback(() => {
    void settle(api.listItemTables(), setTables, onError);
  }, [onError]);

  useEffect(() => {
    if (!editable) {
      return;
    }

    reloadUnits();
    reloadTables();
    void settle(api.itemChoices(), setChoices, onError);
    void settle(api.mapView(), setView, setViewError);
  }, [editable, reloadUnits, reloadTables, onError]);

  function run<T>(edit: () => Promise<Result<T>>, done: (value: T) => void, after: () => void) {
    queue.current = queue.current.then(async () => {
      await settle(
        edit(),
        (value) => {
          onError(null);
          done(value);
        },
        onError,
      );
      after();
      onMapChanged();
    });
  }

  return { placed, tables, choices, view, viewError, reloadUnits, reloadTables, run };
}

export type PlacedData = ReturnType<typeof usePlacedData>;

async function confirmed(
  message: string,
  title: string,
  okLabel: string,
  onError: ErrorHandler,
): Promise<boolean> {
  const sure = await confirm(message, { title, kind: "warning", okLabel, cancelLabel: "Keep it" });

  if (!sure.ok) {
    onError(sure.error);
    return false;
  }

  return sure.value;
}

function ignore(): void {}

export function usePlacedUnits(data: PlacedData, onError: ErrorHandler) {
  const { placed, view, run, reloadUnits, reloadTables } = data;
  const [selected, setSelected] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const [heroesOnly, setHeroesOnly] = useState(false);
  const [withDrops, setWithDrops] = useState(false);
  const [placing, setPlacing] = useState<Placing | null>(null);
  const [owner, setOwner] = useState(0);
  const units = useMemo(() => placed?.units ?? [], [placed]);
  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();

    return units
      .filter((u) => !heroesOnly || u.hero)
      .filter((u) => !withDrops || u.drops.kind !== "none")
      .filter(
        (u) =>
          !needle ||
          u.name.toLowerCase().includes(needle) ||
          u.id.toLowerCase().includes(needle) ||
          u.playerName.toLowerCase().includes(needle),
      );
  }, [units, query, heroesOnly, withDrops]);

  useEffect(() => {
    if (selected !== null) {
      document.getElementById(`placed-${selected}`)?.scrollIntoView({ block: "nearest" });
    }
  }, [selected]);

  function edit(index: number, change: PlacementEdit) {
    run(() => api.editPlacedUnit(index, change), ignore, reloadUnits);
  }

  function setInventory(target: PlacedUnit, items: (string | null)[]) {
    run(() => api.setPlacedInventory(target.index, items), ignore, reloadUnits);
  }

  function setDrops(target: PlacedUnit, drops: DropsView) {
    run(
      () => api.setPlacedDrops(target.index, drops),
      ignore,
      () => {
        reloadUnits();
        reloadTables();
      },
    );
  }

  function pickOnMap(index: number) {
    setSelected(index);
    document
      .getElementById(`placed-${index}`)
      ?.querySelector("button")
      ?.focus({ preventScroll: true });
  }

  function place(x: number, y: number) {
    if (placing) {
      run(() => api.addPlacedUnit(placing.id, owner, x, y), setSelected, reloadUnits);
    }
  }

  async function remove(index: number) {
    const target = units.find((u) => u.index === index);
    const sure =
      target &&
      (await confirmed(
        `Remove ${plainGameText(target.name)} (${target.playerName}) from the map?`,
        "Remove placed unit",
        "Remove unit",
        onError,
      ));

    if (sure) {
      run(
        () => api.removePlacedUnit(index),
        ignore,
        () => {
          setSelected(null);
          reloadUnits();
        },
      );
    }
  }

  function key(event: KeyboardEvent, target: PlacedUnit) {
    if (!view || target.index !== selected) {
      return;
    }

    const action = placementKey(event, target, view);

    if (!action) {
      return;
    }

    event.preventDefault();

    if (action === "remove") {
      void remove(target.index);
      return;
    }

    edit(target.index, action);
  }

  return {
    units,
    shown,
    unit: units.find((u) => u.index === selected),
    players: placed?.players ?? [],
    selected,
    setSelected,
    query,
    setQuery,
    heroesOnly,
    setHeroesOnly,
    withDrops,
    setWithDrops,
    placing,
    setPlacing,
    owner,
    setOwner,
    edit,
    setInventory,
    setDrops,
    pickOnMap,
    place,
    remove,
    key,
  };
}

export type PlacedUnits = ReturnType<typeof usePlacedUnits>;

export function useItemTables(data: PlacedData, onError: ErrorHandler) {
  const { tables, run, reloadUnits, reloadTables } = data;
  const [selected, setSelected] = useState<number | null>(null);
  const list = tables?.tables ?? [];

  function save(
    number: number | null,
    name: string,
    sets: DropSetDraft[],
    tiers: TierDraft[] | null,
  ) {
    const table = { number, name, sets: setsOf(sets), tiers: tiers && tiersOf(tiers) };

    run(() => api.saveItemTable(table), setSelected, reloadTables);
  }

  function add(cascade: boolean) {
    const count = list.length + 1;

    if (cascade) {
      save(null, `${NEW_CASCADE_NAME} ${count}`, [], tierDrafts(STARTER_TIERS));
      return;
    }

    save(null, `${NEW_TABLE_NAME} ${count}`, [], null);
  }

  async function remove(target: ItemTableView) {
    const message = [`Remove the item table “${target.name}”?`];

    if (target.users > 0) {
      message.push(`${counted(target.users, "placed unit")} use it and will drop nothing.`);
    }

    if (await confirmed(message.join(" "), "Remove item table", "Remove table", onError)) {
      run(
        () => api.removeItemTable(target.number),
        ignore,
        () => {
          setSelected(null);
          reloadTables();
          reloadUnits();
        },
      );
    }
  }

  return {
    list,
    error: tables?.error ?? null,
    loaded: tables !== null,
    table: list.find((t) => t.number === selected),
    selected,
    setSelected,
    save,
    add,
    remove,
  };
}

export type ItemTables = ReturnType<typeof useItemTables>;
