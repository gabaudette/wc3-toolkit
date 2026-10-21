import { FlipBackward, InfoCircle } from "@untitledui/icons";
import { type KeyboardEvent, useEffect, useState } from "react";
import { Switch } from "react-aria-components";
import type { Field } from "../types";
import { classes } from "../ui";

type FieldRowProps = {
  field: Field;
  editable: boolean;
  onChange: (value: string) => void;
  onReset: () => void;
};

const TRUE = "1";
const FALSE = "0";
const REAL_STEP = 0.01;
const INTEGER_STEP = 1;
const LONG_TEXT_ROWS = 3;

const inputClass =
  "w-full rounded border border-line bg-surface-0 px-2 py-1 outline-none focus:border-gold read-only:opacity-70";

function numberStep(field: Field): number {
  if (field.integer) {
    return INTEGER_STEP;
  }

  return REAL_STEP;
}

/** Commits a typed value on Enter or blur, so every keystroke is not an edit. */
function useDraft(field: Field, onChange: (value: string) => void) {
  const [draft, setDraft] = useState(field.raw);

  useEffect(() => {
    setDraft(field.raw);
  }, [field.raw]);

  function commit() {
    if (draft !== field.raw) {
      onChange(draft);
    }
  }

  return { draft, setDraft, commit };
}

function Editor({ field, editable, onChange }: Omit<FieldRowProps, "onReset">) {
  const { draft, setDraft, commit } = useDraft(field, onChange);
  const commitOnEnter = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      commit();
    }
  };

  if (field.control === "bool") {
    return (
      <Switch
        isSelected={field.raw === TRUE}
        isReadOnly={!editable}
        onChange={(on) => {
          if (on) {
            onChange(TRUE);
          } else {
            onChange(FALSE);
          }
        }}
        className="group flex items-center gap-2 text-muted"
      >
        <span className="h-5 w-9 rounded-full border border-line bg-surface-0 p-0.5 transition-colors group-data-[selected]:border-gold group-data-[selected]:bg-gold-soft">
          <span className="block size-3.5 rounded-full bg-faint transition-transform group-data-[selected]:translate-x-4 group-data-[selected]:bg-gold" />
        </span>
        {field.display}
      </Switch>
    );
  }

  if (field.control === "select") {
    return (
      <select
        value={field.raw}
        disabled={!editable}
        onChange={(e) => onChange(e.target.value)}
        aria-label={field.label}
        className={inputClass}
      >
        {!field.options.some((o) => o.value.toLowerCase() === field.raw.toLowerCase()) && (
          <option value={field.raw}>{field.display || "(none)"}</option>
        )}
        {field.options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    );
  }

  if (field.control === "number") {
    return (
      <input
        type="number"
        value={draft}
        readOnly={!editable}
        step={numberStep(field)}
        {...(field.min !== null && { min: field.min })}
        {...(field.max !== null && { max: field.max })}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={commitOnEnter}
        aria-label={field.label}
        className={classes(inputClass, "font-mono")}
      />
    );
  }

  if (field.control === "longText") {
    return (
      <textarea
        value={draft}
        readOnly={!editable}
        rows={LONG_TEXT_ROWS}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        aria-label={field.label}
        className={inputClass}
      />
    );
  }

  return (
    <div>
      <input
        value={draft}
        readOnly={!editable}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={commitOnEnter}
        aria-label={field.label}
        className={classes(inputClass, field.control === "codes" && "font-mono text-xs")}
      />
      {field.control === "codes" && field.display && field.display !== field.raw && (
        <p className="mt-0.5 truncate text-xs text-faint" title={field.display}>
          {field.display}
        </p>
      )}
    </div>
  );
}

const numberFormat = new Intl.NumberFormat();

/** "Range 1 – 500,000", "At most 5" or nothing, from the field's limits. */
function rangeText(field: Field): string | null {
  const { min, max } = field;
  if (min !== null && max !== null) {
    return `Range ${numberFormat.format(min)} – ${numberFormat.format(max)}`;
  }

  if (max !== null) {
    return `At most ${numberFormat.format(max)}`;
  }

  if (min !== null) {
    return `At least ${numberFormat.format(min)}`;
  }

  return null;
}

function Description({ field }: { field: Field }) {
  const range = rangeText(field);

  return (
    <div className="col-span-3 mb-1 rounded bg-surface-1 px-3 py-2 text-sm leading-relaxed text-muted">
      <p>{field.description}</p>
      {(range || field.unverified) && (
        <p className="mt-1 flex flex-wrap items-center gap-3 text-xs text-faint">
          {range && <span>{range}</span>}
          {field.unverified && (
            <span className="rounded bg-gold-soft px-1.5 py-0.5 text-gold">
              Unverified: not yet confirmed in game
            </span>
          )}
        </p>
      )}
    </div>
  );
}

export function FieldRow({ field, editable, onChange, onReset }: FieldRowProps) {
  const [showInfo, setShowInfo] = useState(false);

  return (
    <div
      className={classes(
        "grid grid-cols-[minmax(0,1fr)_minmax(12rem,24rem)_1.75rem] items-start gap-x-3 border-b border-line/60 px-3 py-1",
        field.modified && "border-l-2 border-l-changed bg-changed-soft/40",
      )}
    >
      <div className="flex min-w-0 items-baseline gap-2 pt-1">
        <span className="truncate" title={field.name}>
          {field.name}
        </span>
        <span className="shrink-0 font-mono text-xs text-faint">{field.id}</span>
        {field.description && (
          <button
            type="button"
            onClick={() => setShowInfo(!showInfo)}
            aria-expanded={showInfo}
            aria-label={`What ${field.name} does`}
            title="What this field does"
            className={classes(
              "shrink-0 self-center rounded p-0.5 hover:text-ink",
              showInfo && "text-gold",
              !showInfo && "text-faint",
            )}
          >
            <InfoCircle className="size-3.5" aria-hidden="true" />
          </button>
        )}
      </div>
      <Editor field={field} editable={editable} onChange={onChange} />
      {field.modified && editable && (
        <button
          type="button"
          onClick={onReset}
          title="Reset to the base unit's value"
          aria-label={`Reset ${field.name}`}
          className="mt-0.5 rounded p-1 text-faint hover:bg-surface-2 hover:text-ink"
        >
          <FlipBackward className="size-4" aria-hidden="true" />
        </button>
      )}
      {showInfo && field.description && <Description field={field} />}
    </div>
  );
}
