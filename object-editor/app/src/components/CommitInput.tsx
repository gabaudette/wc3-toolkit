import { useEffect, useState } from "react";
import type { Field } from "../types";
import { TextPreview } from "./TextPreview";

type CommitInputProps = {
  label: string;
  field: Field;
  numeric?: boolean;
  /** A text area with the in-game preview below, for tooltips. */
  multiline?: boolean;
  onCommit: (value: string) => void;
};

const TEXT_ROWS = 3;
const boxClass =
  "rounded-[3px] border border-field bg-well px-3 py-1.5 text-ink outline-none focus:border-focus";

/**
 * A text box that saves when it loses focus, or on Enter in a single line.
 * The spell wizard's fields, which save into the spell as they are left.
 */
export function CommitInput({ label, field, numeric, multiline, onCommit }: CommitInputProps) {
  const [value, setValue] = useState(field.raw);

  useEffect(() => setValue(field.raw), [field.raw]);

  function commit() {
    if (value !== field.raw) {
      onCommit(value);
    }
  }

  if (multiline) {
    return (
      <label className="flex flex-col gap-1 text-sm text-muted">
        {label}
        <textarea
          value={value}
          rows={TEXT_ROWS}
          onChange={(e) => setValue(e.target.value)}
          onBlur={commit}
          className={boxClass}
        />
        <TextPreview text={value} />
      </label>
    );
  }

  return (
    <label className="flex flex-col gap-1 text-sm text-muted">
      {label}
      <input
        value={value}
        {...(numeric && { type: "number", min: field.min ?? 1, max: field.max ?? undefined })}
        onChange={(e) => setValue(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            commit();
          }
        }}
        className={boxClass}
      />
    </label>
  );
}
