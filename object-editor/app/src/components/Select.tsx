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

/**
 * A field's choices for a `Select` and the one to show: the stored value
 * matched without case, or, when it is none of them, an extra option for it
 * labeled with its display text.
 */
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

/** Keys never empty: react-aria treats an empty key as no key. */
const KEY_PREFIX = "v:";

type SelectProps = {
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
  /** Its accessible name, when no visible label points at it. */
  label: string;
  isDisabled?: boolean;
  /** Extra classes for the button, such as a width. */
  className?: string;
};

/** A react-aria key for an option's value. */
export function optionKey(value: string): string {
  return KEY_PREFIX + value;
}

/** The value back from a key made by `optionKey`. */
export function optionValue(key: string | number): string {
  return String(key).slice(KEY_PREFIX.length);
}

/**
 * One choice in a list (a `Select`'s or a `ListBox` of our own): `ink` text,
 * `surface-2` under focus, gold with a check when it is the current one.
 */
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

/**
 * The app's drop-down: a button drawn like an input (`well`, `field` edge, a
 * chevron) and a list in a panel; the current option is gold with a check,
 * as "on" is for checkboxes. Replaces native selects, whose open list
 * WebKitGTK draws as a grey system menu no CSS reaches.
 */
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
        {/* Only the text: the option's own content carries the list's check mark. */}
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
