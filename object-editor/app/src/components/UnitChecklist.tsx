import { useMemo, useState } from "react";
import type { UnitItem } from "../types";
import { UnitIcon } from "./UnitIcon";

type UnitChecklistProps = {
  units: UnitItem[];
  chosen: string[];
  onChange: (chosen: string[]) => void;
};

/** Custom units to tick, with a filter and "all / none" for long lists. */
export function UnitChecklist({ units, chosen, onChange }: UnitChecklistProps) {
  const [query, setQuery] = useState("");
  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return units.filter(
      (u) =>
        !needle || u.name.toLowerCase().includes(needle) || u.id.toLowerCase().includes(needle),
    );
  }, [units, query]);

  function toggle(id: string) {
    if (chosen.includes(id)) {
      onChange(chosen.filter((c) => c !== id));
      return;
    }

    onChange([...chosen, id]);
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Filter units"
          aria-label="Filter units"
          className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-3 py-1 text-sm outline-none focus:border-glow"
        />
        <button
          type="button"
          onClick={() => onChange([...new Set([...chosen, ...shown.map((u) => u.id)])])}
          className="text-xs text-muted hover:text-ink"
        >
          All
        </button>
        <button
          type="button"
          onClick={() => onChange([])}
          className="text-xs text-muted hover:text-ink"
        >
          None
        </button>
      </div>
      <ul className="max-h-60 overflow-y-auto rounded-[3px] border border-line" aria-label="Units">
        {shown.map((unit) => (
          <li key={unit.id}>
            <label className="flex cursor-pointer items-center gap-3 px-3 py-1 hover:bg-surface-2">
              <input
                type="checkbox"
                checked={chosen.includes(unit.id)}
                onChange={() => toggle(unit.id)}
              />
              <UnitIcon path={unit.icon} size="small" />
              <span className="min-w-0 flex-1 truncate">{unit.name}</span>
              <span className="font-mono text-xs text-faint">{unit.id}</span>
            </label>
          </li>
        ))}
        {shown.length === 0 && <li className="p-3 text-sm text-muted">No custom units.</li>}
      </ul>
      <p className="text-xs text-faint">{chosen.length} chosen</p>
    </div>
  );
}
