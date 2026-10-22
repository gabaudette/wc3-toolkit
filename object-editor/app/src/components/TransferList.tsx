import type { TransferKind, TransferObject } from "../types";
import { objectCount } from "../ui";
import { UnitIcon } from "./UnitIcon";

type TransferListProps = {
  objects: TransferObject[];
  warnings: string[];
};

const KIND_LABELS: Record<TransferKind, string> = {
  unit: "Unit",
  ability: "Ability",
  buff: "Buff",
};

/** The objects a share code or import holds, with new IDs and what stays behind. */
export function TransferList({ objects, warnings }: TransferListProps) {
  const dependencies = objects.filter((o) => o.dependency).length;

  return (
    <div className="flex flex-col gap-2">
      <p className="text-sm text-muted">
        {objectCount(objects.length)}
        {dependencies > 0 && `, ${dependencies} brought along because the chosen units use them`}.
      </p>
      <ul className="max-h-64 overflow-y-auto rounded-md border border-line" aria-label="Objects">
        {objects.map((object) => (
          <li
            key={`${object.kind}-${object.id}`}
            className="flex items-center gap-3 px-3 py-1 text-sm"
          >
            <UnitIcon path={object.icon} size="small" />
            <span className="min-w-0 flex-1 truncate">
              {object.name}
              {object.dependency && <span className="text-faint"> · used by the selection</span>}
            </span>
            <span className="text-xs text-faint">{KIND_LABELS[object.kind]}</span>
            <span className="font-mono text-xs text-muted">
              {object.id}
              {object.newId && object.newId !== object.id && (
                <span className="text-gold"> → {object.newId}</span>
              )}
            </span>
          </li>
        ))}
      </ul>
      {warnings.length > 0 && (
        <div className="rounded-md border border-gold/40 bg-gold-soft p-2 text-xs">
          <p className="mb-1 font-medium text-gold">Not included</p>
          <ul className="flex max-h-32 list-disc flex-col gap-0.5 overflow-y-auto pl-4 text-muted">
            {warnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
