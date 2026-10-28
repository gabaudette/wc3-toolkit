import { useEffect, useState } from "react";
import { api } from "../api";
import { mdxFromMdl, samePath, TEXTURE_STATUS } from "../assets";
import { failed, ok, type Result, settle } from "../result";
import type { ImportCandidate, ImportRequest } from "../types";
import { classes, counted, fileSize } from "../ui";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { DialogShell } from "./DialogShell";

export type ImportPlan = { files: string[]; folder: string | null };

type Row = ImportCandidate & { included: boolean };

type ImportAssetsDialogProps = {
  plan: ImportPlan | null;
  existing: string[];
  onImported: (paths: string[]) => void;
  onClose: () => void;
};

async function request(row: Row): Promise<Result<ImportRequest>> {
  const plain = { diskPath: row.diskPath, importPath: row.importPath.trim() };

  if (!row.convert) {
    return ok(plain);
  }

  const text = await api.readTextFile(row.diskPath);

  if (!text.ok) {
    return text;
  }

  const converted = mdxFromMdl(text.value);

  if (!converted.ok) {
    return failed(`Cannot convert ${row.diskPath}: ${converted.error}`);
  }

  return ok({ ...plain, converted: converted.value });
}

export function ImportAssetsDialog({
  plan,
  existing,
  onImported,
  onClose,
}: ImportAssetsDialogProps) {
  const [rows, setRows] = useState<Row[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    setRows([]);
    setError(null);

    if (!plan) {
      return;
    }

    void settle(
      api.planImport(plan.files, plan.folder),
      (candidates) => setRows(candidates.map((c) => ({ ...c, included: true }))),
      setError,
    );
  }, [plan]);

  const chosen = rows.filter((r) => r.included);
  const missing = chosen.reduce(
    (count, r) => count + r.textures.filter((t) => t.status === "missing").length,
    0,
  );

  function change(index: number, changed: Partial<Row>) {
    setRows(
      rows.map((r, at) => {
        if (at === index) {
          return { ...r, ...changed };
        }

        return r;
      }),
    );
  }

  async function add() {
    setBusy(true);
    setError(null);
    const requests: ImportRequest[] = [];

    for (const row of chosen) {
      const made = await request(row);

      if (!made.ok) {
        setError(made.error);
        setBusy(false);
        return;
      }

      requests.push(made.value);
    }

    const imported = await api.importAssets(requests);

    setBusy(false);

    if (!imported.ok) {
      setError(imported.error);
      return;
    }

    onImported(requests.map((r) => r.importPath));
  }

  return (
    <DialogShell
      open={plan !== null}
      title="Import files"
      wide
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>Cancel</Button>
          <Button
            primary
            disabled={busy || chosen.length === 0 || chosen.some((r) => !r.importPath.trim())}
            onPress={add}
          >
            Add {counted(chosen.length, "file")} to the map
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted">
        Files are added to the map when you save it. Paths are where the game finds them: keep a
        model's textures at the paths the model names.
      </p>
      {plan && rows.length === 0 && !error && (
        <p className="text-sm text-faint">No models or textures found.</p>
      )}
      {missing > 0 && (
        <p role="alert" className="text-sm text-danger">
          {counted(missing, "texture")} missing from both the map and the game. Add them to this
          import, or the models show without them.
        </p>
      )}
      <ul aria-label="Files to import" className="flex flex-col divide-y divide-line">
        {rows.map((row, index) => (
          <li key={row.diskPath} className="flex flex-col gap-1.5 py-2">
            <div className="flex items-center gap-3">
              <Checkbox
                isSelected={row.included}
                onChange={(included) => change(index, { included })}
                aria-label={`Import ${row.diskPath}`}
              >
                <span className="sr-only">Import</span>
              </Checkbox>
              <input
                value={row.importPath}
                onChange={(e) => change(index, { importPath: e.target.value })}
                aria-label={`Path in the map for ${row.diskPath}`}
                disabled={!row.included}
                className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-2.5 py-1.5 font-mono text-xs text-ink outline-none focus:border-focus disabled:opacity-50"
              />
              <span className="w-16 shrink-0 text-right text-xs text-faint">
                {fileSize(row.size)}
              </span>
            </div>
            <p className="truncate pl-7 text-xs text-faint" title={row.diskPath}>
              {row.diskPath}
              {row.convert && " · converted from a text model"}
            </p>
            {row.included && existing.some((path) => samePath(path, row.importPath)) && (
              <p className="pl-7 text-xs text-gold-bright">Replaces the file already there.</p>
            )}
            {row.textures.length > 0 && (
              <ul aria-label="Textures" className="flex flex-col gap-0.5 pl-7">
                {row.textures.map((texture) => (
                  <li key={texture.path} className="flex items-center gap-2 text-xs">
                    <span className="truncate font-mono text-muted">{texture.path}</span>
                    <span
                      className={classes(
                        "shrink-0",
                        texture.status === "missing" && "font-semibold text-danger",
                        texture.status !== "missing" && "text-faint",
                      )}
                    >
                      {TEXTURE_STATUS[texture.status]}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </li>
        ))}
      </ul>
      {error && (
        <p role="alert" className="text-sm text-danger">
          {error}
        </p>
      )}
    </DialogShell>
  );
}
