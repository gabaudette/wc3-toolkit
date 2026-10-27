import { Check } from "@untitledui/icons";
import type { ReactNode } from "react";
import {
  Checkbox as AriaCheckbox,
  type CheckboxProps as AriaCheckboxProps,
} from "react-aria-components";
import { classes } from "../ui";

type CheckboxProps = Omit<AriaCheckboxProps, "children" | "className"> & {
  children: ReactNode;
  className?: string;
};

/**
 * The box: a `field` edge in a `well`; on, it fills with `gold` and shows a
 * dark check, so the state reads by shape as well as color. Lives inside a
 * react-aria checkbox, whose `group` data attributes it follows.
 */
function Box({ small }: { small?: boolean }) {
  return (
    <span
      aria-hidden="true"
      className={classes(
        "flex shrink-0 items-center justify-center rounded-[3px] border border-field bg-well text-well",
        "group-data-[selected]:border-gold group-data-[selected]:bg-gold",
        // Windows high contrast: the check takes the text color on the system background.
        "forced-colors:group-data-[selected]:bg-[Canvas] forced-colors:text-[CanvasText]",
        small && "size-3.5",
        !small && "size-4",
      )}
    >
      <Check className="size-3 opacity-0 group-data-[selected]:opacity-100" strokeWidth={3} />
    </span>
  );
}

/** A checkbox with its label: "Changed only", "Use HD art", a unit to share. */
export function Checkbox({ children, className, ...props }: CheckboxProps) {
  return (
    <AriaCheckbox
      {...props}
      className={classes(
        "group flex min-h-6 cursor-default items-center gap-2 rounded-[3px] text-muted outline-none data-[disabled]:opacity-50",
        "data-[focus-visible]:outline-2 data-[focus-visible]:outline-solid data-[focus-visible]:outline-offset-2 data-[focus-visible]:outline-focus",
        className,
      )}
    >
      <Box />
      {children}
    </AriaCheckbox>
  );
}

type CheckChipProps = {
  value: string;
  children: ReactNode;
  /** Shown on hover, such as the code the game stores. */
  title?: string;
  /** On where the base object is off, or off where it is on. */
  changed?: boolean;
};

/**
 * One option of a multi-checkbox field, as a chip sized to its label: off
 * is quiet text on a `field` edge; on is gold (`gold-soft` fill, `gold` edge,
 * `gold-bright` semibold text) with a checked box. A gold dot after the label
 * marks an option that differs from the base. Goes in a `CheckboxGroup`.
 */
export function CheckChip({ value, children, title, changed }: CheckChipProps) {
  return (
    <AriaCheckbox
      value={value}
      className={classes(
        "group flex min-h-6 cursor-default items-center gap-1.5 rounded-[3px] border px-2 py-0.5 text-sm text-muted outline-none",
        "border-field data-[hovered]:text-ink",
        "data-[selected]:border-gold data-[selected]:bg-gold-soft data-[selected]:font-semibold data-[selected]:text-gold-bright",
        "data-[focus-visible]:outline-2 data-[focus-visible]:outline-solid data-[focus-visible]:outline-offset-1 data-[focus-visible]:outline-focus",
      )}
    >
      <span title={title} className="flex items-center gap-1.5">
        <Box small />
        {children}
        {changed && (
          <span
            title="Differs from the base value"
            className="size-1.5 shrink-0 rounded-full bg-gold forced-colors:bg-[CanvasText]"
          >
            <span className="sr-only"> (changed)</span>
          </span>
        )}
      </span>
    </AriaCheckbox>
  );
}
