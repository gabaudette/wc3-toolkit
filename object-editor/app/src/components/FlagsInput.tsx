import { CheckboxGroup } from "react-aria-components";
import { hasBit, inList, withBit, withListItem } from "../flags";
import type { Choice, Field } from "../types";
import { CheckChip } from "./Checkbox";

type FlagsInputProps = {
  field: Field;
  raw: string;
  baseRaw?: string;
  editable: boolean;
  onChange: (value: string) => void;
};

export function FlagsInput({ field, raw, baseRaw, editable, onChange }: FlagsInputProps) {
  const bits = field.control === "flags";
  const isOn = (value: string, choice: Choice) => {
    if (bits) {
      return hasBit(value, choice);
    }

    return inList(value, choice);
  };
  const selected = field.options.filter((choice) => isOn(raw, choice)).map((c) => c.value);

  function toggle(values: string[]) {
    const changed = field.options.find(
      (choice) => values.includes(choice.value) !== selected.includes(choice.value),
    );

    if (!changed) {
      return;
    }

    const on = values.includes(changed.value);

    if (bits) {
      onChange(withBit(raw, changed, on));
    } else {
      onChange(withListItem(raw, changed, on));
    }
  }

  return (
    <CheckboxGroup
      aria-label={field.label}
      value={selected}
      onChange={toggle}
      isReadOnly={!editable}
      className="flex flex-wrap gap-1.5"
    >
      {field.options.map((choice) => (
        <CheckChip
          key={choice.value}
          value={choice.value}
          title={choice.value}
          changed={baseRaw !== undefined && isOn(baseRaw, choice) !== isOn(raw, choice)}
        >
          {choice.label}
        </CheckChip>
      ))}
    </CheckboxGroup>
  );
}
