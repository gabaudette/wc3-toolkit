import { classes } from "../ui";

/** The player colors, as the game draws them; most UI text picks from these. */
const PLAYER_COLORS: { label: string; value: string }[] = [
  { label: "White", value: "ffffff" },
  { label: "Red", value: "ff0303" },
  { label: "Blue", value: "0042ff" },
  { label: "Teal", value: "1ce6b9" },
  { label: "Yellow", value: "fffc00" },
  { label: "Orange", value: "fe8a0e" },
  { label: "Green", value: "20c000" },
  { label: "Pink", value: "e55bb0" },
  { label: "Light blue", value: "7ebff1" },
  { label: "Gray", value: "959697" },
];

type ColorSwatchesProps = {
  value: string | null;
  onChange: (value: string | null) => void;
  label: string;
  none: { label: string; swatch: string };
  disabled?: boolean;
};

export function ColorSwatches({ value, onChange, label, none, disabled }: ColorSwatchesProps) {
  const colors = [
    { label: none.label, value: null, swatch: none.swatch },
    ...PLAYER_COLORS.filter((c) => `#${c.value}` !== none.swatch).map((c) => ({
      ...c,
      swatch: `#${c.value}`,
    })),
  ];

  return (
    <fieldset className="flex flex-wrap items-center gap-1.5">
      <legend className="sr-only">{label}</legend>
      <span className="mr-1 text-xs text-faint">Color</span>
      {colors.map((color) => {
        const picked = (value?.toLowerCase() ?? null) === color.value;

        return (
          <button
            key={color.label}
            type="button"
            disabled={disabled}
            onClick={() => onChange(color.value)}
            aria-pressed={picked}
            aria-label={color.label}
            title={color.label}
            className={classes(
              "size-6 rounded-full border-2 disabled:opacity-50",
              picked && "border-ink",
              !picked && "border-transparent",
            )}
          >
            <span
              className="block size-full rounded-full border border-black/40"
              style={{ background: color.swatch }}
            />
          </button>
        );
      })}
    </fieldset>
  );
}
