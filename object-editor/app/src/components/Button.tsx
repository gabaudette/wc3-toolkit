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
        "rounded-md border font-medium outline-none transition-colors",
        "focus-visible:ring-2 focus-visible:ring-gold/60 disabled:cursor-not-allowed disabled:opacity-40",
        small && "px-2 py-1 text-xs",
        !small && "px-3 py-1.5 text-sm",
        primary && "border-gold bg-gold text-surface-0 hover:bg-gold/90",
        !primary && "border-line bg-surface-2 text-ink hover:border-faint",
      )}
    >
      {children}
    </AriaButton>
  );
}
