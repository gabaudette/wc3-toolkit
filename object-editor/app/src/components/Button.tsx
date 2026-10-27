import type { ReactNode } from "react";
import { Button as AriaButton } from "react-aria-components";
import { classes } from "../ui";

type ButtonProps = {
  children: ReactNode;
  onPress: () => void;
  primary?: boolean;
  disabled?: boolean;
  small?: boolean;
  label?: string;
};

export function Button({ children, onPress, primary, disabled, small, label }: ButtonProps) {
  return (
    <AriaButton
      onPress={onPress}
      isDisabled={disabled ?? false}
      {...(label && { "aria-label": label })}
      className={classes(
        "rounded-[3px] border font-semibold outline-none transition-colors",
        "focus-visible:ring-2 focus-visible:ring-focus disabled:cursor-not-allowed disabled:opacity-40",
        small && "px-2 py-1 text-xs",
        !small && "px-3 py-1.5 text-sm",
        primary && "border-gold bg-gold-soft text-gold-bright hover:bg-frame-inner",
        !primary && "border-field bg-surface-2 text-ink hover:border-gold",
      )}
    >
      {children}
    </AriaButton>
  );
}
