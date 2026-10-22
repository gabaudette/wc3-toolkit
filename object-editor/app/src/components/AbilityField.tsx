import { Plus, XClose } from "@untitledui/icons";
import { use, useEffect, useState } from "react";
import { api, errorMessage } from "../api";
import type { AbilityCard, Field } from "../types";
import { levelCount } from "../ui";
import { AbilityPicker } from "./AbilityPicker";
import { Button } from "./Button";
import { GameText } from "./GameText";
import { OpenAbilityContext } from "./openAbility";
import { UnitIcon } from "./UnitIcon";

type AbilityFieldProps = {
  field: Field;
  editable: boolean;
  onChange: (value: string) => void;
};

function codesOf(raw: string): string[] {
  return raw
    .split(",")
    .map((code) => code.trim())
    .filter(Boolean);
}

function Card({ card, onRemove }: { card: AbilityCard; onRemove: (() => void) | null }) {
  const [expanded, setExpanded] = useState(false);
  const openAbility = use(OpenAbilityContext);
  const { summary } = card;
  const levels = Array.from({ length: summary.levels }, (_, i) => i + 1);
  const facts = [
    summary.hero && "Hero ability",
    levelCount(summary.levels),
    card.hotkey && `hotkey ${card.hotkey}`,
  ].filter(Boolean);

  return (
    <li className="rounded-[3px] border border-line bg-surface-1">
      <div className="flex items-start gap-3 p-2">
        <UnitIcon path={summary.icon} size="small" />
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium">
            {summary.name}
            <span className="ml-2 font-mono text-xs font-normal text-faint">{summary.id}</span>
          </p>
          {card.known && <p className="text-xs text-muted">{facts.join(" · ")}</p>}
          {!card.known && (
            <p className="text-xs text-faint">Neither the game data nor this map has it.</p>
          )}
        </div>
        {card.known && (
          <button
            type="button"
            onClick={() => setExpanded(!expanded)}
            aria-expanded={expanded}
            className="rounded px-2 py-0.5 text-xs text-faint hover:bg-surface-2 hover:text-ink"
          >
            {expanded && "Less"}
            {!expanded && "Details"}
          </button>
        )}
        {card.known && openAbility && (
          <button
            type="button"
            onClick={() => openAbility(summary.id)}
            title="Open on the Abilities tab"
            className="rounded px-2 py-0.5 text-xs text-faint hover:bg-surface-2 hover:text-ink"
          >
            Open
          </button>
        )}
        {onRemove && (
          <button
            type="button"
            onClick={onRemove}
            aria-label={`Remove ${summary.name}`}
            title="Remove"
            className="rounded p-1 text-faint hover:bg-surface-2 hover:text-danger"
          >
            <XClose className="size-4" aria-hidden="true" />
          </button>
        )}
      </div>

      {expanded && (
        <div className="border-t border-line px-3 py-2 text-sm">
          {card.description && (
            <p className="mb-2 rounded bg-surface-0 px-3 py-2 leading-relaxed text-ink">
              <GameText text={card.description} />
            </p>
          )}
          {card.rows.length > 0 && (
            <table className="w-full text-xs">
              <thead>
                <tr className="text-faint">
                  <th className="py-1 text-left font-normal">Per level</th>
                  {levels.map((level) => (
                    <th key={level} className="py-1 text-right font-normal">
                      {level}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {card.rows.map((row) => (
                  <tr key={row.label} className="border-t border-line/60">
                    <td className="py-1 text-muted">{row.label}</td>
                    {row.values.map((value, index) => (
                      <td key={levels[index]} className="py-1 text-right font-mono">
                        {value}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </li>
  );
}

/** Ability lists as cards, with a picker instead of typing codes. */
export function AbilityField({ field, editable, onChange }: AbilityFieldProps) {
  const [cards, setCards] = useState<AbilityCard[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [picking, setPicking] = useState(false);
  const codes = codesOf(field.raw);
  const heroOnly = field.valueType === "heroAbilityList";
  const full = field.max !== null && codes.length >= field.max;

  useEffect(() => {
    const ids = codesOf(field.raw);
    if (ids.length === 0) {
      setCards([]);
      return;
    }

    api
      .abilityCards(ids)
      .then(setCards)
      .catch((e) => setError(errorMessage(e)));
  }, [field.raw]);

  function remove(id: string) {
    onChange(codes.filter((c) => c !== id).join(","));
  }

  function add(id: string) {
    setPicking(false);
    onChange([...codes, id].join(","));
  }

  return (
    <div className="col-span-4 mb-2 flex flex-col gap-2">
      {error && <p className="text-sm text-danger">{error}</p>}
      {cards.length === 0 && <p className="text-sm text-faint">No abilities.</p>}
      <ul className="flex flex-col gap-1.5">
        {cards.map((card) => {
          let onRemove: (() => void) | null = null;
          if (editable) {
            onRemove = () => remove(card.summary.id);
          }

          return <Card key={card.summary.id} card={card} onRemove={onRemove} />;
        })}
      </ul>
      {editable && (
        <div className="flex items-center gap-3">
          <Button small disabled={full} onPress={() => setPicking(true)}>
            <span className="flex items-center gap-1">
              <Plus className="size-3.5" aria-hidden="true" />
              Add ability
            </span>
          </Button>
          {full && <span className="text-xs text-faint">A hero can have at most {field.max}.</span>}
        </div>
      )}
      <AbilityPicker
        open={picking}
        heroOnly={heroOnly}
        exclude={codes}
        onPick={add}
        onClose={() => setPicking(false)}
      />
    </div>
  );
}
