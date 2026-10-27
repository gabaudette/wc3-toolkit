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

function Box({ small }: { small?: boolean }) {
  return (
    <span
      aria-hidden="true"
      className={classes(
        "flex shrink-0 items-center justify-center rounded-[3px] border border-field bg-well text-well",
        "group-data-[selected]:border-gold group-data-[selected]:bg-gold",
        "forced-colors:group-data-[selected]:bg-[Canvas] forced-colors:text-[CanvasText]",
        small && "size-3.5",
        !small && "size-4",
      )}
    >
      <Check className="size-3 opacity-0 group-data-[selected]:opacity-100" strokeWidth={3} />
    </span>
  );
}

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
  title?: string;
  changed?: boolean;
};

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
