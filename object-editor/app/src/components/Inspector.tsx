import { AlertTriangle, FlipBackward } from "@untitledui/icons";
import type { Field, HotkeyClash } from "../types";
import { Button } from "./Button";
import { rangeText } from "./FieldRow";
import { GameText, plainGameText } from "./GameText";
import { ModelViewer } from "./ModelViewer";

type InspectorProps = {
  /** The model to preview: `null` shows "No model"; leave it out for objects without models. */
  modelPath?: string | null;
  /** The field last focused on the page. */
  field: Field | null;
  /** The field has a value per level: its base is level 1's, and it resets in its table. */
  perLevel?: boolean;
  editable: boolean;
  /** Hotkey clashes; leave out for objects the check does not cover. */
  clashes?: HotkeyClash[];
  onReset: (field: string) => void;
};

/** What the selected field does, its limits, and its base value. */
function FieldHelp({
  field,
  perLevel,
  editable,
  onReset,
}: {
  field: Field;
  perLevel: boolean;
  editable: boolean;
  onReset: (field: string) => void;
}) {
  const range = rangeText(field);

  return (
    <div className="flex flex-col gap-1.5">
      <h3 className="font-display text-xl leading-tight font-bold text-gold-bright">
        {field.name}
      </h3>
      <p className="font-mono text-xs text-faint">
        {field.id} · {field.valueType}
        {range && ` · ${range}`}
      </p>
      {field.description && <p className="leading-relaxed text-ink">{field.description}</p>}
      {!field.description && <p className="text-muted">No description yet.</p>}
      {field.unverified && (
        <p className="self-start rounded-[3px] bg-gold-soft px-1.5 py-0.5 text-xs text-gold">
          Not yet confirmed in game
        </p>
      )}
      <p className="text-sm text-muted">
        Base value{perLevel && " at level 1"}:{" "}
        <span className="text-ink">
          {field.base && <GameText text={field.base} />}
          {!field.base && "empty"}
        </span>
      </p>
      {perLevel && (
        <p className="text-sm text-muted">Each level has its own value; reset them in the table.</p>
      )}
      {field.modified && editable && !perLevel && (
        <div className="self-start">
          <Button small onPress={() => onReset(field.id)}>
            <span className="flex items-center gap-1.5">
              <FlipBackward className="size-3.5" aria-hidden="true" />
              Restore the base value
            </span>
          </Button>
        </div>
      )}
    </div>
  );
}

/** Problems the editor can see in the object as the map has it. */
function Checks({ clashes }: { clashes: HotkeyClash[] }) {
  if (clashes.length === 0) {
    return <p className="text-muted">No hotkey clashes.</p>;
  }

  return (
    <ul className="flex flex-col gap-2" aria-label="Hotkey clashes">
      {clashes.map((clash) => (
        <li key={`${clash.menu}-${clash.key}`} className="flex gap-2 text-sm text-gold">
          <AlertTriangle className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
          <span>
            Hotkey <kbd className="rounded-[3px] bg-well px-1 font-mono text-ink">{clash.key}</kbd>{" "}
            is used by {clash.names.map(plainGameText).join(" and ")} ({clash.menu.toLowerCase()}).
            Only one of them reacts to the key.
          </span>
        </li>
      ))}
    </ul>
  );
}

/**
 * The page's right column: the model, help for the field being edited, and
 * checks such as hotkey clashes.
 */
export function Inspector({
  modelPath,
  field,
  perLevel = false,
  editable,
  clashes,
  onReset,
}: InspectorProps) {
  return (
    <aside
      aria-label="Inspector"
      className="panel flex w-80 shrink-0 flex-col gap-3 overflow-y-auto rounded-[3px] bg-surface-1 p-3"
    >
      {modelPath !== undefined && <ModelViewer path={modelPath} />}

      <section aria-label="Selected field" className="rounded-[3px] border border-line bg-well p-3">
        {field && (
          <FieldHelp field={field} perLevel={perLevel} editable={editable} onReset={onReset} />
        )}
        {!field && (
          <p className="text-muted">Select a field to see what it does and its base value.</p>
        )}
      </section>

      {clashes && (
        <section aria-label="Checks" className="rounded-[3px] border border-line bg-well p-3">
          <h3 className="mb-1.5 font-display text-lg font-bold text-gold">Checks</h3>
          <Checks clashes={clashes} />
        </section>
      )}
    </aside>
  );
}
