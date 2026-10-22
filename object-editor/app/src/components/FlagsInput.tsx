import { hasBit, inList, withBit, withListItem } from "../flags";
import type { Field } from "../types";

type FlagsInputProps = {
  field: Field;
  /** The value being edited, which may differ from `field.raw`. */
  raw: string;
  editable: boolean;
  onChange: (value: string) => void;
};

/** Checkboxes for bit flags (`flags`) and lists of choices (`flagList`, like targets). */
export function FlagsInput({ field, raw, editable, onChange }: FlagsInputProps) {
  const bits = field.control === "flags";

  return (
    <fieldset className="grid grid-cols-[repeat(auto-fill,minmax(8rem,1fr))] gap-x-3 gap-y-0.5">
      <legend className="sr-only">{field.label}</legend>
      {field.options.map((choice) => {
        let on = inList(raw, choice);
        if (bits) {
          on = hasBit(raw, choice);
        }

        return (
          <label key={choice.value} className="flex items-center gap-1.5 text-sm text-muted">
            <input
              type="checkbox"
              checked={on}
              disabled={!editable}
              onChange={(e) => {
                if (bits) {
                  onChange(withBit(raw, choice, e.target.checked));
                } else {
                  onChange(withListItem(raw, choice, e.target.checked));
                }
              }}
            />
            <span className="truncate" title={choice.value}>
              {choice.label}
            </span>
          </label>
        );
      })}
    </fieldset>
  );
}
