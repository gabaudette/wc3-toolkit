import { FlipBackward } from "@untitledui/icons";
import { type KeyboardEvent, useEffect, useState } from "react";
import {
  BUTTON_COLUMNS,
  BUTTON_ROWS,
  diceProblem,
  diceStats,
  type FieldGroup,
  hexToRgb,
  partProblem,
  rgbToHex,
  rollStats,
} from "../fieldGroups";
import type { DamageBonus, Field } from "../types";
import { classes, FIELD_ROW_GRID } from "../ui";

type FieldGroupRowProps = {
  group: FieldGroup;
  fields: Field[];
  editable: boolean;
  damageBonus: DamageBonus | null;
  onChange: (fieldId: string, value: string) => void;
  onReset: (fieldId: string) => void;
};

const AMOUNT_PRECISION = 100;
const numberClass =
  "w-full rounded border border-field bg-well px-2 py-1 font-mono outline-none focus:border-focus read-only:opacity-70";

function numberOf(field: Field | undefined): number {
  const value = Number(field?.raw ?? "");

  if (Number.isFinite(value)) {
    return value;
  }

  return 0;
}

function Part({
  field,
  label,
  draft,
  problem,
  editable,
  onDraft,
  onCommit,
}: {
  field: Field;
  label: string;
  draft: string;
  problem: string | null;
  editable: boolean;
  onDraft: (value: string) => void;
  onCommit: () => void;
}) {
  function key(e: KeyboardEvent) {
    if (e.key === "Enter") {
      onCommit();
    }
  }

  return (
    <label className="flex min-w-0 flex-1 flex-col gap-0.5">
      <span className="text-[11px] text-faint">{label}</span>
      <input
        type="number"
        value={draft}
        readOnly={!editable}
        onChange={(e) => onDraft(e.target.value)}
        onBlur={onCommit}
        onKeyDown={key}
        aria-label={`${label} (${field.name})`}
        aria-invalid={problem !== null}
        className={classes(
          numberClass,
          field.modified && "border-changed",
          problem !== null && "border-danger",
        )}
      />
    </label>
  );
}

function amount(value: number): string {
  return String(Math.round(value * AMOUNT_PRECISION) / AMOUNT_PRECISION);
}

function DiceResult({
  group,
  values,
  damageBonus,
}: {
  group: FieldGroup;
  values: number[];
  damageBonus: DamageBonus | null;
}) {
  const [base = 0, dice = 0, sides = 0] = values;
  let bonus: DamageBonus | null = null;

  if (group.attack && damageBonus && damageBonus.amount !== 0) {
    bonus = damageBonus;
  }

  const total = diceStats(base, dice, sides, bonus?.amount ?? 0);
  const roll = rollStats(dice, sides);

  return (
    <div className="text-xs text-muted">
      <p>
        <span className="font-mono text-ink">{amount(total.min)}</span> to{" "}
        <span className="font-mono text-ink">{amount(total.max)}</span>, average{" "}
        <span className="font-mono text-ink">{amount(total.average)}</span>
      </p>
      <p className="text-faint">
        {`${amount(base)} base + ${dice} × d${sides} (${amount(roll.min)} to ${amount(roll.max)}, average ${amount(roll.average)})`}
        {bonus &&
          ` + ${amount(bonus.amount)} from ${bonus.attribute} (${amount(bonus.points)} × ${amount(bonus.perPoint)} per point)`}
      </p>
      {group.attack && (
        <p className="text-faint">
          Per attack at level 1, before armor, upgrades, items and auras.
        </p>
      )}
    </div>
  );
}

function ColorPicker({
  fields,
  editable,
  onChange,
}: Pick<FieldGroupRowProps, "fields" | "editable" | "onChange">) {
  const hex = rgbToHex(fields.map(numberOf));

  function pick(value: string) {
    const channels = hexToRgb(value);

    if (!channels) {
      return;
    }

    for (const [index, field] of fields.entries()) {
      const channel = String(channels[index]);

      if (channel !== field.raw) {
        onChange(field.id, channel);
      }
    }
  }

  return (
    <input
      type="color"
      value={hex}
      disabled={!editable}
      onChange={(e) => pick(e.target.value)}
      aria-label="Tint color"
      className="h-8 w-12 shrink-0 cursor-pointer rounded border border-field bg-well"
    />
  );
}

function ButtonGrid({
  fields,
  editable,
  onChange,
}: Pick<FieldGroupRowProps, "fields" | "editable" | "onChange">) {
  const [columnField, rowField] = fields;
  const column = numberOf(columnField);
  const row = numberOf(rowField);
  const slots = Array.from({ length: BUTTON_COLUMNS * BUTTON_ROWS }, (_, i) => ({
    x: i % BUTTON_COLUMNS,
    y: Math.floor(i / BUTTON_COLUMNS),
  }));

  function place(x: number, y: number) {
    if (columnField && String(x) !== columnField.raw) {
      onChange(columnField.id, String(x));
    }

    if (rowField && String(y) !== rowField.raw) {
      onChange(rowField.id, String(y));
    }
  }

  return (
    <fieldset className="grid w-fit grid-cols-4 gap-1 rounded bg-surface-0 p-1">
      <legend className="sr-only">Button position on the command card</legend>
      {slots.map(({ x, y }) => (
        <input
          key={`${x}-${y}`}
          type="radio"
          name={`button-${columnField?.id ?? "x"}`}
          checked={x === column && y === row}
          disabled={!editable}
          onChange={() => place(x, y)}
          aria-label={`Column ${x}, row ${y}`}
          className="size-6 cursor-pointer appearance-none rounded-sm border border-line bg-surface-2 checked:border-gold checked:bg-gold hover:border-faint"
        />
      ))}
    </fieldset>
  );
}

export function FieldGroupRow({
  group,
  fields,
  editable,
  damageBonus,
  onChange,
  onReset,
}: FieldGroupRowProps) {
  const raws = fields.map((f) => f.raw).join("\u0000");
  const [drafts, setDrafts] = useState(() => fields.map((f) => f.raw));
  const modified = fields.filter((f) => f.modified);
  const problems = fields.map((field, index) =>
    partProblem(drafts[index] ?? "", {
      label: group.parts[index] ?? field.name,
      min: field.min,
      max: field.max,
      integer: field.integer,
    }),
  );
  const [, dice = 0, sides = 0] = drafts.map(Number);
  let problem = problems.find((p) => p !== null) ?? null;

  if (group.kind === "dice" && problem === null) {
    problem = diceProblem(dice, sides);
  }

  useEffect(() => {
    setDrafts(raws.split("\u0000"));
  }, [raws]);

  function draft(index: number, value: string) {
    const next = [...drafts];

    next[index] = value;
    setDrafts(next);
  }

  function commit(index: number) {
    const field = fields[index];
    const value = drafts[index] ?? "";

    if (field && value !== field.raw && problem === null) {
      onChange(field.id, value);
    }
  }

  const description = fields.find((f) => f.description)?.description;
  const bases = fields.map((f) => f.base).join(" / ");

  return (
    <div
      className={classes(
        FIELD_ROW_GRID,
        "border-b border-line/60 px-4 py-1.5",
        modified.length > 0 && "bg-changed-soft shadow-[inset_2px_0_0_var(--color-changed)]",
      )}
    >
      <div className="min-w-0 pt-1">
        <span className="block" title={description ?? undefined}>
          {group.label}
        </span>
        <span className="block font-mono text-xs text-faint">
          {fields.map((f) => f.id).join(" · ")}
        </span>
      </div>

      <div className="flex flex-col gap-1.5">
        {group.kind === "button" && (
          <ButtonGrid fields={fields} editable={editable} onChange={onChange} />
        )}
        <div className="flex items-end gap-2">
          {group.kind === "color" && (
            <ColorPicker fields={fields} editable={editable} onChange={onChange} />
          )}
          {fields.map((field, index) => (
            <Part
              key={field.id}
              field={field}
              label={group.parts[index] ?? field.name}
              draft={drafts[index] ?? ""}
              problem={problems[index] ?? null}
              editable={editable}
              onDraft={(value) => draft(index, value)}
              onCommit={() => commit(index)}
            />
          ))}
        </div>
        {problem && (
          <p role="alert" className="text-xs text-danger">
            {problem}
          </p>
        )}
        {group.kind === "dice" && !problem && (
          <DiceResult group={group} values={drafts.map(Number)} damageBonus={damageBonus} />
        )}
      </div>

      <span className="truncate pt-1.5 text-sm text-faint" title={bases}>
        {bases}
      </span>

      {modified.length > 0 && editable && (
        <button
          type="button"
          onClick={() => {
            for (const field of modified) {
              onReset(field.id);
            }
          }}
          title="Reset to the base values"
          aria-label={`Reset ${group.label}`}
          className="mt-0.5 rounded p-1 text-faint hover:bg-surface-2 hover:text-ink"
        >
          <FlipBackward className="size-4" aria-hidden="true" />
        </button>
      )}
    </div>
  );
}
