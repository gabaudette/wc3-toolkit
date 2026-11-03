import { type ReactNode, useId } from "react";
import { CheckboxGroup, Group } from "react-aria-components";
import { choiceGroups, hasBit, inList, withBit, withListItem } from "../flags";
import type { Choice, Field } from "../types";
import { CheckChip } from "./Checkbox";

function ChoiceGroup({ label, children }: { label: string; children: ReactNode }) {
  const id = useId();

  return (
    <Group aria-labelledby={id}>
      <p id={id} className="mb-1 font-display text-sm font-bold text-muted">
        {label}
      </p>
      {children}
    </Group>
  );
}

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

  function chips(choices: Choice[]) {
    return (
      <div className="grid grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] gap-1.5">
        {choices.map((choice) => (
          <CheckChip
            key={choice.value}
            value={choice.value}
            title={choice.value}
            changed={baseRaw !== undefined && isOn(baseRaw, choice) !== isOn(raw, choice)}
          >
            {choice.label}
          </CheckChip>
        ))}
      </div>
    );
  }

  return (
    <CheckboxGroup
      aria-label={field.label}
      value={selected}
      onChange={toggle}
      isReadOnly={!editable}
      className="flex flex-col gap-2.5"
    >
      {choiceGroups(field.valueType, field.options).map(({ label, choices }) => {
        if (!label) {
          return <div key="all">{chips(choices)}</div>;
        }

        return (
          <ChoiceGroup key={label} label={label}>
            {chips(choices)}
          </ChoiceGroup>
        );
      })}
    </CheckboxGroup>
  );
}
