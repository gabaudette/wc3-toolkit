import { Check, ChevronDown } from "@untitledui/icons";
import {
  Select as AriaSelect,
  Button,
  ListBox,
  ListBoxItem,
  Popover,
  SelectValue,
} from "react-aria-components";
import { classes } from "../ui";

export type SelectOption = { value: string; label: string; disabled?: boolean };

export function fieldChoices(
  choices: { value: string; label: string }[],
  raw: string,
  display: string,
): { options: SelectOption[]; value: string } {
  const current = choices.find((c) => c.value.toLowerCase() === raw.toLowerCase());

  if (current) {
    return { options: choices, value: current.value };
  }

  return { options: [{ value: raw, label: display || "(none)" }, ...choices], value: raw };
}

const KEY_PREFIX = "v:";

type SelectProps = {
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
  label: string;
  isDisabled?: boolean;
  className?: string;
};

export function optionKey(value: string): string {
  return KEY_PREFIX + value;
}

export function optionValue(key: string | number): string {
  return String(key).slice(KEY_PREFIX.length);
}

export function Option({ option }: { option: SelectOption }) {
  return (
    <ListBoxItem
      id={optionKey(option.value)}
      textValue={option.label}
      className={classes(
        "group flex cursor-default items-center gap-2 px-2.5 py-1.5 text-sm text-ink outline-none",
        "data-[focused]:bg-surface-2 data-[selected]:font-semibold data-[selected]:text-gold-bright data-[disabled]:text-faint",
      )}
    >
      <Check
        className="size-3.5 shrink-0 text-gold opacity-0 group-data-[selected]:opacity-100"
        aria-hidden="true"
      />
      <span className="truncate">{option.label}</span>
    </ListBoxItem>
  );
}

export function Select({ value, options, onChange, label, isDisabled, className }: SelectProps) {
  return (
    <AriaSelect
      aria-label={label}
      selectedKey={optionKey(value)}
      onSelectionChange={(key) => {
        if (key !== null) {
          onChange(optionValue(key));
        }
      }}
      isDisabled={isDisabled ?? false}
      disabledKeys={options.filter((o) => o.disabled).map((o) => optionKey(o.value))}
      className="min-w-0"
    >
      <Button
        className={classes(
          "flex w-full min-w-0 items-center justify-between gap-2 rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-left text-ink outline-none",
          "data-[pressed]:border-focus data-[focus-visible]:border-focus data-[disabled]:opacity-60",
          className,
        )}
      >
        <SelectValue className="truncate data-[placeholder]:text-faint">
          {({ selectedText }) => selectedText}
        </SelectValue>
        <ChevronDown className="size-4 shrink-0 text-muted" aria-hidden="true" />
      </Button>
      <Popover
        offset={4}
        className="min-w-[var(--trigger-width)] overflow-auto rounded-[3px] border border-line-strong bg-surface-1 py-1 shadow-xl data-[entering]:animate-none"
      >
        <ListBox className="max-h-72 outline-none">
          {options.map((option) => (
            <Option key={option.value} option={option} />
          ))}
        </ListBox>
      </Popover>
    </AriaSelect>
  );
}
