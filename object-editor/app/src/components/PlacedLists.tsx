import { AlertCircle, Plus } from "@untitledui/icons";
import type { DropsView, ItemTableView, MapView, PlacedList } from "../types";
import { classes, counted } from "../ui";
import { Button } from "./Button";
import { Chip } from "./Chip";
import { GameText } from "./GameText";
import { PlacedMap, playerColor } from "./PlacedMap";
import type { ItemTables, PlacedUnits } from "./placedState";
import { Select } from "./Select";
import { UnitIcon } from "./UnitIcon";

export type Mode = "units" | "tables";

const MODES: { id: Mode; label: string }[] = [
  { id: "units", label: "Placed units" },
  { id: "tables", label: "Item tables" },
];

const ROW = "flex w-full px-3 py-1.5 text-left hover:bg-surface-2";
const CURRENT_ROW = "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]";

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

export function ModeSwitch({ mode, onMode }: { mode: Mode; onMode: (mode: Mode) => void }) {
  return (
    <div className="grid grid-cols-2 gap-1 rounded-[3px] border border-line p-1">
      {MODES.map((m) => (
        <button
          key={m.id}
          type="button"
          aria-pressed={mode === m.id}
          onClick={() => onMode(m.id)}
          className={classes(
            "rounded-[3px] px-2 py-1 text-sm",
            mode === m.id && "bg-gold-soft font-semibold text-gold-bright",
            mode !== m.id && "text-muted hover:text-ink",
          )}
        >
          {m.label}
        </button>
      ))}
    </div>
  );
}

type PlacedUnitListProps = {
  placed: PlacedUnits;
  error: string | null | undefined;
  tables: ItemTableView[];
};

export function PlacedUnitFilters({ placed }: { placed: PlacedUnits }) {
  return (
    <>
      <input
        value={placed.query}
        onChange={(e) => placed.setQuery(e.target.value)}
        placeholder="Search: name, ID or player"
        aria-label="Search placed units"
        className="rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-sm text-ink outline-none focus:border-focus"
      />
      <div className="flex gap-1.5">
        <Chip
          label="Heroes"
          on={placed.heroesOnly}
          onPress={() => placed.setHeroesOnly(!placed.heroesOnly)}
        />
        <Chip
          label="With drops"
          on={placed.withDrops}
          onPress={() => placed.setWithDrops(!placed.withDrops)}
        />
      </div>
      <p className="text-xs text-faint">
        {counted(placed.shown.length, "placed unit")} shown. On the picked unit: arrow keys move it
        (Shift: farther), [ and ] turn it, Delete removes it.
      </p>
    </>
  );
}

export function PlacedUnitList({ placed, error, tables }: PlacedUnitListProps) {
  return (
    <>
      {error && <p className="p-3 text-sm text-danger">{error}</p>}
      <ul aria-label="Placed units" className="min-h-0 flex-1 overflow-y-auto">
        {placed.shown.map((u) => (
          <li key={u.index} id={`placed-${u.index}`}>
            <button
              type="button"
              onClick={() => placed.setSelected(u.index)}
              onKeyDown={(e) => placed.key(e, u)}
              aria-current={placed.selected === u.index}
              className={classes(
                ROW,
                "items-center gap-2",
                placed.selected === u.index && CURRENT_ROW,
              )}
            >
              <span
                className="size-2.5 shrink-0 rounded-full border border-ink"
                style={{ backgroundColor: playerColor(u.player) }}
                aria-hidden="true"
              />
              <UnitIcon path={u.icon} size="small" />
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="truncate">
                  <GameText text={u.name} singleLine />
                </span>
                <span className="truncate text-xs text-faint">
                  {u.playerName}
                  {u.hero && ` · level ${u.level}`}
                  {u.drops.kind !== "none" && ` · ${dropsSummary(u.drops, tables)}`}
                </span>
              </span>
            </button>
          </li>
        ))}
      </ul>
    </>
  );
}

function AddButton({ label, primary, disabled, onPress }: AddButtonProps) {
  return (
    <Button primary={primary} disabled={disabled} onPress={onPress}>
      <span className="flex items-center justify-center gap-1.5">
        <Plus className="size-4" aria-hidden="true" />
        {label}
      </span>
    </Button>
  );
}

type AddButtonProps = { label: string; primary: boolean; disabled: boolean; onPress: () => void };

export function ItemTableActions({ tables }: { tables: ItemTables }) {
  const blocked = !tables.loaded || tables.error !== null;

  return (
    <>
      <div className="grid grid-cols-2 gap-2">
        <AddButton label="Cascade" primary disabled={blocked} onPress={() => tables.add(true)} />
        <AddButton
          label="Plain table"
          primary={false}
          disabled={blocked}
          onPress={() => tables.add(false)}
        />
      </div>
      <p className="text-xs text-faint">
        Placed units can drop from these tables. A cascade rolls tiers from the rarest; a plain
        table is the World Editor's item sets.
      </p>
    </>
  );
}

export function ItemTableColumn({ tables }: { tables: ItemTables }) {
  return (
    <>
      {tables.error && <p className="p-3 text-sm text-danger">{tables.error}</p>}
      <ul aria-label="Item tables" className="min-h-0 flex-1 overflow-y-auto">
        {tables.list.map((t) => (
          <li key={t.number}>
            <button
              type="button"
              onClick={() => tables.setSelected(t.number)}
              aria-current={tables.selected === t.number}
              className={classes(ROW, "flex-col", tables.selected === t.number && CURRENT_ROW)}
            >
              <span className="truncate">{t.name || "(unnamed)"}</span>
              <span className="truncate text-xs text-faint">
                {t.tiers && `Cascade · ${counted(t.tiers.length, "tier")}`}
                {!t.tiers && counted(t.sets.length, "item set")} · {counted(t.users, "placed unit")}
              </span>
            </button>
          </li>
        ))}
        {tables.loaded && !tables.error && tables.list.length === 0 && (
          <li className="p-3 text-sm text-faint">
            No item tables yet. Make a cascade: rare tiers roll first, and every unit that uses it
            shares it.
          </li>
        )}
      </ul>
    </>
  );
}

type PlacedMapSectionProps = {
  list: PlacedList | null;
  view: MapView | null;
  viewError: string | null;
  placed: PlacedUnits;
  onAddUnit: () => void;
};

export function PlacedMapSection({
  list,
  view,
  viewError,
  placed,
  onAddUnit,
}: PlacedMapSectionProps) {
  return (
    <section aria-label="Map" className="frame flex min-h-0 min-w-0 flex-1 flex-col bg-surface-1">
      <div className="flex flex-wrap items-end gap-2 border-b border-line px-3 py-2">
        {!placed.placing && (
          <AddButton label="Add unit" primary disabled={list?.error !== null} onPress={onAddUnit} />
        )}
        {placed.placing && <Button onPress={() => placed.setPlacing(null)}>Stop placing</Button>}
        <div className="w-56">
          <Select
            label="Owner of new units"
            value={String(placed.owner)}
            onChange={(value) => placed.setOwner(Number(value))}
            options={placed.players.map((p) => ({ value: String(p.number), label: p.name }))}
          />
        </div>
      </div>
      {list?.scriptProblem && (
        <p className="flex items-start gap-2 border-b border-line bg-well px-3 py-2 text-sm text-muted">
          <AlertCircle className="mt-0.5 size-4 shrink-0 text-gold" aria-hidden="true" />
          {list.scriptProblem}
        </p>
      )}
      {view && (
        <PlacedMap
          view={view}
          units={placed.shown}
          selected={placed.selected}
          placing={placed.placing}
          onSelect={placed.pickOnMap}
          onMove={(index, x, y) => placed.edit(index, { x, y })}
          onTurn={(index, facing) => placed.edit(index, { facing })}
          onPlace={placed.place}
          onCancelPlace={() => placed.setPlacing(null)}
        />
      )}
      {!view && (
        <p className="m-auto max-w-md text-center text-muted">{viewError ?? "Drawing the map…"}</p>
      )}
    </section>
  );
}
