import { useEffect, useMemo, useRef, useState } from "react";
import type { UnitItem, UnitKind } from "../types";
import { Button } from "./Button";
import { Chip, toggle } from "./Chip";
import { DialogShell } from "./DialogShell";
import { GameText, plainGameText } from "./GameText";
import { UnitIcon } from "./UnitIcon";

const MAX_RESULTS = 200;

const KINDS: { id: UnitKind; label: string }[] = [
  { id: "unit", label: "Units" },
  { id: "hero", label: "Heroes" },
  { id: "building", label: "Buildings" },
];

type UnitPickerProps = {
  open: boolean;
  units: UnitItem[];
  onPick: (unit: UnitItem) => void;
  onClose: () => void;
};

export function UnitPicker({ open, units, onPick, onClose }: UnitPickerProps) {
  const [query, setQuery] = useState("");
  const [kinds, setKinds] = useState<UnitKind[]>([]);
  const [mapOnly, setMapOnly] = useState(false);
  const search = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setQuery("");
      requestAnimationFrame(() => search.current?.focus());
    }
  }, [open]);

  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();

    return units
      .filter((u) => KINDS.some((k) => k.id === u.kind))
      .filter((u) => kinds.length === 0 || kinds.includes(u.kind))
      .filter((u) => !mapOnly || u.custom || u.changedFields > 0)
      .filter(
        (u) =>
          !needle ||
          plainGameText(u.name).toLowerCase().includes(needle) ||
          u.id.toLowerCase().includes(needle),
      )
      .slice(0, MAX_RESULTS);
  }, [units, query, kinds, mapOnly]);

  return (
    <DialogShell
      open={open}
      title="Add a unit"
      wide
      onClose={onClose}
      footer={<Button onPress={onClose}>Cancel</Button>}
    >
      <input
        ref={search}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search units: Footman, hfoo, Paladin…"
        aria-label="Search units"
        className="rounded-[3px] border border-field bg-well px-3 py-1.5 text-ink outline-none focus:border-focus"
      />
      <div className="flex flex-wrap items-center gap-1.5">
        {KINDS.map((k) => (
          <Chip
            key={k.id}
            label={k.label}
            on={kinds.includes(k.id)}
            onPress={() => setKinds(toggle(kinds, k.id))}
          />
        ))}
        <span className="mx-1 h-4 w-px bg-line" aria-hidden="true" />
        <Chip label="Changed in this map" on={mapOnly} onPress={() => setMapOnly(!mapOnly)} />
      </div>
      <ul aria-label="Units" className="h-80 overflow-y-auto rounded-[3px] border border-line">
        {shown.map((u) => (
          <li key={u.id}>
            <button
              type="button"
              onClick={() => onPick(u)}
              className="flex w-full items-center gap-2 px-2 py-1 text-left hover:bg-surface-2"
            >
              <UnitIcon path={u.icon} size="small" />
              <span className="min-w-0 flex-1 truncate">
                <GameText text={u.name} singleLine />
              </span>
              <span className="text-xs text-faint">{u.race}</span>
              <span className="w-12 text-right font-mono text-xs text-faint">{u.id}</span>
            </button>
          </li>
        ))}
        {shown.length === 0 && <li className="p-3 text-sm text-faint">No unit matches.</li>}
      </ul>
      <p className="text-sm text-muted">
        After you pick a unit, click the map where it should stand.
      </p>
    </DialogShell>
  );
}
