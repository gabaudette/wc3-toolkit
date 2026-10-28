import { Plus, XClose } from "@untitledui/icons";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import { confirm } from "../dialogs";
import {
  type DropSetDraft,
  HERO_SLOTS,
  itemLabel,
  setDrafts,
  setsOf,
  type TierDraft,
  tierDrafts,
  tiersOf,
} from "../loot";
import { settle } from "../result";
import type {
  DropsView,
  ItemChoice,
  ItemTableList,
  ItemTableView,
  PlacedList,
  PlacedUnit,
} from "../types";
import { classes, counted } from "../ui";
import { Button } from "./Button";
import { Chip } from "./Chip";
import { GameText } from "./GameText";
import { ItemPicker } from "./ItemPicker";
import { CascadeEditor, DropSetsEditor, type PickItem } from "./LootEditors";
import { Select } from "./Select";
import { UnitIcon } from "./UnitIcon";

type Mode = "units" | "tables";

type Picking = { title: string; random: boolean; then: (code: string) => void };

type PlacedPaneProps = {
  editable: boolean;
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

const NEW_TABLE_NAME = "Item table";
const NEW_CASCADE_NAME = "Loot";
const STARTER_TIERS = [
  { name: "Legendary", chance: 2, items: [] },
  { name: "Epic", chance: 5, items: [] },
  { name: "Rare", chance: 15, items: [] },
  { name: "Common", chance: 100, items: [] },
];

function dropsSummary(drops: DropsView, tables: ItemTableView[]): string {
  if (drops.kind === "table") {
    const table = tables.find((t) => t.number === drops.table);

    return `drops from ${table?.name ?? `table ${drops.table}`}`;
  }

  if (drops.kind === "sets") {
    return `drops ${counted(drops.sets.length, "item set")}`;
  }

  return "";
}

export function PlacedPane({ editable, onMapChanged, onError }: PlacedPaneProps) {
  const [mode, setMode] = useState<Mode>("units");
  const [placed, setPlaced] = useState<PlacedList | null>(null);
  const [tables, setTables] = useState<ItemTableList | null>(null);
  const [choices, setChoices] = useState<ItemChoice[]>([]);
  const [selectedUnit, setSelectedUnit] = useState<number | null>(null);
  const [selectedTable, setSelectedTable] = useState<number | null>(null);
  const [picking, setPicking] = useState<Picking | null>(null);
  const [query, setQuery] = useState("");
  const [heroesOnly, setHeroesOnly] = useState(false);
  const [withDrops, setWithDrops] = useState(false);
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
  }, [editable, reloadUnits, reloadTables, onError]);

  const choiceMap = useMemo(() => new Map(choices.map((c) => [c.id, c])), [choices]);
  const tableList = tables?.tables ?? [];
  const units = placed?.units ?? [];
  const shownUnits = useMemo(() => {
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
  const unit = units.find((u) => u.index === selectedUnit);
  const table = tableList.find((t) => t.number === selectedTable);

  function run(work: () => Promise<unknown>, after: () => void) {
    queue.current = queue.current.then(async () => {
      await work();
      after();
      onMapChanged();
    });
  }

  const pickItem = useCallback(
    (random: boolean): PickItem =>
      (title, then) =>
        setPicking({ title, random, then }),
    [],
  );

  function setInventory(target: PlacedUnit, items: (string | null)[]) {
    run(
      () => settle(api.setPlacedInventory(target.index, items), () => onError(null), onError),
      reloadUnits,
    );
  }

  function setDrops(target: PlacedUnit, drops: DropsView) {
    run(
      () => settle(api.setPlacedDrops(target.index, drops), () => onError(null), onError),
      () => {
        reloadUnits();
        reloadTables();
      },
    );
  }

  function saveTable(
    number: number | null,
    name: string,
    sets: DropSetDraft[],
    tiers: TierDraft[] | null,
  ) {
    run(
      () =>
        settle(
          api.saveItemTable({
            number,
            name,
            sets: setsOf(sets),
            tiers: tiers && tiersOf(tiers),
          }),
          (saved) => {
            onError(null);
            setSelectedTable(saved);
          },
          onError,
        ),
      reloadTables,
    );
  }

  function addTable(cascade: boolean) {
    const count = tableList.length + 1;

    if (cascade) {
      saveTable(null, `${NEW_CASCADE_NAME} ${count}`, [], tierDrafts(STARTER_TIERS));
      return;
    }

    saveTable(null, `${NEW_TABLE_NAME} ${count}`, [], null);
  }

  async function removeTable(target: ItemTableView) {
    const message = [`Remove the item table “${target.name}”?`];

    if (target.users > 0) {
      message.push(`${counted(target.users, "placed unit")} use it and will drop nothing.`);
    }

    const sure = await confirm(message.join(" "), {
      title: "Remove item table",
      kind: "warning",
      okLabel: "Remove table",
      cancelLabel: "Keep it",
    });

    if (!sure.ok) {
      onError(sure.error);
      return;
    }

    if (!sure.value) {
      return;
    }

    run(
      () => settle(api.removeItemTable(target.number), () => onError(null), onError),
      () => {
        setSelectedTable(null);
        reloadTables();
        reloadUnits();
      },
    );
  }

  if (!editable) {
    return (
      <p className="m-auto text-muted">
        Open a map to edit its placed heroes' items and what its units drop.
      </p>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <aside
        aria-label="Placed units and item tables"
        className="panel flex min-h-0 w-80 shrink-0 flex-col rounded-[3px] bg-surface-1"
      >
        <div className="flex flex-col gap-2 border-b border-line p-3">
          <div className="grid grid-cols-2 gap-1 rounded-[3px] border border-line p-1">
            {(["units", "tables"] as const).map((m) => (
              <button
                key={m}
                type="button"
                aria-pressed={mode === m}
                onClick={() => setMode(m)}
                className={classes(
                  "rounded-[3px] px-2 py-1 text-sm",
                  mode === m && "bg-gold-soft font-semibold text-gold-bright",
                  mode !== m && "text-muted hover:text-ink",
                )}
              >
                {m === "units" && "Placed units"}
                {m === "tables" && "Item tables"}
              </button>
            ))}
          </div>
          {mode === "units" && (
            <>
              <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search: name, ID or player"
                aria-label="Search placed units"
                className="rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-sm text-ink outline-none focus:border-focus"
              />
              <div className="flex gap-1.5">
                <Chip label="Heroes" on={heroesOnly} onPress={() => setHeroesOnly(!heroesOnly)} />
                <Chip label="With drops" on={withDrops} onPress={() => setWithDrops(!withDrops)} />
              </div>
              <p className="text-xs text-faint">
                {counted(shownUnits.length, "placed unit")} shown
              </p>
            </>
          )}
          {mode === "tables" && (
            <>
              <div className="grid grid-cols-2 gap-2">
                <Button primary disabled={tables?.error !== null} onPress={() => addTable(true)}>
                  <span className="flex items-center justify-center gap-1.5">
                    <Plus className="size-4" aria-hidden="true" />
                    Cascade
                  </span>
                </Button>
                <Button disabled={tables?.error !== null} onPress={() => addTable(false)}>
                  <span className="flex items-center justify-center gap-1.5">
                    <Plus className="size-4" aria-hidden="true" />
                    Plain table
                  </span>
                </Button>
              </div>
              <p className="text-xs text-faint">
                Placed units can drop from these tables. A cascade rolls tiers from the rarest; a
                plain table is the World Editor's item sets.
              </p>
            </>
          )}
        </div>
        {mode === "units" && placed?.error && (
          <p className="p-3 text-sm text-danger">{placed.error}</p>
        )}
        {mode === "tables" && tables?.error && (
          <p className="p-3 text-sm text-danger">{tables.error}</p>
        )}
        {mode === "units" && (
          <ul aria-label="Placed units" className="min-h-0 flex-1 overflow-y-auto">
            {shownUnits.map((u) => (
              <li key={u.index}>
                <button
                  type="button"
                  onClick={() => setSelectedUnit(u.index)}
                  aria-current={selectedUnit === u.index}
                  className={classes(
                    "flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-surface-2",
                    selectedUnit === u.index &&
                      "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]",
                  )}
                >
                  <UnitIcon path={u.icon} size="small" />
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate">
                      <GameText text={u.name} singleLine />
                    </span>
                    <span className="truncate text-xs text-faint">
                      {u.playerName}
                      {u.hero && ` · level ${u.level}`}
                      {u.drops.kind !== "none" && ` · ${dropsSummary(u.drops, tableList)}`}
                    </span>
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
        {mode === "tables" && (
          <ul aria-label="Item tables" className="min-h-0 flex-1 overflow-y-auto">
            {tableList.map((t) => (
              <li key={t.number}>
                <button
                  type="button"
                  onClick={() => setSelectedTable(t.number)}
                  aria-current={selectedTable === t.number}
                  className={classes(
                    "flex w-full flex-col px-3 py-1.5 text-left hover:bg-surface-2",
                    selectedTable === t.number &&
                      "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]",
                  )}
                >
                  <span className="truncate">{t.name || "(unnamed)"}</span>
                  <span className="truncate text-xs text-faint">
                    {t.tiers && `Cascade · ${counted(t.tiers.length, "tier")}`}
                    {!t.tiers && counted(t.sets.length, "item set")} ·{" "}
                    {counted(t.users, "placed unit")}
                  </span>
                </button>
              </li>
            ))}
            {tables && !tables.error && tableList.length === 0 && (
              <li className="p-3 text-sm text-faint">
                No item tables yet. Make a cascade: rare tiers roll first, and every unit that uses
                it shares it.
              </li>
            )}
          </ul>
        )}
      </aside>

      <section className="frame flex min-h-0 min-w-0 flex-1 flex-col bg-surface-1">
        {mode === "units" && unit && (
          <UnitLoot
            key={unit.index}
            unit={unit}
            tables={tableList}
            choices={choiceMap}
            onPickItem={pickItem}
            onInventory={(items) => setInventory(unit, items)}
            onDrops={(drops) => setDrops(unit, drops)}
            onShowTables={() => setMode("tables")}
          />
        )}
        {mode === "tables" && table && (
          <TableLoot
            key={table.number}
            table={table}
            choices={choiceMap}
            onPickItem={pickItem(true)}
            onSave={(name, sets, tiers) => saveTable(table.number, name, sets, tiers)}
            onRemove={() => removeTable(table)}
          />
        )}
        {((mode === "units" && !unit) || (mode === "tables" && !table)) && (
          <p className="m-auto max-w-md text-center text-muted">
            {mode === "units" &&
              "Choose a placed unit to give a hero items or to choose what the unit drops when it dies."}
            {mode === "tables" && "Choose an item table, or make one with the buttons on the left."}
          </p>
        )}
      </section>

      <ItemPicker
        open={picking !== null}
        title={picking?.title ?? ""}
        choices={choices}
        allowRandom={picking?.random ?? false}
        onPick={(code) => {
          picking?.then(code);
          setPicking(null);
        }}
        onClose={() => setPicking(null)}
      />
    </div>
  );
}

type UnitLootProps = {
  unit: PlacedUnit;
  tables: ItemTableView[];
  choices: Map<string, ItemChoice>;
  onPickItem: (random: boolean) => PickItem;
  onInventory: (items: (string | null)[]) => void;
  onDrops: (drops: DropsView) => void;
  onShowTables: () => void;
};

function UnitLoot({
  unit,
  tables,
  choices,
  onPickItem,
  onInventory,
  onDrops,
  onShowTables,
}: UnitLootProps) {
  const [sets, setSets] = useState<DropSetDraft[]>(() => {
    if (unit.drops.kind === "sets") {
      return setDrafts(unit.drops.sets);
    }

    return [];
  });

  function setSlot(slot: number, item: string | null) {
    onInventory(
      HERO_SLOTS.map((at) => {
        if (at === slot) {
          return item;
        }

        return unit.inventory[at] ?? null;
      }),
    );
  }

  function ownSets() {
    if (sets.length > 0) {
      changeSets(sets);
      return;
    }

    changeSets(setDrafts([[]]));
  }

  function changeSets(next: DropSetDraft[]) {
    setSets(next);
    onDrops({ kind: "sets", sets: setsOf(next) });
  }

  return (
    <>
      <header className="flex items-center gap-3 border-b border-line px-5 py-3">
        <UnitIcon path={unit.icon} size="large" />
        <div className="min-w-0">
          <h2 className="truncate font-display text-3xl font-bold text-gold-bright">
            <GameText text={unit.name} singleLine />
          </h2>
          <p className="truncate font-mono text-xs text-faint">
            {unit.id} · {unit.playerName}
            {unit.hero && ` · level ${unit.level}`} · at ({Math.round(unit.x)}, {Math.round(unit.y)}
            )
          </p>
        </div>
      </header>
      <div className="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto p-5">
        <section aria-labelledby="placed-inventory" className="flex flex-col gap-2">
          <h3 id="placed-inventory" className="font-semibold text-gold-bright">
            Inventory
          </h3>
          {!unit.hero && (
            <p className="text-sm text-faint">
              Only heroes start with items here. Units can still drop items when they die.
            </p>
          )}
          {unit.hero && (
            <>
              <p className="text-sm text-faint">The hero starts the game carrying these items.</p>
              <div className="grid w-fit grid-cols-2 gap-2">
                {HERO_SLOTS.map((slot) => (
                  <InventorySlot
                    key={slot}
                    slot={slot}
                    item={unit.inventory[slot] ?? null}
                    choices={choices}
                    onPick={() =>
                      onPickItem(false)(`Inventory slot ${slot + 1}`, (code) => setSlot(slot, code))
                    }
                    onClear={() => setSlot(slot, null)}
                  />
                ))}
              </div>
            </>
          )}
        </section>

        <section aria-labelledby="placed-drops" className="flex flex-col gap-3">
          <h3 id="placed-drops" className="font-semibold text-gold-bright">
            Drops when it dies
          </h3>
          <fieldset className="flex flex-wrap gap-1.5">
            <legend className="sr-only">Drops</legend>
            <Chip
              label="Nothing"
              on={unit.drops.kind === "none"}
              onPress={() => onDrops({ kind: "none" })}
            />
            <Chip
              label="From an item table"
              on={unit.drops.kind === "table"}
              onPress={() => {
                const first = tables[0];

                if (first) {
                  onDrops({ kind: "table", table: first.number });
                  return;
                }

                onShowTables();
              }}
            />
            <Chip label="Its own item sets" on={unit.drops.kind === "sets"} onPress={ownSets} />
          </fieldset>
          {unit.drops.kind === "table" && (
            <div className="flex items-end gap-2">
              <div className="w-72">
                <Select
                  label="Item table"
                  value={String(unit.drops.table)}
                  onChange={(value) => onDrops({ kind: "table", table: Number(value) })}
                  options={tables.map((t) => ({
                    value: String(t.number),
                    label: t.name || "(unnamed)",
                  }))}
                />
              </div>
              <Button onPress={onShowTables}>Edit item tables</Button>
            </div>
          )}
          {unit.drops.kind === "sets" && (
            <DropSetsEditor
              sets={sets}
              choices={choices}
              onPickItem={onPickItem(true)}
              onChange={changeSets}
            />
          )}
        </section>
      </div>
    </>
  );
}

type InventorySlotProps = {
  slot: number;
  item: string | null;
  choices: Map<string, ItemChoice>;
  onPick: () => void;
  onClear: () => void;
};

function labelOf(item: string | null, choices: Map<string, ItemChoice>) {
  if (!item) {
    return null;
  }

  return itemLabel(item, choices);
}

function InventorySlot({ slot, item, choices, onPick, onClear }: InventorySlotProps) {
  const label = labelOf(item, choices);

  return (
    <div className="flex w-64 items-center gap-2 rounded-[3px] border border-line bg-well p-1.5">
      <button
        type="button"
        onClick={onPick}
        aria-label={`Inventory slot ${slot + 1}: ${label?.name ?? "empty"}. Choose an item.`}
        className="flex min-w-0 flex-1 items-center gap-2 rounded-[3px] text-left hover:bg-surface-2"
      >
        <UnitIcon path={label?.icon ?? null} size="small" />
        <span className={classes("min-w-0 flex-1 truncate text-sm", !label && "text-faint")}>
          {label && <GameText text={label.name} singleLine />}
          {!label && "Empty slot"}
        </span>
      </button>
      {item && (
        <Button small label={`Empty slot ${slot + 1}`} onPress={onClear}>
          <XClose className="size-3.5" aria-hidden="true" />
        </Button>
      )}
    </div>
  );
}

type TableLootProps = {
  table: ItemTableView;
  choices: Map<string, ItemChoice>;
  onPickItem: PickItem;
  onSave: (name: string, sets: DropSetDraft[], tiers: TierDraft[] | null) => void;
  onRemove: () => void;
};

function TableLoot({ table, choices, onPickItem, onSave, onRemove }: TableLootProps) {
  const [name, setName] = useState(table.name);
  const [sets, setSets] = useState(() => setDrafts(table.sets));
  const [tiers, setTiers] = useState(() => table.tiers && tierDrafts(table.tiers));

  return (
    <>
      <header className="flex items-center gap-3 border-b border-line px-5 py-3">
        <input
          value={name}
          aria-label="Item table name"
          onChange={(e) => setName(e.target.value)}
          onBlur={() => {
            if (name !== table.name) {
              onSave(name, sets, tiers);
            }
          }}
          className="min-w-0 flex-1 rounded-[3px] border border-transparent bg-transparent px-1 font-display text-3xl font-bold text-gold-bright outline-none hover:border-field focus:border-focus"
        />
        <Button onPress={onRemove}>Remove table</Button>
      </header>
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-5">
        <p className="text-sm text-muted">
          {table.users > 0 && `${counted(table.users, "placed unit")} drop from this table.`}
          {table.users === 0 &&
            "No placed unit uses this table yet: choose it in a unit's drops under Placed units."}
        </p>
        {tiers && (
          <CascadeEditor
            tiers={tiers}
            flattened={table.flattened}
            choices={choices}
            onPickItem={onPickItem}
            onChange={(next) => {
              setTiers(next);
              onSave(name, sets, next);
            }}
          />
        )}
        {!tiers && (
          <DropSetsEditor
            sets={sets}
            choices={choices}
            onPickItem={onPickItem}
            onChange={(next) => {
              setSets(next);
              onSave(name, next, null);
            }}
          />
        )}
      </div>
    </>
  );
}
