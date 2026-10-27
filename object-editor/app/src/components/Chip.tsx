import { classes } from "../ui";

export function toggle<T>(list: T[], value: T): T[] {
  if (list.includes(value)) {
    return list.filter((v) => v !== value);
  }

  return [...list, value];
}

export function Chip({ label, on, onPress }: { label: string; on: boolean; onPress: () => void }) {
  return (
    <button
      type="button"
      onClick={onPress}
      aria-pressed={on}
      className={classes(
        "min-h-6 rounded-full border px-2.5 py-1 text-xs",
        on && "border-gold/60 bg-gold-soft text-gold",
        !on && "border-line text-faint hover:text-muted",
      )}
    >
      {label}
    </button>
  );
}
