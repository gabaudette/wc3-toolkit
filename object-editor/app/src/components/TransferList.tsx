import type { FileStatus, TransferFile, TransferKind, TransferObject } from "../types";
import { classes, fileSize, objectCount } from "../ui";
import { UnitIcon } from "./UnitIcon";

type TransferListProps = {
  objects: TransferObject[];
  files: TransferFile[];
  warnings: string[];
};

const FILE_STATUS_LABELS: Record<FileStatus, string> = {
  new: "added",
  same: "already in this map",
  different: "this map keeps its own",
};

const KIND_LABELS: Record<TransferKind, string> = {
  unit: "Unit",
  ability: "Ability",
  buff: "Buff",
  item: "Item",
  upgrade: "Upgrade",
};

/**
 * The objects a share code, bundle or import holds, with new IDs, the
 * imported files that go with them, and what stays behind.
 */
export function TransferList({ objects, files, warnings }: TransferListProps) {
  const dependencies = objects.filter((o) => o.dependency).length;

  return (
    <div className="flex flex-col gap-2">
      <p className="text-sm text-muted">
        {objectCount(objects.length)}
        {dependencies > 0 && `, ${dependencies} brought along because the chosen units use them`}.
      </p>
      <ul
        className="max-h-64 overflow-y-auto rounded-[3px] border border-line"
        aria-label="Objects"
      >
        {objects.map((object) => (
          <li
            key={`${object.kind}-${object.id}`}
            className="flex items-center gap-3 px-3 py-1 text-sm"
          >
            <UnitIcon path={object.icon} size="small" />
            <span className="min-w-0 flex-1 truncate">
              {object.name}
              {object.dependency && <span className="text-faint"> · brought along</span>}
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
      {files.length > 0 && (
        <>
          <p className="text-sm text-muted">
            {files.length} imported file{files.length > 1 && "s"} (models, textures, icons)
          </p>
          <ul
            className="max-h-40 overflow-y-auto rounded-[3px] border border-line"
            aria-label="Imported files"
          >
            {files.map((file) => (
              <li key={file.path} className="flex items-center gap-3 px-3 py-1 text-xs">
                <span className="min-w-0 flex-1 truncate font-mono" title={file.path}>
                  {file.path}
                </span>
                <span className="text-faint">{fileSize(file.size)}</span>
                {file.status && (
                  <span
                    className={classes(
                      file.status === "different" && "text-gold",
                      file.status !== "different" && "text-muted",
                    )}
                  >
                    {FILE_STATUS_LABELS[file.status]}
                  </span>
                )}
              </li>
            ))}
          </ul>
        </>
      )}
      {warnings.length > 0 && (
        <div className="rounded-[3px] border border-gold/40 bg-gold-soft p-2 text-xs">
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
