import { useEffect, useMemo, useState } from "react";
import { api, errorMessage } from "../api";
import type { SearchHit, UnitGroup, UnitItem } from "../types";
import { capitalized, classes } from "../ui";
import { Chip, toggle } from "./Chip";
import { UnitIcon } from "./UnitIcon";

type Filter = { id: string; label: string };

/** What differs between the lists of object types that share this list. */
export type ListConfig = {
  /** Nouns, lowercase: "unit", "units". */
  singular: string;
  noun: string;
  placeholder: string;
  search: (query: string) => Promise<SearchHit[]>;
  groups: { id: UnitGroup; label: string }[];
  defaultGroups: UnitGroup[];
  /** Chips that filter by `filterOf`; `null` makes one per value the list holds. */
  filters: Filter[] | null;
  filterOf: (unit: UnitItem) => string;
};

export const UNIT_LIST: ListConfig = {
  singular: "unit",
  noun: "units",
  placeholder: "Search units: Paladin, Hpal, Metal…",
  search: api.searchUnits,
  groups: [
    { id: "map", label: "This map" },
    { id: "melee", label: "Melee" },
    { id: "campaign", label: "Campaign" },
    { id: "special", label: "Special" },
    { id: "hidden", label: "Hidden" },
  ],
  defaultGroups: ["map", "melee", "campaign"],
  filters: [
    { id: "hero", label: "Heroes" },
    { id: "unit", label: "Units" },
    { id: "building", label: "Buildings" },
  ],
  filterOf: (unit) => unit.kind,
};

export const ITEM_LIST: ListConfig = {
  singular: "item",
  noun: "items",
  placeholder: "Search items: Claws, ratf…",
  search: api.searchItems,
  groups: [
    { id: "map", label: "This map" },
    { id: "game", label: "Game" },
  ],
  defaultGroups: ["map", "game"],
  filters: null,
  filterOf: (item) => item.race,
};

export const UPGRADE_LIST: ListConfig = {
  singular: "upgrade",
  noun: "upgrades",
  placeholder: "Search upgrades: Swords, Rhme…",
  search: api.searchUpgrades,
  groups: [
    { id: "map", label: "This map" },
    { id: "game", label: "Game" },
  ],
  defaultGroups: ["map", "game"],
  filters: null,
  filterOf: (upgrade) => upgrade.race,
};

type UnitListProps = {
  config: ListConfig;
  units: UnitItem[];
  selected: string | null;
  onSelect: (id: string) => void;
  onError: (message: string) => void;
};

const SEARCH_DELAY_MS = 200;

/** One chip per distinct value, in the order the list first shows them. */
function filtersFrom(units: UnitItem[], filterOf: (unit: UnitItem) => string): Filter[] {
  const values = [...new Set(units.map(filterOf))].filter(Boolean);
  return values.map((value) => ({ id: value, label: value }));
}

export function UnitList({ config, units, selected, onSelect, onError }: UnitListProps) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [groups, setGroups] = useState<UnitGroup[]>(config.defaultGroups);
  /** Filters turned off, so values that appear later start on. */
  const [hidden, setHidden] = useState<string[]>([]);
  const { search, filterOf } = config;
  const filters = useMemo(
    () => config.filters ?? filtersFrom(units, filterOf),
    [config.filters, units, filterOf],
  );

  useEffect(() => {
    const trimmed = query.trim();
    if (!trimmed) {
      setHits(null);
      return;
    }

    const timer = window.setTimeout(() => {
      search(trimmed)
        .then(setHits)
        .catch((e) => onError(errorMessage(e)));
    }, SEARCH_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [query, search, onError]);

  const shown = useMemo(() => {
    const byId = new Map(units.map((u) => [u.id, u]));
    const shownKind = (unit: UnitItem) => !hidden.includes(filterOf(unit));
    const filtered = (unit: UnitItem) => groups.includes(unit.group) && shownKind(unit);

    if (hits) {
      return hits
        .map((hit) => ({ unit: byId.get(hit.id), reason: hit.reason }))
        .filter((row): row is { unit: UnitItem; reason: string } => Boolean(row.unit))
        .filter((row) => shownKind(row.unit));
    }

    return units.filter(filtered).map((unit) => ({ unit, reason: "" }));
  }, [units, hits, groups, hidden, filterOf]);

  return (
    <aside
      aria-label={`${capitalized(config.singular)} list`}
      className="panel flex min-h-0 w-72 shrink-0 flex-col rounded-[3px] bg-surface-1"
    >
      <div className="flex flex-col gap-2 border-b border-line p-3">
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={config.placeholder}
          aria-label={`Search ${config.noun}`}
          className="rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-glow"
        />
        <div className="flex flex-wrap gap-1">
          {filters.map((f) => (
            <Chip
              key={f.id}
              label={f.label}
              on={!hidden.includes(f.id)}
              onPress={() => setHidden(toggle(hidden, f.id))}
            />
          ))}
        </div>
        {!hits && (
          <div className="flex flex-wrap gap-1">
            {config.groups.map((g) => (
              <Chip
                key={g.id}
                label={g.label}
                on={groups.includes(g.id)}
                onPress={() => setGroups(toggle(groups, g.id))}
              />
            ))}
          </div>
        )}
        <p className="text-xs text-faint">{shown.length} shown</p>
      </div>

      <ul className="min-h-0 flex-1 overflow-y-auto" aria-label={capitalized(config.noun)}>
        {shown.map(({ unit, reason }) => (
          <li key={unit.id}>
            <button
              type="button"
              onClick={() => onSelect(unit.id)}
              aria-current={selected === unit.id}
              className={classes(
                "flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-surface-2",
                selected === unit.id && "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]",
              )}
            >
              <UnitIcon path={unit.icon} size="small" />
              <span className="min-w-0 flex-1">
                <span className="block truncate">{unit.name}</span>
                <span className="block truncate font-mono text-xs text-faint">
                  {unit.id}
                  {unit.custom && ` · based on ${unit.base}`}
                  {reason && ` · ${reason}`}
                </span>
              </span>
              {unit.changedFields > 0 && (
                <span
                  className="rounded bg-changed-soft px-1.5 text-xs text-changed"
                  title={`${unit.changedFields} changed fields`}
                >
                  {unit.changedFields}
                </span>
              )}
            </button>
          </li>
        ))}
      </ul>
    </aside>
  );
}
