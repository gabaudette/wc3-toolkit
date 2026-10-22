import { FlipBackward, InfoCircle } from "@untitledui/icons";
import { useEffect, useState } from "react";
import type { Field, LevelCell, LevelField } from "../types";
import { classes } from "../ui";
import { ColoredTextInput } from "./ColoredTextInput";
import { Description } from "./FieldRow";
import { TextPreview } from "./TextPreview";

type LevelTextProps = {
  field: LevelField;
  editable: boolean;
  onChange: (level: number, value: string) => void;
  onReset: (level: number) => void;
  /** Shows the field in the page's inspector, when the page has one. */
  onInspect?: (field: Field) => void;
};

const LONG_TEXT_ROWS = 3;
/** Game text with color codes or placeholders gets a preview even on one line. */
const FORMATTED = /[|<]/;

const inputClass =
  "w-full rounded border border-field bg-well px-2 py-1 outline-none focus:border-glow read-only:opacity-70";

type LevelInputProps = {
  field: LevelField;
  cell: LevelCell;
  level: number;
  editable: boolean;
  onChange: (value: string) => void;
  onReset: () => void;
};

function LevelInput({ field, cell, level, editable, onChange, onReset }: LevelInputProps) {
  const [draft, setDraft] = useState(cell.raw);
  const multiline = field.field.control === "longText";

  useEffect(() => {
    setDraft(cell.raw);
  }, [cell.raw]);

  return (
    <div
      className={classes(
        "grid grid-cols-[4.5rem_minmax(0,1fr)_1.75rem] items-start gap-x-2 py-1",
        cell.modified && "rounded bg-changed-soft/40",
      )}
    >
      <span
        className={classes(
          "pt-1 text-xs",
          cell.modified && "text-changed",
          !cell.modified && "text-faint",
        )}
      >
        Level {level}
      </span>
      <div>
        <ColoredTextInput
          value={draft}
          multiline={multiline}
          readOnly={!editable}
          rows={LONG_TEXT_ROWS}
          label={`${field.field.name}, level ${level}`}
          className={inputClass}
          onDraft={setDraft}
          onCommit={(text) => {
            if (text !== cell.raw) {
              onChange(text);
            }
          }}
        />
        {(multiline || FORMATTED.test(draft)) && <TextPreview text={draft} />}
      </div>
      {cell.modified && editable && (
        <button
          type="button"
          onClick={onReset}
          title="Reset to the base value"
          aria-label={`Reset ${field.field.name}, level ${level}`}
          className="mt-0.5 rounded p-1 text-faint hover:bg-surface-2 hover:text-ink"
        >
          <FlipBackward className="size-4" aria-hidden="true" />
        </button>
      )}
    </div>
  );
}

/** A text field with one value per level (tooltips), each level under the field's name. */
export function LevelText({ field, editable, onChange, onReset, onInspect }: LevelTextProps) {
  const [showInfo, setShowInfo] = useState(false);
  const meta = field.field;
  const changed = field.cells.some((c) => c.modified);

  return (
    <div
      className={classes(
        "border-b border-line/60 px-3 py-1.5",
        changed && "border-l-2 border-l-changed",
      )}
    >
      <div className="flex items-baseline gap-2">
        <span>{meta.name}</span>
        <span className="font-mono text-xs text-faint">{meta.id}</span>
        {meta.description && (
          <button
            type="button"
            onClick={() => {
              if (onInspect) {
                onInspect(meta);
                return;
              }

              setShowInfo(!showInfo);
            }}
            aria-expanded={showInfo}
            aria-label={`What ${meta.name} does`}
            title="What this field does"
            className={classes(
              "self-center -my-1 rounded p-1.5 hover:text-ink",
              showInfo && "text-gold",
              !showInfo && "text-faint",
            )}
          >
            <InfoCircle className="size-3.5" aria-hidden="true" />
          </button>
        )}
      </div>
      {showInfo && meta.description && <Description field={meta} />}
      {field.cells.map((cell, index) => {
        const level = index + 1;
        return (
          <LevelInput
            key={level}
            field={field}
            cell={cell}
            level={level}
            editable={editable}
            onChange={(value) => onChange(level, value)}
            onReset={() => onReset(level)}
          />
        );
      })}
    </div>
  );
}
