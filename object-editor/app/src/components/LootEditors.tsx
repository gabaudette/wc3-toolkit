import { ArrowDown, ArrowUp, Plus, Trash01 } from "@untitledui/icons";
import { useEffect, useState } from "react";
import {
  type DropSetDraft,
  FULL_CHANCE,
  itemLabel,
  newKey,
  percent,
  setTotal,
  type TierDraft,
} from "../loot";
import type { FlatDrop, ItemChoice } from "../types";
import { classes, counted } from "../ui";
import { Button } from "./Button";
import { GameText } from "./GameText";
import { UnitIcon } from "./UnitIcon";

export type PickItem = (title: string, then: (code: string) => void) => void;

const DEFAULT_TIER_CHANCE = 10;
const DEFAULT_WEIGHT = 1;
const DECIMALS = 100;

function replacedAt<T>(list: T[], index: number, value: T): T[] {
  return list.map((item, at) => {
    if (at === index) {
      return value;
    }

    return item;
  });
}

function removedAt<T>(list: T[], index: number): T[] {
  return list.filter((_, at) => at !== index);
}

function moved<T>(list: T[], index: number, by: number): T[] {
  const target = index + by;

  if (target < 0 || target >= list.length) {
    return list;
  }

  const next = [...list];
  const [item] = next.splice(index, 1);

  next.splice(target, 0, item as T);
  return next;
}

type NumberInputProps = {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  onCommit: (value: number) => void;
};

export function NumberInput({ label, value, min, max, step = 1, onCommit }: NumberInputProps) {
  const [text, setText] = useState(String(value));

  useEffect(() => setText(String(value)), [value]);

  function commit() {
    const parsed = Number(text);

    if (!Number.isFinite(parsed)) {
      setText(String(value));
      return;
    }

    const clamped = Math.min(max, Math.max(min, Math.round(parsed * DECIMALS) / DECIMALS));

    setText(String(clamped));
    if (clamped !== value) {
      onCommit(clamped);
    }
  }

  return (
    <input
      type="number"
      aria-label={label}
      value={text}
      min={min}
      max={max}
      step={step}
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          commit();
        }
      }}
      className="w-20 rounded-[3px] border border-field bg-well px-2 py-1 text-right text-sm text-ink outline-none focus:border-focus"
    />
  );
}

type ItemButtonProps = {
  code: string;
  choices: Map<string, ItemChoice>;
  onPress: () => void;
};

export function ItemButton({ code, choices, onPress }: ItemButtonProps) {
  const label = itemLabel(code, choices);

  return (
    <button
      type="button"
      onClick={onPress}
      title={`${label.name} (${code}). Choose another item.`}
      className="flex min-w-0 flex-1 items-center gap-2 rounded-[3px] px-1 py-0.5 text-left hover:bg-surface-2"
    >
      {label.random && (
        <span
          aria-hidden="true"
          className="flex size-8 shrink-0 items-center justify-center rounded border border-line bg-surface-2 font-display text-lg text-gold"
        >
          ?
        </span>
      )}
      {!label.random && <UnitIcon path={label.icon} size="small" />}
      <span className="min-w-0 flex-1 truncate text-sm">
        <GameText text={label.name} singleLine />
      </span>
      <span className="font-mono text-xs text-faint">{code}</span>
    </button>
  );
}

type DropSetsEditorProps = {
  sets: DropSetDraft[];
  choices: Map<string, ItemChoice>;
  onPickItem: PickItem;
  onChange: (sets: DropSetDraft[]) => void;
};

export function DropSetsEditor({ sets, choices, onPickItem, onChange }: DropSetsEditorProps) {
  function changeSet(index: number, set: DropSetDraft) {
    onChange(replacedAt(sets, index, set));
  }

  return (
    <div className="flex flex-col gap-3">
      <p className="text-xs text-faint">
        Each item set drops at most one item: one roll picks an item by its chance, and what is left
        drops nothing. Several sets drop several items.
      </p>
      {sets.map((set, setIndex) => {
        const total = setTotal(set.drops);

        return (
          <section key={set.key} className="rounded-[3px] border border-line p-3">
            <header className="mb-2 flex items-center gap-2">
              <h4 className="font-semibold text-gold-bright">Item set {setIndex + 1}</h4>
              <span
                className={classes(
                  "text-xs",
                  total > FULL_CHANCE && "font-semibold text-danger",
                  total <= FULL_CHANCE && "text-faint",
                )}
              >
                {total}% an item · {Math.max(0, FULL_CHANCE - total)}% nothing
              </span>
              <span className="ml-auto">
                <Button small onPress={() => onChange(removedAt(sets, setIndex))}>
                  Remove set
                </Button>
              </span>
            </header>
            {total > FULL_CHANCE && (
              <p role="alert" className="mb-2 text-xs text-danger">
                The chances add up to more than 100%; the last items can never drop.
              </p>
            )}
            <ul className="flex flex-col gap-1">
              {set.drops.map((drop, dropIndex) => (
                <li key={drop.key} className="flex items-center gap-2">
                  <ItemButton
                    code={drop.item}
                    choices={choices}
                    onPress={() =>
                      onPickItem("Choose a drop", (item) =>
                        changeSet(setIndex, {
                          ...set,
                          drops: replacedAt(set.drops, dropIndex, { ...drop, item }),
                        }),
                      )
                    }
                  />
                  <NumberInput
                    label={`Chance of ${drop.item}, in percent`}
                    value={drop.chance}
                    min={0}
                    max={FULL_CHANCE}
                    onCommit={(chance) =>
                      changeSet(setIndex, {
                        ...set,
                        drops: replacedAt(set.drops, dropIndex, { ...drop, chance }),
                      })
                    }
                  />
                  <span className="text-sm text-faint">%</span>
                  <Button
                    small
                    label={`Remove ${drop.item}`}
                    onPress={() =>
                      changeSet(setIndex, { ...set, drops: removedAt(set.drops, dropIndex) })
                    }
                  >
                    <Trash01 className="size-3.5" aria-hidden="true" />
                  </Button>
                </li>
              ))}
            </ul>
            <div className="mt-2">
              <Button
                small
                onPress={() =>
                  onPickItem("Add a drop", (item) =>
                    changeSet(setIndex, {
                      ...set,
                      drops: [
                        ...set.drops,
                        { key: newKey(), item, chance: Math.max(0, FULL_CHANCE - total) },
                      ],
                    }),
                  )
                }
              >
                <span className="flex items-center gap-1">
                  <Plus className="size-3.5" aria-hidden="true" />
                  Add item
                </span>
              </Button>
            </div>
          </section>
        );
      })}
      <div>
        <Button onPress={() => onChange([...sets, { key: newKey(), drops: [] }])}>
          <span className="flex items-center gap-1.5">
            <Plus className="size-4" aria-hidden="true" />
            Add item set
          </span>
        </Button>
      </div>
    </div>
  );
}

type CascadeEditorProps = {
  tiers: TierDraft[];
  flattened: FlatDrop[];
  choices: Map<string, ItemChoice>;
  onPickItem: PickItem;
  onChange: (tiers: TierDraft[]) => void;
};

export function CascadeEditor({
  tiers,
  flattened,
  choices,
  onPickItem,
  onChange,
}: CascadeEditorProps) {
  const nothing = Math.max(0, FULL_CHANCE - flattened.reduce((sum, d) => sum + d.exact, 0));
  const lost = flattened.filter((d) => d.chance === 0);

  function changeTier(index: number, tier: TierDraft) {
    onChange(replacedAt(tiers, index, tier));
  }

  function addTier() {
    onChange([
      ...tiers,
      { key: newKey(), name: `Tier ${tiers.length + 1}`, chance: DEFAULT_TIER_CHANCE, items: [] },
    ]);
  }

  return (
    <div className="flex flex-col gap-3">
      <p className="text-xs text-faint">
        When a unit dies, the first tier rolls its chance. On a hit, one of its items drops (picked
        by weight); on a miss, the next tier rolls, and so on. Put the rarest tier first. The World
        Editor stores the result as one item set in whole percents.
      </p>
      {tiers.map((tier, tierIndex) => (
        <section key={tier.key} className="rounded-[3px] border border-line p-3">
          <header className="mb-2 flex flex-wrap items-center gap-2">
            <span className="text-xs text-faint">Roll {tierIndex + 1}</span>
            <input
              value={tier.name}
              aria-label={`Tier ${tierIndex + 1} name`}
              onChange={(e) => changeTier(tierIndex, { ...tier, name: e.target.value })}
              className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-2 py-1 text-sm text-ink outline-none focus:border-focus"
            />
            <NumberInput
              label={`${tier.name} chance, in percent`}
              value={tier.chance}
              min={0}
              max={FULL_CHANCE}
              step={0.1}
              onCommit={(chance) => changeTier(tierIndex, { ...tier, chance })}
            />
            <span className="text-sm text-faint">% chance</span>
            <Button
              small
              label={`Move ${tier.name} up`}
              disabled={tierIndex === 0}
              onPress={() => onChange(moved(tiers, tierIndex, -1))}
            >
              <ArrowUp className="size-3.5" aria-hidden="true" />
            </Button>
            <Button
              small
              label={`Move ${tier.name} down`}
              disabled={tierIndex === tiers.length - 1}
              onPress={() => onChange(moved(tiers, tierIndex, 1))}
            >
              <ArrowDown className="size-3.5" aria-hidden="true" />
            </Button>
            <Button small onPress={() => onChange(removedAt(tiers, tierIndex))}>
              Remove tier
            </Button>
          </header>
          {tier.items.length === 0 && (
            <p className="text-xs text-faint">No items yet: a hit on this tier drops nothing.</p>
          )}
          <ul className="flex flex-col gap-1">
            {tier.items.map((entry, itemIndex) => {
              const flat = flattened.find((d) => d.tier === tierIndex && d.item === entry.item);

              return (
                <li key={entry.key} className="flex items-center gap-2">
                  <ItemButton
                    code={entry.item}
                    choices={choices}
                    onPress={() =>
                      onPickItem("Choose an item", (item) =>
                        changeTier(tierIndex, {
                          ...tier,
                          items: replacedAt(tier.items, itemIndex, { ...entry, item }),
                        }),
                      )
                    }
                  />
                  <span className="text-xs text-faint">weight</span>
                  <NumberInput
                    label={`Weight of ${entry.item}`}
                    value={entry.weight}
                    min={1}
                    max={1000}
                    onCommit={(weight) =>
                      changeTier(tierIndex, {
                        ...tier,
                        items: replacedAt(tier.items, itemIndex, { ...entry, weight }),
                      })
                    }
                  />
                  <span
                    className={classes(
                      "w-28 text-right text-xs",
                      flat?.chance === 0 && "font-semibold text-danger",
                      flat?.chance !== 0 && "text-muted",
                    )}
                  >
                    {flat && `${percent(flat.exact)} → ${flat.chance}%`}
                  </span>
                  <Button
                    small
                    label={`Remove ${entry.item}`}
                    onPress={() =>
                      changeTier(tierIndex, { ...tier, items: removedAt(tier.items, itemIndex) })
                    }
                  >
                    <Trash01 className="size-3.5" aria-hidden="true" />
                  </Button>
                </li>
              );
            })}
          </ul>
          <div className="mt-2">
            <Button
              small
              onPress={() =>
                onPickItem("Add an item to the tier", (item) =>
                  changeTier(tierIndex, {
                    ...tier,
                    items: [...tier.items, { key: newKey(), item, weight: DEFAULT_WEIGHT }],
                  }),
                )
              }
            >
              <span className="flex items-center gap-1">
                <Plus className="size-3.5" aria-hidden="true" />
                Add item
              </span>
            </Button>
          </div>
        </section>
      ))}
      <div>
        <Button onPress={addTier}>
          <span className="flex items-center gap-1.5">
            <Plus className="size-4" aria-hidden="true" />
            Add tier
          </span>
        </Button>
      </div>
      <p className="text-sm text-muted">
        Nothing drops {percent(nothing)} of the time. Each item shows its exact chance and the whole
        percent the World Editor gets.
      </p>
      {lost.length > 0 && (
        <p role="alert" className="text-sm text-danger">
          {counted(lost.length, "item")} round down to 0% and never drop. Raise their tier's chance
          or weight, or move them to a tier of their own.
        </p>
      )}
    </div>
  );
}
