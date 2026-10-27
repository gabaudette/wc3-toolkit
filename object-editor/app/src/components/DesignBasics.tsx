import { Trash01 } from "@untitledui/icons";
import { Button } from "./Button";
import { ColorSwatches } from "./ColorSwatches";

export const designInputClass =
  "rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-ink outline-none focus:border-focus disabled:opacity-60";

type DesignBasicsProps = {
  name: string;
  title: string;
  titleColor: string | null;
  /** The Lua call to show under the name, such as `UI.Kills.AddValue(player, 1)`. */
  lua: string;
  editable: boolean;
  onChange: (change: { name?: string; title?: string; titleColor?: string | null }) => void;
  onDelete: () => void;
};

/** What every design has on top: its name (and Lua call), Delete, and a colored title. */
export function DesignBasics({
  name,
  title,
  titleColor,
  lua,
  editable,
  onChange,
  onDelete,
}: DesignBasicsProps) {
  return (
    <>
      <div className="flex items-end gap-3">
        <label className="flex min-w-0 flex-1 flex-col gap-1 text-sm text-muted">
          Name
          <input
            value={name}
            disabled={!editable}
            onChange={(e) => onChange({ name: e.target.value })}
            className={designInputClass}
          />
        </label>
        <Button onPress={onDelete} disabled={!editable} label={`Delete ${name}`}>
          <span className="flex items-center gap-1.5">
            <Trash01 className="size-4" aria-hidden="true" />
            Delete
          </span>
        </Button>
      </div>
      <p className="-mt-2 text-xs text-faint">
        In Lua: <code className="font-mono text-muted">{lua}</code>
      </p>

      <label className="flex flex-col gap-1 text-sm text-muted">
        Title
        <input
          value={title}
          disabled={!editable}
          onChange={(e) => onChange({ title: e.target.value })}
          className={designInputClass}
        />
      </label>
      <div className="-mt-2">
        <ColorSwatches
          value={titleColor}
          onChange={(color) => onChange({ titleColor: color })}
          label="Title color"
          none={{ label: "Game default", swatch: "#ffffff" }}
          disabled={!editable}
        />
      </div>
    </>
  );
}

/** A 16:9 game screen for previews; children are placed on it. */
export function GameScreen({ children }: { children: React.ReactNode }) {
  return (
    <figure className="flex min-w-0 flex-col gap-1.5">
      <figcaption className="text-xs text-faint">Preview, as the game shows it</figcaption>
      <div className="@container relative aspect-video overflow-hidden rounded-[3px] border border-line bg-[radial-gradient(ellipse_at_center,#2b3a2a_0%,#141c14_70%)]">
        {children}
      </div>
    </figure>
  );
}
