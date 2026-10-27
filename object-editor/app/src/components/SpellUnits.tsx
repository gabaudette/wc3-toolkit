import { Check } from "@untitledui/icons";
import { useEffect, useMemo, useState } from "react";
import { api, errorMessage } from "../api";
import type { CardButton, SpellFit, UnitItem } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { GameText } from "./GameText";
import { UnitIcon } from "./UnitIcon";

type SpellUnitsProps = {
  spell: string;
  /** Called after the spell is given or taken, so the map shows as unsaved and units reload. */
  onChanged: () => void;
};

/** Kinds that can have abilities. */
const HOLDER_KINDS = ["hero", "unit", "building"];
const KIND_LABELS: Record<string, string> = { hero: "Hero", unit: "Unit", building: "Building" };
/** Rows drawn at once; typing narrows the list further. */
const MAX_ROWS = 80;
const COLUMNS = 4;
const ROWS = 3;

/**
 * Gives a spell to units, heroes and buildings, or takes it away: a list of
 * them (those with the spell first, then the map's own) and, for the one
 * picked, its command card and learn menu with the spell placed, plus
 * hotkey, spot and full-card warnings.
 */
export function SpellUnits({ spell, onChanged }: SpellUnitsProps) {
  const [units, setUnits] = useState<UnitItem[]>([]);
  const [holders, setHolders] = useState<ReadonlySet<string>>(new Set());
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const [fit, setFit] = useState<SpellFit | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.listUnits(), api.spellUnits(spell)])
      .then(([all, has]) => {
        setUnits(all.filter((u) => HOLDER_KINDS.includes(u.kind)));
        setHolders(new Set(has));
      })
      .catch((e: unknown) => setError(errorMessage(e)));
  }, [spell]);

  useEffect(() => {
    setFit(null);
    if (!selected) {
      return;
    }

    // A slower answer for a unit picked earlier must not replace this one.
    let current = true;
    api
      .spellFit(spell, selected)
      .then((answer) => {
        if (current) {
          setFit(answer);
        }
      })
      .catch((e: unknown) => setError(errorMessage(e)));
    return () => {
      current = false;
    };
  }, [spell, selected]);

  const listed = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const matches = units.filter((u) => {
      if (needle) {
        return u.name.toLowerCase().includes(needle) || u.id.toLowerCase().includes(needle);
      }

      return holders.has(u.id) || u.group === "map";
    });
    const first = matches.filter((u) => holders.has(u.id));
    const rest = matches.filter((u) => !holders.has(u.id));
    return [...first, ...rest].slice(0, MAX_ROWS);
  }, [units, holders, query]);

  const unit = units.find((u) => u.id === selected);

  async function toggle() {
    if (!selected || !fit) {
      return;
    }

    setBusy(true);
    setError(null);
    try {
      let updated: SpellFit;

      if (fit.has) {
        updated = await api.takeSpell(spell, selected);
      } else {
        updated = await api.giveSpell(spell, selected);
      }

      setFit(updated);
      setHolders((current) => {
        const next = new Set(current);

        if (updated.has) {
          next.add(selected);
        } else {
          next.delete(selected);
        }

        return next;
      });

      onChanged();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      {error && (
        <p role="alert" className="text-danger">
          {error}
        </p>
      )}
      <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)] gap-5">
        <section aria-label="Units" className="flex min-w-0 flex-col gap-2">
          <input
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search units: footman, Hpal, archmage…"
            aria-label="Search units"
            className="rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-focus"
          />
          {listed.length === 0 && (
            <p className="text-sm text-faint">
              No unit has this spell yet. Search for a unit, hero or building to give it to.
            </p>
          )}
          <ul
            aria-label="Units to give the spell to"
            className="flex h-96 flex-col overflow-y-auto"
          >
            {listed.map((u) => (
              <li key={u.id}>
                <UnitRow
                  unit={u}
                  has={holders.has(u.id)}
                  picked={u.id === selected}
                  onPick={() => setSelected(u.id)}
                />
              </li>
            ))}
          </ul>
        </section>
        {unit && fit && <UnitFit unit={unit} fit={fit} busy={busy} onToggle={toggle} />}
        {!unit && (
          <p className="m-auto text-center text-sm text-faint">
            Pick a unit to see its command card with the spell on it.
          </p>
        )}
      </div>
    </>
  );
}

type UnitRowProps = {
  unit: UnitItem;
  has: boolean;
  picked: boolean;
  onPick: () => void;
};

function UnitRow({ unit, has, picked, onPick }: UnitRowProps) {
  return (
    <button
      type="button"
      onClick={onPick}
      aria-pressed={picked}
      className={classes(
        "flex w-full items-center gap-2 rounded-[3px] border px-2 py-1 text-left outline-none hover:bg-well focus-visible:border-focus",
        picked && "border-gold bg-well",
        !picked && "border-transparent",
      )}
    >
      <UnitIcon path={unit.icon} size="small" />
      <span className="min-w-0 flex-1">
        <span className="block truncate">
          <GameText text={unit.name} singleLine />
        </span>
        <span className="font-mono text-xs text-faint">
          {unit.id} · {KIND_LABELS[unit.kind] ?? unit.kind}
        </span>
      </span>
      {has && (
        <span className="flex items-center gap-1 text-xs text-gold">
          <Check className="size-4" aria-hidden="true" />
          Has it
        </span>
      )}
    </button>
  );
}

type UnitFitProps = {
  unit: UnitItem;
  fit: SpellFit;
  busy: boolean;
  onToggle: () => void;
};

function placement(fit: SpellFit): string {
  if (fit.has && fit.learned) {
    return "Learns it from the learn menu, level by level.";
  }

  if (fit.has) {
    return "Has it as a normal ability on the command card.";
  }

  if (fit.learned) {
    return "Would learn it from the learn menu, level by level.";
  }

  return "Would get it as a normal ability on the command card.";
}

function UnitFit({ unit, fit, busy, onToggle }: UnitFitProps) {
  let action = "Give the spell";

  if (fit.has) {
    action = "Take the spell away";
  }

  return (
    <section aria-label="The spell on this unit" className="flex min-w-0 flex-col gap-3">
      <div className="flex items-center gap-3">
        <UnitIcon path={unit.icon} size="large" />
        <div className="min-w-0">
          <h3 className="truncate font-semibold text-gold-bright">
            <GameText text={unit.name} singleLine />
          </h3>
          <p className="text-sm text-muted">{placement(fit)}</p>
        </div>
      </div>
      <div className="flex flex-wrap gap-6">
        <CardPreview title="Command card" buttons={fit.card} />
        {fit.learned && <CardPreview title="Learn menu" buttons={fit.learn} />}
      </div>
      {fit.warnings.map((warning) => (
        <p key={warning} role="status" className="text-sm text-gold">
          {warning}
        </p>
      ))}
      {fit.blocked && (
        <p role="status" className="text-sm text-danger">
          {fit.blocked}
        </p>
      )}
      <div>
        <Button primary={!fit.has} onPress={onToggle} disabled={busy || fit.blocked !== null}>
          {action}
        </Button>
      </div>
    </section>
  );
}

/** A 4×3 menu with each button on its spot, the spell outlined in gold. */
function CardPreview({ title, buttons }: { title: string; buttons: CardButton[] }) {
  const cells = Array.from({ length: COLUMNS * ROWS }, (_, i) => ({
    x: i % COLUMNS,
    y: Math.floor(i / COLUMNS),
  }));

  return (
    <figure className="flex flex-col gap-2">
      <figcaption className="text-sm font-semibold text-muted">{title}</figcaption>
      <ul aria-label={title} className="grid w-fit grid-cols-4 gap-1">
        {cells.map((cell) => {
          const here = buttons.filter((b) => b.x === cell.x && b.y === cell.y);
          const shown = here.find((b) => b.spell) ?? here[0];
          const names = here.map((b) => b.name).join(", ");

          return (
            <li
              key={`${cell.x}-${cell.y}`}
              title={names || undefined}
              aria-label={`Column ${cell.x + 1}, row ${cell.y + 1}: ${names || "empty"}`}
              className={classes(
                "flex size-12 items-center justify-center overflow-hidden rounded-[3px] border text-center text-[10px] leading-tight text-faint",
                shown?.spell && "border-gold",
                !shown?.spell && "border-line bg-well",
              )}
            >
              {shown?.icon && <UnitIcon path={shown.icon} size="small" />}
              {shown && !shown.icon && shown.name}
            </li>
          );
        })}
      </ul>
    </figure>
  );
}
