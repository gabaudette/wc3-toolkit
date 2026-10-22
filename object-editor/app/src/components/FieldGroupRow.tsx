import { FlipBackward } from "@untitledui/icons";
import { type KeyboardEvent, useEffect, useState } from "react";
import {
  BUTTON_COLUMNS,
  BUTTON_ROWS,
  diceRange,
  type FieldGroup,
  hexToRgb,
  rgbToHex,
} from "../fieldGroups";
import type { DamageBonus, Field } from "../types";
import { classes } from "../ui";

type FieldGroupRowProps = {
  group: FieldGroup;
  fields: Field[];
  editable: boolean;
  damageBonus: DamageBonus | null;
  onChange: (fieldId: string, value: string) => void;
  onReset: (fieldId: string) => void;
};

const numberClass =
  "w-full rounded border border-line bg-surface-0 px-2 py-1 font-mono outline-none focus:border-gold read-only:opacity-70";

function numberOf(field: Field | undefined): number {
  const value = Number(field?.raw ?? "");
  if (Number.isFinite(value)) {
    return value;
  }

  return 0;
}

/** One small number box that applies on Enter or when leaving it. */
function Part({
  field,
  label,
  editable,
  onChange,
}: {
  field: Field;
  label: string;
  editable: boolean;
  onChange: (value: string) => void;
}) {
  const [draft, setDraft] = useState(field.raw);

  useEffect(() => {
    setDraft(field.raw);
  }, [field.raw]);

  function commit() {
    if (draft !== field.raw) {
      onChange(draft);
    }
  }

  function key(e: KeyboardEvent) {
    if (e.key === "Enter") {
      commit();
    }
  }

  return (
    <label className="flex min-w-0 flex-1 flex-col gap-0.5">
      <span className="text-[11px] text-faint">{label}</span>
      <input
        type="number"
        value={draft}
        readOnly={!editable}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={key}
        aria-label={`${label} (${field.name})`}
        className={classes(numberClass, field.modified && "border-changed")}
      />
    </label>
  );
}

function DiceResult({
  group,
  fields,
  damageBonus,
}: Omit<FieldGroupRowProps, "editable" | "onChange" | "onReset">) {
  const [base, dice, sides] = fields.map(numberOf);
  const bonus = (group.attack && damageBonus?.amount) || 0;
  const range = diceRange(base ?? 0, dice ?? 0, sides ?? 0, bonus);

  return (
    <p className="text-xs text-muted">
      Range <span className="font-mono text-ink">{range.min}</span> –{" "}
      <span className="font-mono text-ink">{range.max}</span>
      {bonus > 0 && damageBonus && ` (includes +${bonus} from ${damageBonus.attribute})`}
    </p>
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
      className="h-8 w-12 shrink-0 cursor-pointer rounded border border-line bg-surface-0"
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

/** Related fields edited in one row; each part still saves to its own field. */
export function FieldGroupRow({
  group,
  fields,
  editable,
  damageBonus,
  onChange,
  onReset,
}: FieldGroupRowProps) {
  const modified = fields.filter((f) => f.modified);
  const description = fields.find((f) => f.description)?.description;

  return (
    <div
      className={classes(
        "grid grid-cols-[minmax(0,1fr)_minmax(12rem,24rem)_1.75rem] items-start gap-x-3 border-b border-line/60 px-3 py-1.5",
        modified.length > 0 && "border-l-2 border-l-changed bg-changed-soft/40",
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
              editable={editable}
              onChange={(value) => onChange(field.id, value)}
            />
          ))}
        </div>
        {group.kind === "dice" && (
          <DiceResult group={group} fields={fields} damageBonus={damageBonus} />
        )}
      </div>

      {modified.length > 0 && editable && (
        <button
          type="button"
          onClick={() => {
            for (const field of modified) {
              onReset(field.id);
            }
          }}
          title="Reset to the base unit's values"
          aria-label={`Reset ${group.label}`}
          className="mt-0.5 rounded p-1 text-faint hover:bg-surface-2 hover:text-ink"
        >
          <FlipBackward className="size-4" aria-hidden="true" />
        </button>
      )}
    </div>
  );
}
