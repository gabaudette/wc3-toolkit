import { useEffect, useMemo, useState } from "react";
import { api, errorMessage } from "../api";
import type { SearchHit, UnitGroup, UnitItem, UnitKind } from "../types";
import { classes } from "../ui";
import { Chip, toggle } from "./Chip";
import { UnitIcon } from "./UnitIcon";

type UnitListProps = {
  units: UnitItem[];
  selected: string | null;
  onSelect: (id: string) => void;
  onError: (message: string) => void;
};

const SEARCH_DELAY_MS = 200;

const GROUPS: { id: UnitGroup; label: string }[] = [
  { id: "map", label: "This map" },
  { id: "melee", label: "Melee" },
  { id: "campaign", label: "Campaign" },
  { id: "special", label: "Special" },
  { id: "hidden", label: "Hidden" },
];

const KINDS: { id: UnitKind; label: string }[] = [
  { id: "hero", label: "Heroes" },
  { id: "unit", label: "Units" },
  { id: "building", label: "Buildings" },
];

const DEFAULT_GROUPS: UnitGroup[] = ["map", "melee", "campaign"];

export function UnitList({ units, selected, onSelect, onError }: UnitListProps) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [groups, setGroups] = useState<UnitGroup[]>(DEFAULT_GROUPS);
  const [kinds, setKinds] = useState<UnitKind[]>(["hero", "unit", "building"]);

  useEffect(() => {
    const trimmed = query.trim();
    if (!trimmed) {
      setHits(null);
      return;
    }

    const timer = window.setTimeout(() => {
      api
        .searchUnits(trimmed)
        .then(setHits)
        .catch((e) => onError(errorMessage(e)));
    }, SEARCH_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [query, onError]);

  const shown = useMemo(() => {
    const byId = new Map(units.map((u) => [u.id, u]));
    const filtered = (unit: UnitItem) => groups.includes(unit.group) && kinds.includes(unit.kind);

    if (hits) {
      return hits
        .map((hit) => ({ unit: byId.get(hit.id), reason: hit.reason }))
        .filter((row): row is { unit: UnitItem; reason: string } => Boolean(row.unit))
        .filter((row) => kinds.includes(row.unit.kind));
    }

    return units.filter(filtered).map((unit) => ({ unit, reason: "" }));
  }, [units, hits, groups, kinds]);

  return (
    <aside
      aria-label="Unit list"
      className="panel flex min-h-0 w-72 shrink-0 flex-col rounded-[3px] bg-surface-1"
    >
      <div className="flex flex-col gap-2 border-b border-line p-3">
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search units: Paladin, Hpal, Metal…"
          aria-label="Search units"
          className="rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-glow"
        />
        <div className="flex flex-wrap gap-1">
          {KINDS.map((k) => (
            <Chip
              key={k.id}
              label={k.label}
              on={kinds.includes(k.id)}
              onPress={() => setKinds(toggle(kinds, k.id))}
            />
          ))}
        </div>
        {!hits && (
          <div className="flex flex-wrap gap-1">
            {GROUPS.map((g) => (
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

      <ul className="min-h-0 flex-1 overflow-y-auto" aria-label="Units">
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
