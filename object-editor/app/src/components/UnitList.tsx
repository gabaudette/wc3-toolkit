import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { type Result, settle } from "../result";
import type { SearchHit, UnitGroup, UnitItem } from "../types";
import { capitalized, classes } from "../ui";
import { Chip, toggle } from "./Chip";
import { GameText } from "./GameText";
import { ListFold, useFolded } from "./ListFold";
import { UnitIcon } from "./UnitIcon";

type Filter = { id: string; label: string };

export type ListConfig = {
  singular: string;
  article: string;
  noun: string;
  placeholder: string;
  search: (query: string) => Promise<Result<SearchHit[]>>;
  groups: { id: UnitGroup; label: string }[];
  defaultGroups: UnitGroup[];
  filters: Filter[] | null;
  filterOf: (unit: UnitItem) => string;
};

export const UNIT_LIST: ListConfig = {
  singular: "unit",
  article: "a",
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
  article: "an",
  noun: "items",
  placeholder: "Search items: Claws, ratf…",
  search: (query) => api.searchPlain("items", query),
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
  article: "an",
  noun: "upgrades",
  placeholder: "Search upgrades: Swords, Rhme…",
  search: (query) => api.searchLeveled("upgrades", query),
  groups: [
    { id: "map", label: "This map" },
    { id: "game", label: "Game" },
  ],
  defaultGroups: ["map", "game"],
  filters: null,
  filterOf: (upgrade) => upgrade.race,
};

export const DOODAD_LIST: ListConfig = {
  singular: "doodad",
  article: "a",
  noun: "doodads",
  placeholder: "Search doodads: Mushrooms, APms…",
  search: (query) => api.searchLeveled("doodads", query),
  groups: [
    { id: "map", label: "This map" },
    { id: "game", label: "Game" },
  ],
  defaultGroups: ["map", "game"],
  filters: null,
  filterOf: (doodad) => doodad.race,
};

export const BUFF_LIST: ListConfig = {
  singular: "buff",
  article: "a",
  noun: "buffs",
  placeholder: "Search buffs: Blizzard, BHbd…",
  search: (query) => api.searchPlain("buffs", query),
  groups: [
    { id: "map", label: "This map" },
    { id: "game", label: "Game" },
  ],
  defaultGroups: ["map", "game"],
  filters: null,
  filterOf: (buff) => buff.race,
};

export const DESTRUCTABLE_LIST: ListConfig = {
  singular: "destructable",
  article: "a",
  noun: "destructables",
  placeholder: "Search destructables: Tree, LTlt…",
  search: (query) => api.searchPlain("destructables", query),
  groups: [
    { id: "map", label: "This map" },
    { id: "game", label: "Game" },
  ],
  defaultGroups: ["map", "game"],
  filters: null,
  filterOf: (destructable) => destructable.race,
};

type UnitListProps = {
  config: ListConfig;
  units: UnitItem[];
  selected: string | null;
  onSelect: (id: string) => void;
  onError: (message: string) => void;
};

const SEARCH_DELAY_MS = 200;

function filtersFrom(units: UnitItem[], filterOf: (unit: UnitItem) => string): Filter[] {
  const values = [...new Set(units.map(filterOf))].filter(Boolean);

  return values.map((value) => ({ id: value, label: value }));
}

export function UnitList({ config, units, selected, onSelect, onError }: UnitListProps) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [groups, setGroups] = useState<UnitGroup[]>(config.defaultGroups);
  const [hidden, setHidden] = useState<string[]>([]);
  const [folded, toggleFolded] = useFolded(`wc3-object-editor.fold.${config.noun}`);
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
      void settle(search(trimmed), setHits, onError);
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

  const own = shown.filter((r) => Boolean(hits) || r.unit.group === "map");
  const game = shown.filter((r) => !hits && r.unit.group !== "map");
  const mapHasNone = !units.some((u) => u.group === "map");
  let visible = own.length + game.length;

  if (folded) {
    visible = own.length;
  }

  function row({ unit, reason }: { unit: UnitItem; reason: string }) {
    return (
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
            <span className="block truncate">
              <GameText text={unit.name} singleLine />
            </span>
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
    );
  }

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
          className="rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-focus"
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
        <p className="text-xs text-faint">
          {visible} shown
          {folded && game.length > 0 && ` · ${game.length} game ${config.noun} folded`}
        </p>
      </div>

      <ul className="min-h-0 flex-1 overflow-y-auto" aria-label={capitalized(config.noun)}>
        {own.map(row)}
        {mapHasNone && folded && !hits && (
          <li className="px-3 py-4 text-sm text-muted">
            This map adds or changes no {config.noun} yet.
          </li>
        )}
        {game.length > 0 && (
          <ListFold
            label={`Game ${config.noun}`}
            count={game.length}
            folded={folded}
            onToggle={toggleFolded}
          />
        )}
        {!folded && game.map(row)}
      </ul>
    </aside>
  );
}
